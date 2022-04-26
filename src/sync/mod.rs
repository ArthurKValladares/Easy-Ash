use crate::device::Device;
use anyhow::Result;
use ash::vk;

pub struct Fence {
    pub fence: vk::Fence,
}

impl Fence {
    pub fn new(device: &Device) -> Result<Self> {
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let fence = unsafe { device.device.create_fence(&fence_create_info, None)? };

        Ok(Self { fence })
    }
}

pub struct Semaphore {
    pub semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &Device) -> Result<Self> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe { device.device.create_semaphore(&create_info, None)? };

        Ok(Self { semaphore })
    }
}
