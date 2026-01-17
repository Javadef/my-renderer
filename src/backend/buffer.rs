// Buffer utilities for vertex, index, and uniform buffers
//
// Provides helpers for creating GPU-accessible memory buffers

use anyhow::{Context, Result};
use ash::vk;
use super::VulkanDevice;

/// Helper to create a GPU buffer with specified usage and memory properties
pub fn create_buffer(
    device: &VulkanDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    // Create buffer
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
    let buffer = unsafe {
        device.device.create_buffer(&buffer_info, None)
            .context("Failed to create buffer")?
    };
    
    // Get memory requirements
    let mem_requirements = unsafe {
        device.device.get_buffer_memory_requirements(buffer)
    };
    
    // Find suitable memory type
    let memory_type_index = find_memory_type(
        device,
        mem_requirements.memory_type_bits,
        memory_properties,
    )?;
    
    // Allocate memory
    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type_index);
    
    let buffer_memory = unsafe {
        device.device.allocate_memory(&alloc_info, None)
            .context("Failed to allocate buffer memory")?
    };
    
    // Bind memory to buffer
    unsafe {
        device.device.bind_buffer_memory(buffer, buffer_memory, 0)
            .context("Failed to bind buffer memory")?;
    }
    
    Ok((buffer, buffer_memory))
}

/// Create a buffer and fill it with data
pub fn create_buffer_with_data<T: Copy>(
    device: &VulkanDevice,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size = (std::mem::size_of::<T>() * data.len()) as vk::DeviceSize;
    
    let (buffer, memory) = create_buffer(
        device,
        size,
        usage,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    
    // Copy data to buffer
    unsafe {
        let ptr = device.device.map_memory(
            memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )? as *mut T;
        
        ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        device.device.unmap_memory(memory);
    }
    
    Ok((buffer, memory))
}

/// Find a suitable memory type index
fn find_memory_type(
    device: &VulkanDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32> {
    let mem_properties = unsafe {
        device.instance.get_physical_device_memory_properties(device.physical_device)
    };
    
    for i in 0..mem_properties.memory_type_count {
        let has_type = (type_filter & (1 << i)) != 0;
        let has_properties = mem_properties.memory_types[i as usize]
            .property_flags
            .contains(properties);
        
        if has_type && has_properties {
            return Ok(i);
        }
    }
    
    anyhow::bail!("Failed to find suitable memory type")
}

/// Create a depth buffer image, memory, and view
pub fn create_depth_buffer(
    device: &VulkanDevice,
    extent: vk::Extent2D,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
    let format = vk::Format::D32_SFLOAT;
    
    // Create depth image
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
    let image = unsafe {
        device.device.create_image(&image_info, None)
            .context("Failed to create depth image")?
    };
    
    // Allocate memory
    let mem_requirements = unsafe {
        device.device.get_image_memory_requirements(image)
    };
    
    let memory_type_index = find_memory_type(
        device,
        mem_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    
    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type_index);
    
    let memory = unsafe {
        device.device.allocate_memory(&alloc_info, None)
            .context("Failed to allocate depth image memory")?
    };
    
    unsafe {
        device.device.bind_image_memory(image, memory, 0)
            .context("Failed to bind depth image memory")?;
    }
    
    // Create image view
    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    
    let view = unsafe {
        device.device.create_image_view(&view_info, None)
            .context("Failed to create depth image view")?
    };
    
    Ok((image, memory, view))
}
