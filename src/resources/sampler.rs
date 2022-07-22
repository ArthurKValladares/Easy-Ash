use crate::device::Device;
use anyhow::Result;
use ash::vk;

#[derive(Debug, Copy, Clone)]
pub enum SamplerFilter {
    Linear,
    Nearest,
}

impl From<SamplerFilter> for vk::Filter {
    fn from(filter: SamplerFilter) -> Self {
        match filter {
            SamplerFilter::Linear => vk::Filter::LINEAR,
            SamplerFilter::Nearest => vk::Filter::NEAREST,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SamplerWrapMode {
    Tile,
    Clamp,
    Mirror,
    Border,
}

impl From<SamplerWrapMode> for vk::SamplerAddressMode {
    fn from(wrap_mode: SamplerWrapMode) -> Self {
        match wrap_mode {
            SamplerWrapMode::Tile => vk::SamplerAddressMode::REPEAT,
            SamplerWrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            SamplerWrapMode::Mirror => vk::SamplerAddressMode::MIRRORED_REPEAT,
            SamplerWrapMode::Border => vk::SamplerAddressMode::CLAMP_TO_BORDER,
        }
    }
}

pub struct Sampler {
    pub sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: &Device, filer: SamplerFilter, wrap_mode: SamplerWrapMode) -> Result<Self> {
        let sampler_info = vk::SamplerCreateInfo::builder()
            .address_mode_u(wrap_mode.into())
            .address_mode_v(wrap_mode.into())
            .address_mode_w(wrap_mode.into())
            .anisotropy_enable(false)
            .min_filter(filer.into())
            .mag_filter(filer.into())
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE);

        let sampler = unsafe { device.device.create_sampler(&sampler_info, None)? };

        Ok(Self { sampler })
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_sampler(self.sampler, None);
    }
}
