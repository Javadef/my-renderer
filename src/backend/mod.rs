// Backend module - Vulkan abstraction layer
// 
// Design: Thin wrapper around ash with safety and ergonomics
// Performance: Zero-cost abstractions, explicit control

pub mod device;
pub mod swapchain;
pub mod sync;
pub mod shader;
pub mod buffer;
pub mod pipeline;

pub use device::VulkanDevice;
pub use swapchain::Swapchain;
