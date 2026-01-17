// =============================================================================
// HIGH-PERFORMANCE VULKAN RENDERER - Learning Project with Bevy Integration
// =============================================================================
//
// This renderer demonstrates core Vulkan concepts with extensive comments
// for learning purposes, now integrated with Bevy's ECS.
//
// ARCHITECTURE OVERVIEW:
// ┌─────────────────────────────────────────────────────────────────┐
// │  Bevy App (ECS, Assets, Input, Time)                            │
// │    └── Custom Vulkan Renderer Plugin                            │
// │          └── Vulkan Device + Swapchain                          │
// │                └── Command Buffers (GPU instructions)           │
// │                      └── Synchronization (fences, semaphores)   │
// └─────────────────────────────────────────────────────────────────┘
//
// FRAME FLOW:
// 1. Bevy Update systems (ECS logic)
// 2. Extract render data from ECS
// 3. Acquire swapchain image
// 4. Wait for previous frame
// 5. Submit pre-recorded commands to GPU
// 6. Present rendered image to screen
//
// =============================================================================

mod backend;
mod config;
#[cfg(feature = "bevy")]
mod bevy_integration;

use anyhow::{Context, Result};
use ash::vk;
use backend::{VulkanDevice, Swapchain};
use config::Config;
use std::sync::Arc;
use std::time::Instant;
use std::fs::OpenOptions;
use std::io::Write;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, Fullscreen},
};

// =============================================================================
// VERTEX DATA & CUBE GEOMETRY
// =============================================================================

/// Vertex structure with position, normal, and color
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

/// Cube vertices with proper normals for lighting (24 vertices - 4 per face)
/// Each face has its own vertices so normals are correct for flat shading
const CUBE_VERTICES: &[Vertex] = &[
    // Front face (Z+) - Red
    Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [0.9, 0.2, 0.2] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [0.9, 0.2, 0.2] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [0.9, 0.2, 0.2] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [0.9, 0.2, 0.2] },
    // Back face (Z-) - Green
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [0.2, 0.9, 0.2] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [0.2, 0.9, 0.2] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [0.2, 0.9, 0.2] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [0.2, 0.9, 0.2] },
    // Right face (X+) - Blue
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [1.0, 0.0, 0.0], color: [0.2, 0.2, 0.9] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0], color: [0.2, 0.2, 0.9] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0], color: [0.2, 0.2, 0.9] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [1.0, 0.0, 0.0], color: [0.2, 0.2, 0.9] },
    // Left face (X-) - Yellow
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0], color: [0.9, 0.9, 0.2] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0, 0.0], color: [0.9, 0.9, 0.2] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0, 0.0, 0.0], color: [0.9, 0.9, 0.2] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0, 0.0], color: [0.9, 0.9, 0.2] },
    // Top face (Y+) - Cyan
    Vertex { position: [-0.5,  0.5,  0.5], normal: [0.0, 1.0, 0.0], color: [0.2, 0.9, 0.9] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [0.0, 1.0, 0.0], color: [0.2, 0.9, 0.9] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 1.0, 0.0], color: [0.2, 0.9, 0.9] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 1.0, 0.0], color: [0.2, 0.9, 0.9] },
    // Bottom face (Y-) - Magenta
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], color: [0.9, 0.2, 0.9] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], color: [0.9, 0.2, 0.9] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0], color: [0.9, 0.2, 0.9] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0], color: [0.9, 0.2, 0.9] },
];

/// Cube indices: 12 triangles (2 per face, 6 faces)
const CUBE_INDICES: &[u16] = &[
    0,  1,  2,   2,  3,  0,  // Front
    4,  5,  6,   6,  7,  4,  // Back
    8,  9,  10,  10, 11, 8,  // Right
    12, 13, 14,  14, 15, 12, // Left
    16, 17, 18,  18, 19, 16, // Top
    20, 21, 22,  22, 23, 20, // Bottom
];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Convert MVP and Model matrices to bytes for push constants
fn matrices_to_bytes(mvp: &glam::Mat4, model: &glam::Mat4) -> [u8; 128] {
    let mut bytes = [0u8; 128];
    let mvp_cols = mvp.to_cols_array();
    let model_cols = model.to_cols_array();
    
    for (i, &val) in mvp_cols.iter().enumerate() {
        let val_bytes = val.to_ne_bytes();
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&val_bytes);
    }
    for (i, &val) in model_cols.iter().enumerate() {
        let val_bytes = val.to_ne_bytes();
        bytes[64 + i * 4..64 + (i + 1) * 4].copy_from_slice(&val_bytes);
    }
    bytes
}

