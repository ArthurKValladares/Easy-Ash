use crate::{device::Device, mem};
use anyhow::Result;
use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageCreationError {
    #[error("could not find memory index for image")]
    CouldNotFindMemoryIndex,
}

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

impl From<vk::Extent2D> for ImageResolution {
    fn from(extent: vk::Extent2D) -> ImageResolution {
        ImageResolution {
            width: extent.width,
            height: extent.height,
            depth: 1,
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

    fn aspect_mask(&self) -> vk::ImageAspectFlags {
        match self {
            ImageType::Color => todo!(),
            ImageType::Depth => vk::ImageAspectFlags::DEPTH,
        }
    }
}

#[derive(Debug)]
pub struct Image {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
}

impl Image {
    pub fn new(device: &Device, resolution: ImageResolution, ty: ImageType) -> Result<Self> {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(ty.format())
            .extent(resolution.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(ty.usage())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.device.create_image(&image_create_info, None)? };
        let image_memory_req = unsafe { device.device.get_image_memory_requirements(image) };
        let image_memory_index = mem::find_memory_type_index(
            &image_memory_req,
            &device.memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .ok_or(ImageCreationError::CouldNotFindMemoryIndex)?;

        let image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let memory = unsafe { device.device.allocate_memory(&image_allocate_info, None)? };

        unsafe { device.device.bind_image_memory(image, memory, 0)? };

        let image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(ty.aspect_mask())
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(image)
            .format(image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let view = unsafe { device.device.create_image_view(&image_view_info, None)? };

        Ok(Self {
            image,
            memory,
            view,
        })
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_image(self.image, None);
        device.device.destroy_image_view(self.view, None);
        device.device.free_memory(self.memory, None);
    }
}
