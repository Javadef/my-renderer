// =============================================================================
// HIGH-PERFORMANCE VULKAN RENDERER - Learning Project
// =============================================================================
//
// This renderer demonstrates core Vulkan concepts with extensive comments
// for learning purposes. Each section explains WHY things are done, not just HOW.
//
// ARCHITECTURE OVERVIEW:
// ┌─────────────────────────────────────────────────────────────────┐
// │  Window (winit)                                                 │
// │    └── Surface (platform-specific window connection)            │
// │          └── Swapchain (double/triple buffered images)          │
// │                └── Command Buffers (GPU instructions)           │
// │                      └── Synchronization (fences, semaphores)   │
// └─────────────────────────────────────────────────────────────────┘
//
// FRAME FLOW:
// 1. Acquire swapchain image (get next image to render to)
// 2. Wait for previous frame using this sync slot
// 3. Submit pre-recorded commands to GPU
// 4. Present rendered image to screen
// 5. Advance to next frame sync slot
//
// =============================================================================

mod backend;

use anyhow::Result;
use ash::vk;
use backend::{VulkanDevice, Swapchain};
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Number of frames that can be "in flight" simultaneously.
/// 
/// WHY 2? This allows CPU to prepare frame N+1 while GPU renders frame N.
/// More frames = more latency but smoother frame pacing.
/// Fewer frames = lower latency but potential CPU/GPU stalls.
const MAX_FRAMES_IN_FLIGHT: usize = 2;

// =============================================================================
// ENTRY POINT
// =============================================================================

fn main() -> Result<()> {
    // Initialize logging - set RUST_LOG=debug for verbose output
    env_logger::init();
    log::info!("Starting Vulkan renderer");

    // Create the event loop (handles window events, input, etc.)
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    
    // Run the application - this blocks until the window is closed
    event_loop.run_app(&mut app)?;
    
    Ok(())
}

// =============================================================================
// APPLICATION STATE
// =============================================================================

/// Main application struct holding all Vulkan resources.
/// 
/// IMPORTANT: Field order matters for Drop! Resources must be destroyed
/// in reverse order of creation to avoid use-after-free.
struct App {
    // ─────────────────────────────────────────────────────────────────────────
    // WINDOW & SURFACE
    // ─────────────────────────────────────────────────────────────────────────
    window: Option<Arc<Window>>,
    surface: Option<vk::SurfaceKHR>,
    surface_loader: Option<ash::extensions::khr::Surface>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // VULKAN CORE
    // ─────────────────────────────────────────────────────────────────────────
    device: Option<Arc<VulkanDevice>>,
    swapchain: Option<Swapchain>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // COMMANDS
    // ─────────────────────────────────────────────────────────────────────────
    command_pool: Option<vk::CommandPool>,
    /// One command buffer per swapchain image (pre-recorded for performance)
    command_buffers: Vec<vk::CommandBuffer>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // SYNCHRONIZATION
    // ─────────────────────────────────────────────────────────────────────────
    /// Sync objects for each frame in flight
    frame_sync: Vec<backend::sync::FrameSync>,
    /// Which sync slot we're currently using (0 to MAX_FRAMES_IN_FLIGHT-1)
    current_frame: usize,
    
    // ─────────────────────────────────────────────────────────────────────────
    // OPTIMIZATION: Pre-allocated arrays to avoid per-frame heap allocations
    // ─────────────────────────────────────────────────────────────────────────
    wait_stages: [vk::PipelineStageFlags; 1],
    
    // ─────────────────────────────────────────────────────────────────────────
    // STATE FLAGS
    // ─────────────────────────────────────────────────────────────────────────
    /// Set to true when window is resized - triggers swapchain recreation
    needs_resize: bool,
    /// Set to true when window is minimized (size = 0) - skip rendering
    is_minimized: bool,
    
