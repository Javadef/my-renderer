// Swapchain - Window presentation
//
// Manages the chain of images we render to and present to the screen
// Performance-critical: double/triple buffering, mailbox mode

use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;
use super::VulkanDevice;

pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    device: Arc<VulkanDevice>,
}

impl Swapchain {
    pub fn new(
        device: Arc<VulkanDevice>,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::extensions::khr::Surface,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        log::info!("Creating swapchain: {}x{}", width, height);
        
        // Query surface capabilities
        let surface_caps = unsafe {
            surface_loader.get_physical_device_surface_capabilities(
                device.physical_device,
                surface,
            )
        }?;
        
        // Query supported formats
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(
                device.physical_device,
                surface,
            )
        }?;
        
        // Query supported present modes
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(
                device.physical_device,
                surface,
            )
        }?;
        
        // Choose surface format (prefer SRGB)
        let surface_format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .or_else(|| formats.first())
            .context("No suitable surface format")?;
        
        // Choose present mode (prefer IMMEDIATE for lowest latency benchmarking)
        // IMMEDIATE: No vsync, lowest latency, may tear
        // MAILBOX: No vsync, no tearing, triple buffered
        // FIFO: Vsync enabled, guaranteed available
        let present_mode = present_modes
            .iter()
            .copied()
            .find(|&mode| mode == vk::PresentModeKHR::IMMEDIATE)
            .or_else(|| {
                present_modes
                    .iter()
                    .copied()
                    .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            })
            .unwrap_or(vk::PresentModeKHR::FIFO); // FIFO is always supported
        
        log::info!("Present mode: {:?}", present_mode);
        
        // Choose extent
        let extent = if surface_caps.current_extent.width != u32::MAX {
            surface_caps.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    surface_caps.min_image_extent.width,
                    surface_caps.max_image_extent.width,
                ),
                height: height.clamp(
                    surface_caps.min_image_extent.height,
                    surface_caps.max_image_extent.height,
                ),
            }
        };
        
        // Choose image count (triple buffering for performance)
        let mut image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
            image_count = surface_caps.max_image_count;
        }
        
        // Create swapchain
        let swapchain_loader = ash::extensions::khr::Swapchain::new(&device.instance, &device.device);
        
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&create_info, None)
        }?;
        
        // Get swapchain images
        let images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
        }?;
        
        log::info!("Created swapchain with {} images", images.len());
        
        // Create image views
        let image_views: Result<Vec<_>> = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                
                unsafe {
                    device.device.create_image_view(&create_info, None)
                        .context("Failed to create image view")
                }
            })
            .collect();
        
        Ok(Self {
            swapchain,
            swapchain_loader,
            images,
            image_views: image_views?,
            format: surface_format.format,
            extent,
            device,
        })
    }
    
    /// Acquire next image for rendering
    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: vk::Semaphore,
    ) -> Result<(u32, bool)> {
        let result = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                timeout,
                semaphore,
                vk::Fence::null(),
            )
        };
        
        match result {
            Ok((index, suboptimal)) => Ok((index, suboptimal)),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                anyhow::bail!("Swapchain out of date")
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Present rendered image to screen
    pub fn present(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        
        let result = unsafe {
            self.swapchain_loader.queue_present(queue, &present_info)
        };
        
        match result {
            Ok(suboptimal) => Ok(suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &view in &self.image_views {
                self.device.device.destroy_image_view(view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
