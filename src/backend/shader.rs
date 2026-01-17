// Shader module loading and management
// 
// Vulkan uses SPIR-V bytecode for shaders. This module provides
// utilities to load compiled shaders and create shader modules.

use anyhow::{Context, Result};
use ash::vk;
use super::VulkanDevice;

/// Load SPIR-V shader from bytes and create a shader module
pub fn create_shader_module(device: &VulkanDevice, code: &[u8]) -> Result<vk::ShaderModule> {
    // SPIR-V uses 4-byte words, so we need to convert bytes to u32s
    // Safety: We trust that the shader compiler produces valid aligned data
    let code_aligned = unsafe {
        std::slice::from_raw_parts(
            code.as_ptr() as *const u32,
            code.len() / 4,
        )
    };
    
    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code(code_aligned);
    
    unsafe {
        device.device.create_shader_module(&create_info, None)
            .context("Failed to create shader module")
    }
}

/// Helper to load shader from embedded bytes at compile time
#[macro_export]
macro_rules! load_shader {
    ($device:expr, $path:expr) => {{
        let bytes = include_bytes!($path);
        $crate::backend::shader::create_shader_module($device, bytes)
    }};
}
