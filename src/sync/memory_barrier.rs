use crate::resources::ash_image::{Image, ImageLayout};
use ash::vk;

pub enum AccessMask {
    DepthStencil,
    Transfer,
}

impl From<AccessMask> for vk::AccessFlags {
    fn from(mask: AccessMask) -> Self {
        match mask {
            AccessMask::DepthStencil => {
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }
            AccessMask::Transfer => vk::AccessFlags::TRANSFER_WRITE,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ImageMemoryBarrier {
    pub raw: vk::ImageMemoryBarrier,
}

impl ImageMemoryBarrier {
    pub fn new(
        image: &Image,
        dst_access_mask: AccessMask,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) -> Self {
        let raw = vk::ImageMemoryBarrier::builder()
            .image(image.image)
            .dst_access_mask(dst_access_mask.into())
            .old_layout(old_layout.into())
            .new_layout(new_layout.into())
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(image.ty.aspect_mask())
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .build();

        Self { raw }
    }
}
