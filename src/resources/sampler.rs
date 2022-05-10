use crate::device::Device;
use anyhow::Result;
use ash::vk;

pub struct Sampler {
    pub sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: &Device) -> Result<Self> {
        // TODO: Make this configurable
        let sampler_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::MIRRORED_REPEAT,
            address_mode_v: vk::SamplerAddressMode::MIRRORED_REPEAT,
            address_mode_w: vk::SamplerAddressMode::MIRRORED_REPEAT,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
            compare_op: vk::CompareOp::NEVER,
            ..Default::default()
        };

        let sampler = unsafe { device.device.create_sampler(&sampler_info, None)? };

        Ok(Self { sampler })
    }
}