// =============================================================================
// ENTRY POINT
// =============================================================================

fn main() -> Result<()> {
    // Load configuration from config.toml
    let config = Config::load();
    
    // Initialize logging
    init_logging(&config);
    log::info!("Starting Vulkan renderer");
    log::info!("Window: {}x{} ({})", 
        config.window.width, 
        config.window.height,
        if config.window.fullscreen { "fullscreen" } else { "windowed" }
    );
    log::info!("Present mode: {}", config.graphics.present_mode);

    // OPTION 1: Run with Bevy (ECS integration)
    #[cfg(feature = "bevy")]
    {
        let mut app = bevy_integration::create_bevy_app();
        app.run();
        Ok(())
    }
    
    // OPTION 2: Run standalone (original implementation)
    #[cfg(not(feature = "bevy"))]
    {
        let event_loop = EventLoop::new()?;
        let mut app = App::new(config);
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

/// Initialize logging with optional file output for validation errors
fn init_logging(config: &Config) {
    use env_logger::Builder;
    use log::LevelFilter;
    
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.init();
    
    // Create/clear log file if enabled
    if config.debug.log_to_file {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&config.debug.log_file)
        {
            let _ = writeln!(file, "=== Vulkan Renderer Log ===");
            let _ = writeln!(file, "Started: {:?}", std::time::SystemTime::now());
            let _ = writeln!(file);
        }
    }
}

// =============================================================================
// APPLICATION STATE
// =============================================================================

/// Main application struct holding all Vulkan resources.
/// 
/// IMPORTANT: Field order matters for Drop! Resources must be destroyed
/// in reverse order of creation to avoid use-after-free.
/// 
/// This struct is now pub so it can be used from bevy_integration module
pub struct App {
    // ─────────────────────────────────────────────────────────────────────────
    // CONFIGURATION
    // ─────────────────────────────────────────────────────────────────────────
    config: Config,
    
    // ─────────────────────────────────────────────────────────────────────────
    // WINDOW & SURFACE
    // ─────────────────────────────────────────────────────────────────────────
    window: Option<Arc<Window>>,
    surface: Option<vk::SurfaceKHR>,
    surface_loader: Option<ash::extensions::khr::Surface>,
    is_fullscreen: bool,
    
    // ─────────────────────────────────────────────────────────────────────────
    // VULKAN CORE
    // ─────────────────────────────────────────────────────────────────────────
    device: Option<Arc<VulkanDevice>>,
    swapchain: Option<Swapchain>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // RENDERING PIPELINE
    // ─────────────────────────────────────────────────────────────────────────
    render_pass: Option<vk::RenderPass>,
    framebuffers: Vec<vk::Framebuffer>,
    pipeline: Option<vk::Pipeline>,
    pipeline_layout: Option<vk::PipelineLayout>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // DEPTH BUFFER
    // ─────────────────────────────────────────────────────────────────────────
    depth_image: Option<vk::Image>,
    depth_image_memory: Option<vk::DeviceMemory>,
    depth_image_view: Option<vk::ImageView>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // GEOMETRY BUFFERS
    // ─────────────────────────────────────────────────────────────────────────
    vertex_buffer: Option<vk::Buffer>,
    vertex_buffer_memory: Option<vk::DeviceMemory>,
    index_buffer: Option<vk::Buffer>,
    index_buffer_memory: Option<vk::DeviceMemory>,
    
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
    pub needs_resize: bool,
    /// Set to true when window is minimized (size = 0) - skip rendering
    pub is_minimized: bool,
    /// Set to true after focus regained - forces GPU sync before next frame
    needs_sync: bool,
    
    // ─────────────────────────────────────────────────────────────────────────
    // FPS TRACKING
    // ─────────────────────────────────────────────────────────────────────────
    frame_count: u32,
    last_fps_update: Instant,
    last_frame_time: Instant,
    
    // ─────────────────────────────────────────────────────────────────────────
    // ANIMATION
    // ─────────────────────────────────────────────────────────────────────────
    start_time: Instant,
}

impl App {
    pub fn new(config: Config) -> Self {
        let is_fullscreen = config.window.fullscreen;
        let now = Instant::now();
        Self {
            config,
            window: None,
            device: None,
            surface: None,
            surface_loader: None,
            is_fullscreen,
            swapchain: None,
            render_pass: None,
            framebuffers: Vec::new(),
            pipeline: None,
            pipeline_layout: None,
            depth_image: None,
            depth_image_memory: None,
            depth_image_view: None,
            vertex_buffer: None,
            vertex_buffer_memory: None,
            index_buffer: None,
            index_buffer_memory: None,
            command_pool: None,
            command_buffers: Vec::new(),
            frame_sync: Vec::new(),
            current_frame: 0,
            wait_stages: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            needs_resize: false,
            is_minimized: false,
            needs_sync: false,
            frame_count: 0,
            last_fps_update: now,
            last_frame_time: now,
            start_time: now,
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
        // Enable validation layers based on config (and debug build)
        let enable_validation = cfg!(debug_assertions) && self.config.debug.validation_layers;
        let device = VulkanDevice::new(&self.config.window.title, enable_validation)?;
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 2: Create surface (platform-specific window connection)
        // ─────────────────────────────────────────────────────────────────────
        let entry = unsafe { ash::Entry::load()? };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &device.instance);
        
        let surface = unsafe {
            use raw_window_handle::{HasWindowHandle, HasDisplayHandle, RawWindowHandle, RawDisplayHandle};
            let window_handle = window.window_handle()
                .context("Failed to get window handle")?
                .as_raw();
            let display_handle = window.display_handle()
                .context("Failed to get display handle")?
                .as_raw();
            
            #[cfg(target_os = "windows")]
            {
                match (display_handle, window_handle) {
                    (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(handle)) => {
                        let hinstance = handle.hinstance.map(|h| h.get()).unwrap_or(0) as *const std::ffi::c_void;
                        let hwnd = handle.hwnd.get() as *const std::ffi::c_void;
                        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
                            .hinstance(hinstance)
                            .hwnd(hwnd);
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
        // STEP 3.5: Create rendering pipeline and geometry buffers
        // ─────────────────────────────────────────────────────────────────────
        self.create_rendering_resources()?;
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 4: Create synchronization primitives
        // ─────────────────────────────────────────────────────────────────────
        // These don't need to be recreated on resize
        let max_frames = self.config.graphics.max_frames_in_flight;
        let frame_sync = (0..max_frames)
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
        let device = self.device.as_ref()
            .context("Device not initialized")?;
        let surface = self.surface
            .context("Surface not initialized")?;
        let surface_loader = self.surface_loader.as_ref()
            .context("Surface loader not initialized")?;
        
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
        let command_pool = self.command_pool
            .context("Command pool not initialized")?;
        
        if !self.command_buffers.is_empty() {
            unsafe {
                device.device.free_command_buffers(
                    command_pool,
                    &self.command_buffers,
                );
            }
        }
        
        let swapchain_image_count = swapchain.images.len() as u32;
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(swapchain_image_count);
        
        let command_buffers = unsafe { device.device.allocate_command_buffers(&alloc_info)? };
        
        self.swapchain = Some(swapchain);
        self.command_buffers = command_buffers;
        self.needs_resize = false;
        
        Ok(())
    }
    
    /// Create rendering pipeline, shaders, and geometry buffers
    fn create_rendering_resources(&mut self) -> Result<()> {
        let device = self.device.as_ref()
            .context("Device not initialized")?;
        let swapchain = self.swapchain.as_ref()
            .context("Swapchain not initialized")?;
        
        log::info!("Creating rendering resources...");
        
        // ─────────────────────────────────────────────────────────────────────
        // Load shaders
        // ─────────────────────────────────────────────────────────────────────
        let vert_shader = load_shader!(device, "../shaders/cube.vert.spv")?;
        let frag_shader = load_shader!(device, "../shaders/cube.frag.spv")?;
        
        // ─────────────────────────────────────────────────────────────────────
        // Create render pass
        // ─────────────────────────────────────────────────────────────────────
        let render_pass = backend::pipeline::create_render_pass(device, swapchain.format)?;
        self.render_pass = Some(render_pass);
        
        // ─────────────────────────────────────────────────────────────────────
        // Create depth buffer
        // ─────────────────────────────────────────────────────────────────────
        let (depth_image, depth_image_memory, depth_image_view) = 
            backend::buffer::create_depth_buffer(device, swapchain.extent)?;
        self.depth_image = Some(depth_image);
        self.depth_image_memory = Some(depth_image_memory);
        self.depth_image_view = Some(depth_image_view);
        
        // ─────────────────────────────────────────────────────────────────────
        // Create framebuffers
        // ─────────────────────────────────────────────────────────────────────
        let framebuffers = backend::pipeline::create_framebuffers(
            device,
            &swapchain.image_views,
            depth_image_view,
            render_pass,
            swapchain.extent,
        )?;
        self.framebuffers = framebuffers;
        
        // ─────────────────────────────────────────────────────────────────────
        // Create graphics pipeline
        // ─────────────────────────────────────────────────────────────────────
        let (pipeline, pipeline_layout) = backend::pipeline::create_graphics_pipeline(
            device,
            render_pass,
            swapchain.extent,
            vert_shader,
            frag_shader,
        )?;
        
        // Clean up shader modules (no longer needed after pipeline creation)
        unsafe {
            device.device.destroy_shader_module(vert_shader, None);
            device.device.destroy_shader_module(frag_shader, None);
        }
        
        // ─────────────────────────────────────────────────────────────────────
        // Create vertex buffer
        // ─────────────────────────────────────────────────────────────────────
        let (vertex_buffer, vertex_buffer_memory) = backend::buffer::create_buffer_with_data(
            device,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            CUBE_VERTICES,
        )?;
        
        // ─────────────────────────────────────────────────────────────────────
        // Create index buffer
        // ─────────────────────────────────────────────────────────────────────
        let (index_buffer, index_buffer_memory) = backend::buffer::create_buffer_with_data(
            device,
            vk::BufferUsageFlags::INDEX_BUFFER,
            CUBE_INDICES,
        )?;
        
        self.pipeline = Some(pipeline);
        self.pipeline_layout = Some(pipeline_layout);
        self.vertex_buffer = Some(vertex_buffer);
        self.vertex_buffer_memory = Some(vertex_buffer_memory);
        self.index_buffer = Some(index_buffer);
        self.index_buffer_memory = Some(index_buffer_memory);
        
        // ─────────────────────────────────────────────────────────────────────
        // Now record command buffers with the rendering pipeline
        // ─────────────────────────────────────────────────────────────────────
        if let Some(swapchain) = &self.swapchain {
            log::info!("Recording command buffers: {} buffers, {} framebuffers", 
                self.command_buffers.len(), self.framebuffers.len());
            self.record_command_buffers_with_config(&device.device, swapchain, &self.command_buffers)?;
            log::info!("Command buffers recorded with rendering pipeline");
        }
        
        log::info!("Rendering resources created successfully!");
        Ok(())
    }
    
    /// Recreate swapchain after window resize.
    /// 
    /// WHY IS THIS NEEDED?
    /// When the window size changes, the old swapchain images are the wrong size.
    /// We must create a new swapchain with correctly sized images.
    fn recreate_swapchain(&mut self) -> Result<()> {
        // Wait for GPU to finish ALL work before destroying resources
        if let Some(ref device) = self.device {
            device.wait_idle()?;
        }
        
        // Destroy old framebuffers
        if let Some(ref device) = self.device {
            for &framebuffer in &self.framebuffers {
                unsafe {
                    device.device.destroy_framebuffer(framebuffer, None);
                }
            }
            self.framebuffers.clear();
            
            // Destroy old depth buffer
            if let Some(view) = self.depth_image_view.take() {
                unsafe { device.device.destroy_image_view(view, None); }
            }
            if let Some(image) = self.depth_image.take() {
                unsafe { device.device.destroy_image(image, None); }
            }
            if let Some(memory) = self.depth_image_memory.take() {
                unsafe { device.device.free_memory(memory, None); }
            }
        }
        
        // Clone the window Arc to avoid borrow conflict
        let window = self.window.clone();
        if let Some(ref win) = window {
            self.create_swapchain_resources(win)?;
        }
        
        // Recreate depth buffer and framebuffers for new swapchain
        let device = self.device.as_ref().context("Device missing")?;
        let swapchain = self.swapchain.as_ref().context("Swapchain missing")?;
        let render_pass = self.render_pass.context("Render pass missing")?;
        
        // Create new depth buffer
        let (depth_image, depth_image_memory, depth_image_view) = 
            backend::buffer::create_depth_buffer(device, swapchain.extent)?;
        self.depth_image = Some(depth_image);
        self.depth_image_memory = Some(depth_image_memory);
        self.depth_image_view = Some(depth_image_view);
        
        self.framebuffers = backend::pipeline::create_framebuffers(
            device,
            &swapchain.image_views,
            depth_image_view,
            render_pass,
            swapchain.extent,
        )?;
        log::info!("Recreated {} framebuffers", self.framebuffers.len());
        
        // Recreate synchronization primitives after swapchain recreation
        // This ensures all fences start in signaled state and prevents
        // "fence not yet completed" errors when resuming rendering
        for sync in &self.frame_sync {
            sync.destroy(&device.device);
        }
        self.frame_sync.clear();
        
        let max_frames = self.config.graphics.max_frames_in_flight;
        for _ in 0..max_frames {
            self.frame_sync.push(backend::sync::FrameSync::new(device)?);
        }
        log::info!("Recreated {} frame sync objects", self.frame_sync.len());
        
        // Reset current frame to 0 to ensure clean state
        self.current_frame = 0;
        
        Ok(())
    }
    
    // =========================================================================
    // COMMAND RECORDING
    // =========================================================================
    
    /// Pre-record command buffers for all swapchain images.
    /// 
    /// Records commands to render the rotating cube with MVP transformation.
    fn record_command_buffers_with_config(
        &self,
        device: &ash::Device,
        swapchain: &Swapchain,
        command_buffers: &[vk::CommandBuffer],
    ) -> Result<()> {
        let render_pass = self.render_pass.context("Render pass not initialized")?;
        let pipeline = self.pipeline.context("Pipeline not initialized")?;
        let pipeline_layout = self.pipeline_layout.context("Pipeline layout not initialized")?;
        let vertex_buffer = self.vertex_buffer.context("Vertex buffer not initialized")?;
        let index_buffer = self.index_buffer.context("Index buffer not initialized")?;
        
        // Clear values: color and depth
        let color = self.config.graphics.clear_color;
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: color,
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0, // Far plane
                    stencil: 0,
                },
            },
        ];
        
        for (i, &cmd) in command_buffers.iter().enumerate() {
            let framebuffer = self.framebuffers[i];
            
            unsafe {
                // Begin recording
                let begin_info = vk::CommandBufferBeginInfo::builder();
                device.begin_command_buffer(cmd, &begin_info)?;
                
                // Begin render pass
                let render_pass_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(render_pass)
                    .framebuffer(framebuffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: swapchain.extent,
                    })
                    .clear_values(&clear_values);
                
                device.cmd_begin_render_pass(cmd, &render_pass_info, vk::SubpassContents::INLINE);
                
                // Bind pipeline
                device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);
                
                // Bind vertex buffer
                device.cmd_bind_vertex_buffers(cmd, 0, &[vertex_buffer], &[0]);
                
                // Bind index buffer
                device.cmd_bind_index_buffer(cmd, index_buffer, 0, vk::IndexType::UINT16);
                
                // Calculate matrices (animated)
                let time = self.start_time.elapsed().as_secs_f32();
                let (mvp, model) = self.calculate_matrices(time, swapchain.extent);
                
                // Push MVP and model matrices
                let matrix_bytes = matrices_to_bytes(&mvp, &model);
                device.cmd_push_constants(
                    cmd,
                    pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    &matrix_bytes,
                );
                
                // Draw indexed
                device.cmd_draw_indexed(
                    cmd,
                    CUBE_INDICES.len() as u32,
                    1,  // instance count
                    0,  // first index
                    0,  // vertex offset
                    0,  // first instance
                );
                
                // End render pass
                device.cmd_end_render_pass(cmd);
                
                // End recording
                device.end_command_buffer(cmd)?;
            }
        }
        
        Ok(())
    }
    
    /// Calculate Model-View-Projection and Model matrices for the cube
    fn calculate_matrices(&self, time: f32, extent: vk::Extent2D) -> (glam::Mat4, glam::Mat4) {
        use glam::{Mat4, Vec3};
        
        // Model matrix: rotate cube over time
        let model = Mat4::from_rotation_y(time * 0.5)
            * Mat4::from_rotation_x(time * 0.3);
        
        // View matrix: camera positioned at (0, 0, 3) looking at origin
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 3.0),  // eye
            Vec3::ZERO,                 // center
            Vec3::Y,                    // up
        );
        
        // Projection matrix: perspective with 45° FOV
        let aspect = extent.width as f32 / extent.height as f32;
        let mut proj = Mat4::perspective_rh(
            45.0_f32.to_radians(),
            aspect,
            0.1,   // near plane
            100.0, // far plane
        );
        
        // Vulkan has Y pointing down in clip space, flip it
        proj.y_axis.y *= -1.0;
        
        (proj * view * model, model)
    }
    
    // =========================================================================
    // RENDER LOOP
    // =========================================================================
    
    /// Render a single frame.
    /// 
    /// This is the hot path - called every frame. Keep it lean!
    /// Made public for Bevy integration.
    /// 
    /// FRAME TIMELINE:
    /// ┌──────────────────────────────────────────────────────────────────────┐
    /// │  acquire_image ─┬─> wait_fence ─> submit ─> present ─> next_frame    │
    /// │                 │                                                     │
    /// │  (GPU starts    │   (CPU waits   (GPU      (Display                  │
    /// │   acquiring)    │    if needed)   works)    shows)                   │
    /// └──────────────────────────────────────────────────────────────────────┘
    pub fn render_frame(&mut self) -> Result<bool> {
        // Skip rendering if minimized
        if self.is_minimized {
            return Ok(false);
        }
        
        // If we need a full GPU sync (e.g., after focus regain or pause),
        // wait for all GPU work to complete and reset all fences
        if self.needs_sync {
            if let Some(ref device) = self.device {
                device.wait_idle()?;
                // After wait_idle, all work is done, so all fences should be signaled
                // But to be safe, recreate them
                for sync in &self.frame_sync {
                    sync.destroy(&device.device);
                }
                self.frame_sync.clear();
                
                let max_frames = self.config.graphics.max_frames_in_flight;
                for _ in 0..max_frames {
                    self.frame_sync.push(backend::sync::FrameSync::new(device)?);
                }
                self.current_frame = 0;
            }
            self.needs_sync = false;
            log::info!("GPU sync completed, fences reset");
        }
        
        // Handle resize if needed
        if self.needs_resize {
            self.recreate_swapchain()?;
            if self.is_minimized {
                return Ok(false);
            }
        }
        
        // Get required resources (these should always exist after init)
        let device = self.device.as_ref()
            .context("Device not initialized")?;
        let swapchain = self.swapchain.as_ref()
            .context("Swapchain not initialized")?;
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
        // 
        // IMPORTANT: Always wait first, then reset. wait_for_fences returns
        // immediately if the fence is already signaled. This is the canonical
        // Vulkan synchronization pattern.
        unsafe {
            // Wait for fence - returns immediately if already signaled
            // Use a reasonable timeout to prevent infinite hangs
            let wait_result = device.device.wait_for_fences(
                &[sync.in_flight_fence],
                true,
                5_000_000_000, // 5 second timeout
            );
            
            match wait_result {
                Ok(_) => {
                    // Fence is signaled, safe to reset
                    device.device.reset_fences(&[sync.in_flight_fence])?;
                }
                Err(vk::Result::TIMEOUT) => {
                    // Timeout - something is very wrong, trigger recreation
                    log::warn!("Fence wait timeout - triggering swapchain recreation");
                    self.needs_resize = true;
                    return Ok(false);
                }
                Err(vk::Result::NOT_READY) => {
                    // Fence not ready (shouldn't happen with wait, but handle it)
                    // Skip this frame and try again
                    return Ok(false);
                }
                Err(e) => {
                    // Other error - log and skip frame
                    log::error!("Fence wait error: {:?}", e);
                    return Ok(false);
                }
            }
        }
        
        // ─────────────────────────────────────────────────────────────────────
        // STEP 2.5: Re-record command buffer with updated time for animation
        // ─────────────────────────────────────────────────────────────────────
        self.record_command_buffers_with_config(&device.device, swapchain, &self.command_buffers)?;
        
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
        self.current_frame = (self.current_frame + 1) % self.config.graphics.max_frames_in_flight;
        
        Ok(true)
    }
    
    // =========================================================================
    // FULLSCREEN TOGGLE
    // =========================================================================
    
    fn toggle_fullscreen(&mut self) {
        if let Some(ref window) = self.window {
            self.is_fullscreen = !self.is_fullscreen;
            
            if self.is_fullscreen {
                // Enter fullscreen (use current monitor)
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                log::info!("Entered fullscreen mode");
            } else {
                // Exit fullscreen
                window.set_fullscreen(None);
                log::info!("Exited fullscreen mode");
            }
            
            self.needs_resize = true;
        }
    }
    
    // =========================================================================
    // FPS TRACKING
    // =========================================================================
    
    pub fn update_fps(&mut self) {
        if !self.config.debug.show_fps {
            return;
        }
        
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.frame_count += 1;
        
        // Update title every second
        if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
            let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();
            let fps = self.frame_count as f32 / elapsed;
            
            if let Some(ref window) = self.window {
                let mode = if self.is_fullscreen { "fullscreen" } else { "windowed" };
                window.set_title(&format!(
                    "{} - {:.0} FPS ({:.2}ms) [{}]",
                    self.config.window.title,
                    fps,
                    frame_time * 1000.0,
                    mode
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
        
        // Create window with settings from config
        let mut window_attributes = WindowAttributes::default()
            .with_title(&self.config.window.title)
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.config.window.width,
                self.config.window.height,
            ));
        
        // Set fullscreen if configured
        if self.config.window.fullscreen {
            window_attributes = window_attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        
        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                log::error!("Failed to create window: {:?}", e);
                event_loop.exit();
                return;
            }
        };
        
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
            // KEYBOARD INPUT
            // ─────────────────────────────────────────────────────────────────
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};
                
                if event.state.is_pressed() {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            // ESC - Quit application
                            KeyCode::Escape => {
                                log::info!("ESC pressed, exiting...");
                                event_loop.exit();
                            }
                            // F11 - Toggle fullscreen
                            KeyCode::F11 => {
                                self.toggle_fullscreen();
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // ─────────────────────────────────────────────────────────────────
            // FOCUS CHANGE
            // ─────────────────────────────────────────────────────────────────
            WindowEvent::Focused(focused) => {
                if focused {
                    // Window regained focus - might need to sync GPU state
                    // This helps prevent fence errors after the window was in background
                    log::debug!("Window focused, requesting GPU sync");
                    self.needs_sync = true;
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
                
                // 3. Geometry buffers
                if let Some(buffer) = self.index_buffer {
                    device.device.destroy_buffer(buffer, None);
                }
                if let Some(memory) = self.index_buffer_memory {
                    device.device.free_memory(memory, None);
                }
                if let Some(buffer) = self.vertex_buffer {
                    device.device.destroy_buffer(buffer, None);
                }
                if let Some(memory) = self.vertex_buffer_memory {
                    device.device.free_memory(memory, None);
                }
                
                // 4. Pipeline
                if let Some(pipeline) = self.pipeline {
                    device.device.destroy_pipeline(pipeline, None);
                }
                if let Some(layout) = self.pipeline_layout {
                    device.device.destroy_pipeline_layout(layout, None);
                }
                
                // 5. Framebuffers
                for &framebuffer in &self.framebuffers {
                    device.device.destroy_framebuffer(framebuffer, None);
                }
                
                // 5.5. Depth buffer
                if let Some(view) = self.depth_image_view {
                    device.device.destroy_image_view(view, None);
                }
                if let Some(image) = self.depth_image {
                    device.device.destroy_image(image, None);
                }
                if let Some(memory) = self.depth_image_memory {
                    device.device.free_memory(memory, None);
                }
                
                // 6. Render pass
                if let Some(render_pass) = self.render_pass {
                    device.device.destroy_render_pass(render_pass, None);
                }
                
                // 7. Swapchain is dropped automatically
                
                // 8. Surface
                if let (Some(surface), Some(ref loader)) = (self.surface, &self.surface_loader) {
                    loader.destroy_surface(surface, None);
                }
                
                // 9. Device is dropped automatically (Arc)
            }
        }
        
        log::info!("Cleanup complete");
    }
}
