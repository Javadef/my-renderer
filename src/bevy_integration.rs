// =============================================================================
// BEVY CUSTOM RENDERER PLUGIN
// =============================================================================
//
// This module bridges Bevy's ECS with our custom Vulkan renderer.
//
// ARCHITECTURE:
// ┌─────────────────────────────────────────────────────────────────┐
// │ Bevy App (ECS, Assets, Input, Time)                            │
// │   └── CustomVulkanPlugin                                        │
// │       ├── Setup: Initialize Vulkan renderer                     │
// │       ├── Extract: Pull data from ECS                           │
// │       └── Render: Call our Vulkan renderer                      │
// └─────────────────────────────────────────────────────────────────┘
//
// =============================================================================

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use std::sync::{Arc, Mutex};
use crate::backend::VulkanDevice;

// =============================================================================
// RESOURCES
// =============================================================================

/// Wrapper for our Vulkan renderer state
#[derive(Resource)]
pub struct VulkanRenderer {
    /// Our existing renderer (from main.rs App struct)
    /// Wrapped in Arc<Mutex> for thread-safe access from Bevy systems
    pub renderer: Arc<Mutex<crate::App>>,
}

/// Data extracted from Bevy ECS for rendering
#[derive(Resource, Default)]
pub struct ExtractedRenderData {
    pub clear_color: [f32; 4],
    // Future: Add mesh data, transforms, cameras, etc.
}

// =============================================================================
// CUSTOM RENDERER PLUGIN
// =============================================================================

pub struct CustomVulkanPlugin;

impl Plugin for CustomVulkanPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<ExtractedRenderData>()
            
            // Startup systems (run once)
            .add_systems(Startup, setup_vulkan_renderer)
            
            // Main loop systems
            .add_systems(PreUpdate, handle_window_resize)
            .add_systems(Update, extract_render_data)
            .add_systems(PostUpdate, render_vulkan);
    }
}

// =============================================================================
// SYSTEMS
// =============================================================================

/// Initialize Vulkan renderer with Bevy's window
fn setup_vulkan_renderer(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    log::info!("Initializing Vulkan renderer with Bevy integration");
    
    // Get Bevy's primary window
    let window = window_query.single();
    
    // Note: We need to get the actual winit window handle
    // For now, we'll create a placeholder
    // In a full implementation, we'd extract the raw window handle from Bevy
    
    log::info!("Window size: {}x{}", window.width(), window.height());
    
    // Create our renderer
    // TODO: Initialize with proper window handle
    let renderer = Arc::new(Mutex::new(crate::App::new()));
    
    commands.insert_resource(VulkanRenderer { renderer });
    
    log::info!("Vulkan renderer initialized");
}

/// Handle window resize events
fn handle_window_resize(
    mut resize_events: EventReader<WindowResized>,
    vulkan_renderer: Option<ResMut<VulkanRenderer>>,
) {
    if let Some(renderer) = vulkan_renderer {
        for event in resize_events.read() {
            log::debug!("Window resized to {}x{}", event.width, event.height);
            
            // Set resize flag in our renderer
            if let Ok(mut r) = renderer.renderer.lock() {
                if event.width == 0.0 || event.height == 0.0 {
                    r.is_minimized = true;
                } else {
                    r.is_minimized = false;
                    r.needs_resize = true;
                }
            }
        }
    }
}

/// Extract data from Bevy ECS for rendering
fn extract_render_data(
    mut extracted: ResMut<ExtractedRenderData>,
) {
    // For Phase 1, just set the clear color
    extracted.clear_color = [0.1, 0.2, 0.8, 1.0];
    
    // Future: Extract meshes, transforms, cameras
    // Example:
    // for (transform, mesh) in mesh_query.iter() {
    //     extracted.meshes.push(/* ... */);
    // }
}

/// Render frame using our Vulkan renderer
fn render_vulkan(
    vulkan_renderer: Option<ResMut<VulkanRenderer>>,
    extracted: Res<ExtractedRenderData>,
) {
    if let Some(renderer) = vulkan_renderer {
        if let Ok(mut r) = renderer.renderer.lock() {
            // Render using our existing render_frame logic
            match r.render_frame() {
                Ok(rendered) => {
                    if rendered {
                        r.update_fps();
                    }
                }
                Err(e) => {
                    log::error!("Render error: {:?}", e);
                }
            }
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create Bevy app with our custom renderer
pub fn create_bevy_app() -> App {
    let mut app = App::new();
    
    app
        // Minimal Bevy plugins (no default renderer)
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::window::WindowPlugin {
            primary_window: Some(Window {
                title: "Vulkan Renderer (Bevy Integration)".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        })
        
        // Our custom Vulkan renderer
        .add_plugins(CustomVulkanPlugin);
    
    app
}