    // ─────────────────────────────────────────────────────────────────────────
    // FPS TRACKING
    // ─────────────────────────────────────────────────────────────────────────
    frame_count: u32,
    last_fps_update: Instant,
    last_frame_time: Instant,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            window: None,
            device: None,
            surface: None,
            surface_loader: None,
            swapchain: None,
            command_pool: None,
            command_buffers: Vec::new(),
            frame_sync: Vec::new(),
            current_frame: 0,
            wait_stages: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            needs_resize: false,
            is_minimized: false,
            frame_count: 0,
            last_fps_update: now,
            last_frame_time: now,
        }
    }
    
    // =========================================================================
    // INITIALIZATION
    // =========================================================================
    
    /// Initialize all Vulkan resources.
    /// 
    /// This is called once when the window is created. It sets up:
    /// 1. Vulkan device (GPU interface)
    /// 2. Surface (window connection)
    /// 3. Swapchain (image buffers)
    /// 4. Command pool & buffers
    /// 5. Synchronization primitives
    fn init_vulkan(&mut self, window: Arc<Window>) -> Result<()> {
        log::info!("Initializing Vulkan...");
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 1: Create Vulkan device
        // ─────────────────────────────────────────────────────────────────────
        // Enable validation layers in debug builds for error checking
        let enable_validation = cfg!(debug_assertions);
        let device = VulkanDevice::new("Vulkan Renderer", enable_validation)?;
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 2: Create surface (platform-specific window connection)
        // ─────────────────────────────────────────────────────────────────────
        let entry = unsafe { ash::Entry::load()? };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &device.instance);
        
        let surface = unsafe {
            use raw_window_handle::{HasWindowHandle, HasDisplayHandle, RawWindowHandle, RawDisplayHandle};
            let window_handle = window.window_handle().unwrap().as_raw();
            let display_handle = window.display_handle().unwrap().as_raw();
            
            #[cfg(target_os = "windows")]
            {
                match (display_handle, window_handle) {
                    (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(handle)) => {
                        let hinstance = handle.hinstance.map(|h| h.get() as isize).unwrap_or(0);
                        let hwnd = handle.hwnd.get() as isize;
                        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
                            .hinstance(hinstance as *const std::ffi::c_void)
                            .hwnd(hwnd as *const std::ffi::c_void);
                        let win32_surface_loader = ash::extensions::khr::Win32Surface::new(&entry, &device.instance);
                        win32_surface_loader.create_win32_surface(&create_info, None)?
                    }
                    _ => anyhow::bail!("Unsupported window handle type"),
                }
            }
            
            #[cfg(not(target_os = "windows"))]
            {
                anyhow::bail!("Platform not supported")
            }
        };
        
        // Verify the GPU supports presenting to this surface
        let surface_support = unsafe {
            surface_loader.get_physical_device_surface_support(
                device.physical_device,
                device.graphics_queue_family,
                surface,
            )?
        };
        
        if !surface_support {
            anyhow::bail!("GPU doesn't support presenting to this surface");
        }
        
        // Store device and surface before creating swapchain
        self.device = Some(device.clone());
        self.surface = Some(surface);
        self.surface_loader = Some(surface_loader);
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 3: Create swapchain and related resources
        // ─────────────────────────────────────────────────────────────────────
        self.create_swapchain_resources(&window)?;
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 4: Create synchronization primitives
        // ─────────────────────────────────────────────────────────────────────
        // These don't need to be recreated on resize
        let frame_sync = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| backend::sync::FrameSync::new(&device))
            .collect::<Result<Vec<_>>>()?;
        
        self.frame_sync = frame_sync;
        
        log::info!("Vulkan initialized successfully!");
        Ok(())
    }
    
    /// Create swapchain and command buffers.
    /// 
    /// This is separated from init_vulkan because it needs to be called
    /// again when the window is resized.
    fn create_swapchain_resources(&mut self, window: &Window) -> Result<()> {
        let device = self.device.as_ref().unwrap();
        let surface = self.surface.unwrap();
        let surface_loader = self.surface_loader.as_ref().unwrap();
        
        // Get current window size
        let size = window.inner_size();
        
        // Don't create swapchain if window is minimized (size = 0)
        if size.width == 0 || size.height == 0 {
            self.is_minimized = true;
            return Ok(());
        }
        self.is_minimized = false;
        
        log::info!("Creating swapchain: {}x{}", size.width, size.height);
        
        // ─────────────────────────────────────────────────────────────────────
        // IMPORTANT: Drop old swapchain BEFORE creating new one
        // ─────────────────────────────────────────────────────────────────────
        // The surface can only have one swapchain at a time
        self.swapchain = None;
        
        // ─────────────────────────────────────────────────────────────────────
        // Create new swapchain
        // ─────────────────────────────────────────────────────────────────────
        let swapchain = Swapchain::new(
            device.clone(),
            surface,
            surface_loader,
            size.width,
            size.height,
        )?;
        
        // ─────────────────────────────────────────────────────────────────────
        // Create command pool (if not exists)
        // ─────────────────────────────────────────────────────────────────────
        if self.command_pool.is_none() {
            let pool_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.graphics_queue_family)
                // TRANSIENT: Command buffers are short-lived
                // RESET: Allow individual buffer reset
                .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            
            let command_pool = unsafe { device.device.create_command_pool(&pool_info, None)? };
            self.command_pool = Some(command_pool);
        }
        
        // ─────────────────────────────────────────────────────────────────────
        // Allocate command buffers (one per swapchain image)
        // ─────────────────────────────────────────────────────────────────────
        // Free old command buffers if they exist
        if !self.command_buffers.is_empty() {
            unsafe {
                device.device.free_command_buffers(
                    self.command_pool.unwrap(),
                    &self.command_buffers,
                );
            }
        }
        
        let swapchain_image_count = swapchain.images.len() as u32;
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool.unwrap())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(swapchain_image_count);
        
        let command_buffers = unsafe { device.device.allocate_command_buffers(&alloc_info)? };
        
        // ─────────────────────────────────────────────────────────────────────
        // Pre-record command buffers for each swapchain image
        // ─────────────────────────────────────────────────────────────────────
        Self::record_command_buffers(&device.device, &swapchain, &command_buffers)?;
        
        log::info!("Created {} pre-recorded command buffers", swapchain_image_count);
        
        self.swapchain = Some(swapchain);
        self.command_buffers = command_buffers;
        self.needs_resize = false;
        
        Ok(())
    }
    
    /// Recreate swapchain after window resize.
    /// 
    /// WHY IS THIS NEEDED?
    /// When the window size changes, the old swapchain images are the wrong size.
    /// We must create a new swapchain with correctly sized images.
    fn recreate_swapchain(&mut self) -> Result<()> {
        // Wait for GPU to finish all work before destroying resources
        if let Some(ref device) = self.device {
            device.wait_idle()?;
        }
        
        // Clone the window Arc to avoid borrow conflict
        let window = self.window.clone();
        if let Some(ref win) = window {
            self.create_swapchain_resources(win)?;
        }
        
        Ok(())
    }
    
    // =========================================================================
    // COMMAND RECORDING
    // =========================================================================
    
    /// Pre-record command buffers for all swapchain images.
    /// 
    /// WHY PRE-RECORD?
    /// Recording commands has CPU overhead. For static content (like clearing
    /// the screen), we can record once and resubmit every frame.
    /// 
    /// WHEN YOU ADD DYNAMIC CONTENT:
    /// You'll need to re-record every frame, or use secondary command buffers
    /// for the dynamic parts.
    fn record_command_buffers(
        device: &ash::Device,
        swapchain: &Swapchain,
        command_buffers: &[vk::CommandBuffer],
    ) -> Result<()> {
        // Clear color (RGBA, 0-1 range)
        let clear_color = vk::ClearColorValue {
            float32: [0.1, 0.2, 0.8, 1.0], // Nice blue color
        };
        
        // Which parts of the image to affect (all of it)
        let subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        
        for (i, &cmd) in command_buffers.iter().enumerate() {
            let image = swapchain.images[i];
            
            unsafe {
                // ─────────────────────────────────────────────────────────────
                // Begin recording
                // ─────────────────────────────────────────────────────────────
                let begin_info = vk::CommandBufferBeginInfo::builder();
                device.begin_command_buffer(cmd, &begin_info)?;
                
                // ─────────────────────────────────────────────────────────────
                // IMAGE LAYOUT TRANSITION: UNDEFINED -> TRANSFER_DST
                // ─────────────────────────────────────────────────────────────
                // WHY? Images have "layouts" that optimize memory access patterns.
                // We need TRANSFER_DST layout to use vkCmdClearColorImage.
                let barrier_to_transfer = vk::ImageMemoryBarrier::builder()
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)  // Don't care about old contents
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(subresource_range)
                    .build();
                
                device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TOP_OF_PIPE,  // Wait for: nothing (start of pipeline)
                    vk::PipelineStageFlags::TRANSFER,     // Block: transfer operations
                    vk::DependencyFlags::empty(),
                    &[],  // Memory barriers
                    &[],  // Buffer barriers
                    &[barrier_to_transfer],  // Image barriers
                );
                
                // ─────────────────────────────────────────────────────────────
                // CLEAR THE IMAGE
                // ─────────────────────────────────────────────────────────────
                device.cmd_clear_color_image(
                    cmd,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &clear_color,
                    &[subresource_range],
                );
                
                // ─────────────────────────────────────────────────────────────
                // IMAGE LAYOUT TRANSITION: TRANSFER_DST -> PRESENT_SRC
                // ─────────────────────────────────────────────────────────────
                // WHY? To present the image, it must be in PRESENT_SRC layout.
                let barrier_to_present = vk::ImageMemoryBarrier::builder()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::empty())
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(subresource_range)
                    .build();
                
                device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,  // Block: nothing (end of pipeline)
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier_to_present],
                );
                
                // ─────────────────────────────────────────────────────────────
                // End recording
                // ─────────────────────────────────────────────────────────────
                device.end_command_buffer(cmd)?;
            }
        }
        
        Ok(())
    }
    
    // =========================================================================
    // RENDER LOOP
    // =========================================================================
    
    /// Render a single frame.
    /// 
    /// This is the hot path - called every frame. Keep it lean!
    /// 
    /// FRAME TIMELINE:
    /// ┌──────────────────────────────────────────────────────────────────────┐
    /// │  acquire_image ─┬─> wait_fence ─> submit ─> present ─> next_frame    │
    /// │                 │                                                     │
    /// │  (GPU starts    │   (CPU waits   (GPU      (Display                  │
    /// │   acquiring)    │    if needed)   works)    shows)                   │
    /// └──────────────────────────────────────────────────────────────────────┘
    fn render_frame(&mut self) -> Result<bool> {
        // Skip rendering if minimized
        if self.is_minimized {
            return Ok(false);
        }
        
        // Handle resize if needed
        if self.needs_resize {
            self.recreate_swapchain()?;
            if self.is_minimized {
                return Ok(false);
            }
        }
        
        let device = self.device.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let sync = &self.frame_sync[self.current_frame];
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 1: Acquire next swapchain image
        // ─────────────────────────────────────────────────────────────────────
        // OPTIMIZATION: Do this BEFORE waiting for fence.
        // The GPU can start acquiring while we wait for the previous frame.
        let acquire_result = swapchain.acquire_next_image(
            u64::MAX,  // Timeout (infinite)
            sync.image_available,  // Signal this semaphore when ready
        );
        
        let image_index = match acquire_result {
            Ok((index, suboptimal)) => {
                // Suboptimal means swapchain still works but should be recreated
                if suboptimal {
                    self.needs_resize = true;
                }
                index
            }
            Err(e) => {
                // Swapchain is out of date - recreate it
                if e.to_string().contains("out of date") {
                    self.needs_resize = true;
                    return Ok(false);
                }
                return Err(e);
            }
        };
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 2: Wait for previous frame using this sync slot
        // ─────────────────────────────────────────────────────────────────────
        // WHY WAIT HERE? We have MAX_FRAMES_IN_FLIGHT sync slots.
        // We must wait for the frame that used this slot to complete.
        unsafe {
            device.device.wait_for_fences(
                &[sync.in_flight_fence],
                true,   // Wait for all fences
                u64::MAX,  // Timeout
            )?;
            device.device.reset_fences(&[sync.in_flight_fence])?;
        }
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 3: Submit command buffer
        // ─────────────────────────────────────────────────────────────────────
        let cmd = self.command_buffers[image_index as usize];
        
        let wait_semaphores = [sync.image_available];
        let signal_semaphores = [sync.render_finished];
        let command_buffers = [cmd];
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)      // Wait for image to be available
            .wait_dst_stage_mask(&self.wait_stages) // Which stage waits
            .command_buffers(&command_buffers)      // Commands to execute
            .signal_semaphores(&signal_semaphores); // Signal when done
        
        unsafe {
            device.device.queue_submit(
                device.graphics_queue,
                &[submit_info.build()],
                sync.in_flight_fence,  // Signal this fence when GPU is done
            )?;
        }
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 4: Present the image
        // ─────────────────────────────────────────────────────────────────────
        let present_result = swapchain.present(
            device.graphics_queue,
            image_index,
            &[sync.render_finished],  // Wait for rendering to finish
        );
        
        match present_result {
            Ok(suboptimal) => {
                if suboptimal {
                    self.needs_resize = true;
                }
            }
            Err(_) => {
                self.needs_resize = true;
            }
        }
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 5: Advance to next frame
        // ─────────────────────────────────────────────────────────────────────
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        
        Ok(true)
    }
    
    // =========================================================================
    // FPS TRACKING
    // =========================================================================
    
    fn update_fps(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.frame_count += 1;
        
        // Update title every second
        if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
            let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();
            let fps = self.frame_count as f32 / elapsed;
            
            if let Some(ref window) = self.window {
                let _ = window.set_title(&format!(
                    "Vulkan - {:.0} FPS ({:.2}ms)", 
                    fps, 
                    frame_time * 1000.0
                ));
            }
            
            self.frame_count = 0;
            self.last_fps_update = now;
        }
    }
}

