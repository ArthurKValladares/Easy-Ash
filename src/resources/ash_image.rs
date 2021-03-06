use crate::{
    context::Context,
    device::Device,
    mem,
    pipeline::PipelineStages,
    resources::buffer::{Buffer, BufferType},
    sync::{AccessMask, Fence, ImageMemoryBarrier},
};
use anyhow::Result;
use ash::vk;
use bytes::Bytes;
use image::GenericImageView;
use std::{fs::File, io::BufReader, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageCreationError {
    #[error("could not find memory index for image")]
    CouldNotFindMemoryIndex,
}

pub enum ImageLayout {
    Undefined,
    DepthStencil,
    TransferDest,
}

impl From<ImageLayout> for vk::ImageLayout {
    fn from(layout: ImageLayout) -> Self {
        match layout {
            ImageLayout::Undefined => vk::ImageLayout::UNDEFINED,
            ImageLayout::DepthStencil => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::TransferDest => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct ImageResolution {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl ImageResolution {
    pub fn from_width_height(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            depth: 1,
        }
    }
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
    pub fn format(&self) -> vk::Format {
        match self {
            ImageType::Color => vk::Format::R8G8B8A8_UNORM,
            ImageType::Depth => vk::Format::D16_UNORM,
        }
    }

    pub fn usage(&self) -> vk::ImageUsageFlags {
        match self {
            // TODO: Should not be like this for all color textures, probably
            ImageType::Color => vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            ImageType::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        }
    }

    pub fn aspect_mask(&self) -> vk::ImageAspectFlags {
        match self {
            ImageType::Color => vk::ImageAspectFlags::COLOR,
            ImageType::Depth => vk::ImageAspectFlags::DEPTH,
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub ty: ImageType,
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
            ty,
        })
    }

    fn image_create_commands(
        device: &Device,
        context: &Context,
        fence: &Fence,
        image: &Image,
        image_buffer: &Buffer,
        image_extent: vk::Extent2D,
    ) {
        let layout_transition_barrier = ImageMemoryBarrier::new(
            &image,
            AccessMask::Transfer,
            ImageLayout::Undefined,
            ImageLayout::TransferDest,
        );
        device.pipeline_image_barrier(
            context,
            PipelineStages::BottomOfPipe,
            PipelineStages::Transfer,
            &[layout_transition_barrier],
        );
        // TODO: Abstract better later
        let buffer_copy_regions = vk::BufferImageCopy::builder()
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .build(),
            )
            .image_extent(image_extent.into())
            .build();

        unsafe {
            device.device.cmd_copy_buffer_to_image(
                context.command_buffer,
                image_buffer.buffer,
                image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_copy_regions],
            );
        }
        let texture_barrier_end = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            dst_access_mask: vk::AccessFlags::SHADER_READ,
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image: image.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        unsafe {
            device.device.cmd_pipeline_barrier(
                context.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[texture_barrier_end],
            );
        }
    }

    pub fn from_data_and_dims(
        device: &Device,
        context: &Context,
        fence: &Fence,
        width: u32,
        height: u32,
        image_data: &[u8],
        record: bool,
    ) -> Result<(Self, Buffer)> {
        let image_extent = vk::Extent2D { width, height };
        let image_buffer = Buffer::from_data_with_size(
            device,
            BufferType::Staging,
            image_data,
            (width * height * 4) as u64,
        )?;
        let image = Image::new(
            device,
            ImageResolution::from_width_height(width, height),
            ImageType::Color,
        )?;

        if record {
            context.record(&device, &[], &[], &fence, &[], |device, context| {
                Self::image_create_commands(
                    device,
                    context,
                    fence,
                    &image,
                    &image_buffer,
                    image_extent,
                );
            });
        } else {
            Self::image_create_commands(
                device,
                context,
                fence,
                &image,
                &image_buffer,
                image_extent,
            );
        }

        Ok((image, image_buffer))
    }

    pub fn from_file(
        device: &Device,
        context: &Context,
        fence: &Fence,
        path: impl AsRef<Path>,
        record: bool,
    ) -> Result<(Self, Buffer)> {
        // TODO: hook up file format
        // TODO: More efficient image transfers later
        let im = image::load(BufReader::new(File::open(path)?), image::ImageFormat::Png)?;
        let (width, height) = im.dimensions();
        let image_data = Bytes::from(im.into_bytes());

        Image::from_data_and_dims(device, context, fence, width, height, &image_data, record)
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_image(self.image, None);
        device.device.destroy_image_view(self.view, None);
        device.device.free_memory(self.memory, None);
    }
}
