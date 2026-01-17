// Vulkan Device - Core GPU interface
//
// Responsibilities:
// - Instance creation with validation layers
// - Physical device selection (prefer discrete GPU)
// - Logical device + queue creation
// - Memory allocator setup

use anyhow::{Context, Result};
use ash::{vk, Entry};
use std::ffi::{CStr, CString};
use std::sync::Arc;

/// Required Vulkan device features for our renderer
const REQUIRED_DEVICE_FEATURES: vk::PhysicalDeviceFeatures = vk::PhysicalDeviceFeatures {
    // Phase 1: Basic requirements
    fill_mode_non_solid: vk::TRUE,
    wide_lines: vk::TRUE,
    sampler_anisotropy: vk::TRUE,
    
    // Phase 5+: Ray tracing requirements (will enable later)
    // shader_int64: vk::TRUE,
    // buffer_device_address: vk::TRUE,
    
    ..unsafe { std::mem::zeroed() }
};

/// Vulkan device wrapper with automatic cleanup
pub struct VulkanDevice {
    // Vulkan handles (order matters for drop!)
    pub allocator: gpu_allocator::vulkan::Allocator,
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub instance: ash::Instance,
    _entry: Entry,
    
    // Queue handles
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family: u32,
    
    // Debug utils (if validation enabled)
    debug_utils: Option<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)>,
    
    // Device properties (cached for performance)
    pub properties: vk::PhysicalDeviceProperties,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl VulkanDevice {
    /// Create Vulkan device
    /// 
    /// # Arguments
    /// * `app_name` - Application name for debugging
    /// * `enable_validation` - Enable Vulkan validation layers (debug only)
    pub fn new(app_name: &str, enable_validation: bool) -> Result<Arc<Self>> {
        log::info!("Creating Vulkan device: {}", app_name);
        
        // Step 1: Load Vulkan library
        let entry = unsafe { Entry::load() }
            .context("Failed to load Vulkan library. Is Vulkan installed?")?;
        
        // Step 2: Create instance
        let instance = Self::create_instance(&entry, app_name, enable_validation)?;
        
        // Step 3: Setup debug messenger if validation enabled
        let debug_utils = if enable_validation {
            Some(Self::setup_debug_messenger(&entry, &instance)?)
        } else {
            None
        };
        
        // Step 4: Pick physical device (GPU)
        let (physical_device, graphics_queue_family) = 
            Self::pick_physical_device(&instance)?;
        
        // Step 5: Create logical device
        let (device, graphics_queue) = 
            Self::create_logical_device(&instance, physical_device, graphics_queue_family)?;
        
        // Step 6: Cache device properties
        let properties = unsafe { 
            instance.get_physical_device_properties(physical_device) 
        };
        let memory_properties = unsafe {
            instance.get_physical_device_memory_properties(physical_device)
        };
        
        log::info!("Selected GPU: {}", 
            unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                .to_string_lossy());
        log::info!("API Version: {}.{}.{}", 
            vk::api_version_major(properties.api_version),
            vk::api_version_minor(properties.api_version),
            vk::api_version_patch(properties.api_version));
        
        // Step 7: Create memory allocator
        let allocator = Self::create_allocator(&instance, physical_device, &device)?;
        
        Ok(Arc::new(Self {
            allocator,
            device,
            physical_device,
            instance,
            _entry: entry,
            graphics_queue,
            graphics_queue_family,
            debug_utils,
            properties,
            memory_properties,
        }))
    }
    
    fn create_instance(
        entry: &Entry,
        app_name: &str,
        enable_validation: bool,
    ) -> Result<ash::Instance> {
        let app_name_cstr = CString::new(app_name)?;
        let engine_name = CString::new("Custom Engine")?;
        
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name_cstr)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);
        
        // Required extensions
        let mut extensions = vec![
            ash::extensions::ext::DebugUtils::name().as_ptr(), // Debug utils
        ];
        
        // Platform-specific surface extensions
        #[cfg(target_os = "windows")]
        {
            extensions.push(ash::extensions::khr::Surface::name().as_ptr());
            extensions.push(ash::extensions::khr::Win32Surface::name().as_ptr());
        }
        
        // Validation layers
        let layer_names = if enable_validation {
            vec![c"VK_LAYER_KHRONOS_validation".as_ptr()]
        } else {
            vec![]
        };
        
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layer_names);
        
        let instance = unsafe { entry.create_instance(&create_info, None) }
            .context("Failed to create Vulkan instance")?;
        
        Ok(instance)
    }
    
    fn setup_debug_messenger(
        entry: &Entry,
        instance: &ash::Instance,
    ) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)> {
        let debug_utils = ash::extensions::ext::DebugUtils::new(entry, instance);
        
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(debug_callback));
        
        let messenger = unsafe {
            debug_utils.create_debug_utils_messenger(&create_info, None)
        }?;
        
        Ok((debug_utils, messenger))
    }
    
    fn pick_physical_device(
        instance: &ash::Instance,
    ) -> Result<(vk::PhysicalDevice, u32)> {
        let devices = unsafe { instance.enumerate_physical_devices() }?;
        
        if devices.is_empty() {
            anyhow::bail!("No Vulkan-capable GPU found");
        }
        
        // Score each device
        let mut best_device = None;
        let mut best_score = 0;
        
        for device in devices {
            let props = unsafe { instance.get_physical_device_properties(device) };
            let features = unsafe { instance.get_physical_device_features(device) };
            
            // Check required features
            if !Self::check_device_features(&features) {
                continue;
            }
            
            // Find graphics queue family
            let queue_families = unsafe {
                instance.get_physical_device_queue_family_properties(device)
            };
            
            let graphics_family = queue_families
                .iter()
                .enumerate()
                .find(|(_, props)| props.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|(i, _)| i as u32);
            
            if let Some(graphics_family) = graphics_family {
                // Score device (prefer discrete GPU)
                let score = match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
                    _ => 1,
                };
                
                if score > best_score {
                    best_score = score;
                    best_device = Some((device, graphics_family));
                }
            }
        }
        
        best_device.ok_or_else(|| anyhow::anyhow!("No suitable GPU found"))
    }
    
    fn check_device_features(features: &vk::PhysicalDeviceFeatures) -> bool {
        // Check all required features are supported
        features.fill_mode_non_solid == vk::TRUE
            && features.sampler_anisotropy == vk::TRUE
    }
    
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family: u32,
    ) -> Result<(ash::Device, vk::Queue)> {
        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family)
            .queue_priorities(&queue_priorities)
            .build();
        
        // Required device extensions
        let extensions = vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::DynamicRendering::name().as_ptr(), // Vulkan 1.3 dynamic rendering
        ];
        
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_extension_names(&extensions)
            .enabled_features(&REQUIRED_DEVICE_FEATURES);
        
        let device = unsafe {
            instance.create_device(physical_device, &create_info, None)
        }?;
        
        let graphics_queue = unsafe {
            device.get_device_queue(graphics_queue_family, 0)
        };
        
        Ok((device, graphics_queue))
    }
    
    fn create_allocator(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<gpu_allocator::vulkan::Allocator> {
        let allocator = gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false, // Enable later for ray tracing
            allocation_sizes: Default::default(),
        })?;
        
        Ok(allocator)
    }
    
    /// Wait for device to be idle (e.g., before cleanup)
    pub fn wait_idle(&self) -> Result<()> {
        unsafe { self.device.device_wait_idle() }?;
        Ok(())
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        log::info!("Destroying Vulkan device...");
        
        // Wait for device to finish
        let _ = self.wait_idle();
        
        // Cleanup in reverse order
        unsafe {
            if let Some((debug_utils, messenger)) = self.debug_utils.take() {
                debug_utils.destroy_debug_utils_messenger(messenger, None);
            }
            
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

// Debug callback for validation layers
unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("[Vulkan] {}", message.to_string_lossy());
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("[Vulkan] {}", message.to_string_lossy());
        }
        _ => {
            log::debug!("[Vulkan] {}", message.to_string_lossy());
        }
    }
    
    vk::FALSE
}