// =============================================================================
// EVENT HANDLING
// =============================================================================

impl ApplicationHandler for App {
    /// Called when the application is ready to create windows.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        
        // Create window
        let window_attributes = WindowAttributes::default()
            .with_title("Vulkan Renderer")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
        
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        
        // Initialize Vulkan
        if let Err(e) = self.init_vulkan(window.clone()) {
            log::error!("Failed to initialize Vulkan: {:?}", e);
            event_loop.exit();
            return;
        }
        
        self.window = Some(window);
    }
    
    /// Handle window events.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            // ─────────────────────────────────────────────────────────────────
            // CLOSE REQUEST
            // ─────────────────────────────────────────────────────────────────
            WindowEvent::CloseRequested => {
                log::info!("Close requested, shutting down...");
                if let Some(ref device) = self.device {
                    let _ = device.wait_idle();
                }
                event_loop.exit();
            }
            
            // ─────────────────────────────────────────────────────────────────
            // WINDOW RESIZED
            // ─────────────────────────────────────────────────────────────────
            WindowEvent::Resized(size) => {
                log::debug!("Window resized to {}x{}", size.width, size.height);
                
                if size.width == 0 || size.height == 0 {
                    self.is_minimized = true;
                } else {
                    self.is_minimized = false;
                    self.needs_resize = true;
                }
            }
            
            // ─────────────────────────────────────────────────────────────────
            // REDRAW REQUESTED
            // ─────────────────────────────────────────────────────────────────
            WindowEvent::RedrawRequested => {
                match self.render_frame() {
                    Ok(rendered) => {
                        if rendered {
                            self.update_fps();
                        }
                    }
                    Err(e) => {
                        log::error!("Render error: {:?}", e);
                    }
                }
            }
            
            // ─────────────────────────────────────────────────────────────────
            // KEYBOARD INPUT (for future use)
            // ─────────────────────────────────────────────────────────────────
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};
                
                if event.state.is_pressed() {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Escape => {
                                event_loop.exit();
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            _ => {}
        }
    }
    
    /// Called when the event loop is about to block waiting for events.
    /// We use this to request continuous redraws for maximum FPS.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

// =============================================================================
// CLEANUP
// =============================================================================

impl Drop for App {
    fn drop(&mut self) {
        log::info!("Cleaning up Vulkan resources...");
        
        if let Some(ref device) = self.device {
            // Wait for GPU to finish before destroying anything
            let _ = device.wait_idle();
            
            unsafe {
                // Destroy in reverse order of creation!
                
                // 1. Sync objects
                for sync in &self.frame_sync {
                    sync.destroy(&device.device);
                }
                
                // 2. Command pool (also frees command buffers)
                if let Some(pool) = self.command_pool {
                    device.device.destroy_command_pool(pool, None);
                }
                
                // 3. Swapchain is dropped automatically
                
                // 4. Surface
                if let (Some(surface), Some(ref loader)) = (self.surface, &self.surface_loader) {
                    loader.destroy_surface(surface, None);
                }
                
                // 5. Device is dropped automatically (Arc)
            }
        }
        
        log::info!("Cleanup complete");
    }
}
