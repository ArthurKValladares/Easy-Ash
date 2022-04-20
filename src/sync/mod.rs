use crate::device::Device;
use anyhow::Result;
use ash::vk;

pub struct Receipt {
    fence: vk::Fence,
}

impl Receipt {
    pub fn new(device: &Device) -> Result<Self> {
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let fence = unsafe { device.device.create_fence(&fence_create_info, None)? };

        Ok(Self { fence })
    }
}
