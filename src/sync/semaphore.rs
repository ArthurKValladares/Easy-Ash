use crate::device::Device;
use anyhow::Result;
use ash::vk;

#[derive(Copy, Clone)]
pub struct Semaphore {
    pub semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &Device) -> Result<Self> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe { device.device.create_semaphore(&create_info, None)? };

        Ok(Self { semaphore })
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_semaphore(self.semaphore, None);
    }
}
