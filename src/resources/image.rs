use crate::{device::Device, util};
use anyhow::Result;
use ash::vk;

#[derive(Debug, Copy, Clone)]
pub struct ImageResolution {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl From<ImageResolution> for vk::Extent3D {
    fn from(resolution: ImageResolution) -> vk::Extent3D {
        vk::Extent3D {
            width: resolution.width,
            height: resolution.height,
            depth: resolution.depth,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ImageType {
    Color,
    Depth,
}

impl ImageType {
    fn format(&self) -> vk::Format {
        match self {
            ImageType::Color => todo!(),
            ImageType::Depth => vk::Format::D16_UNORM,
        }
    }

    fn usage(&self) -> vk::ImageUsageFlags {
        match self {
            ImageType::Color => todo!(),
            ImageType::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        }
    }
}

#[derive(Debug)]
pub struct Image {
    image: vk::Image,
    memory: vk::DeviceMemory,
}

impl Image {
    pub fn new(device: &Device, resolution: ImageResolution, ty: ImageType) -> Result<Self> {
        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(ty.format())
            .extent(resolution.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(ty.usage())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.device.create_image(&depth_image_create_info, None)? };
        let image_memory_req = unsafe { device.device.get_image_memory_requirements(image) };
        let image_memory_index = util::find_memory_type_index(
            &image_memory_req,
            &device.memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find suitable memory index for depth image.");

        let image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let memory = unsafe { device.device.allocate_memory(&image_allocate_info, None)? };

        Ok(Self { image, memory })
    }
}
