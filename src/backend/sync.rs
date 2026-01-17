// Synchronization primitives
//
// Fences, semaphores for GPU-CPU and GPU-GPU sync
// Critical for correct and efficient multi-frame rendering

use ash::vk;
use anyhow::Result;
use std::sync::Arc;
use super::VulkanDevice;

/// Frame synchronization - one per frame in flight
pub struct FrameSync {
    pub image_available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl FrameSync {
    pub fn new(device: &Arc<VulkanDevice>) -> Result<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED); // Start signaled
        
        unsafe {
            Ok(Self {
                image_available: device.device.create_semaphore(&semaphore_info, None)?,
                render_finished: device.device.create_semaphore(&semaphore_info, None)?,
                in_flight_fence: device.device.create_fence(&fence_info, None)?,
            })
        }
    }
    
    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_fence(self.in_flight_fence, None);
        }
    }
}
