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

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_fence(self.fence, None);
    }
}

impl Fence {
    pub fn wait(&self, device: &Device) -> Result<()> {
        unsafe {
            device
                .device
                .wait_for_fences(&[self.fence], true, std::u64::MAX)?
        };
        Ok(())
    }

    pub fn reset(&self, device: &Device) -> Result<()> {
        unsafe { device.device.reset_fences(&[self.fence])? };
        Ok(())
    }
}
