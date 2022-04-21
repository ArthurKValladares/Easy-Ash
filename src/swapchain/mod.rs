use crate::{
    device::Device,
    entry::Entry,
    surface::{Surface, SurfaceData},
};
use anyhow::Result;
use ash::vk;

pub struct Swapchain {
    pub surface: Surface,
    pub surface_data: SurfaceData,
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn new(
        entry: &Entry,
        device: &Device,
        surface: Surface,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let surface_data = SurfaceData::new(&surface, device, width, height)?;

        let pre_transform = if surface_data
            .capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_data.capabilities.current_transform
        };

        let present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device.p_device, surface.raw)?
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(&entry.instance, &device.device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.raw)
            .min_image_count(surface_data.desired_image_count)
            .image_color_space(surface_data.format.color_space)
            .image_format(surface_data.format.format)
            .image_extent(surface_data.resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };

        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_data.format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);
                unsafe { device.device.create_image_view(&create_view_info, None) }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            surface,
            surface_data,
            swapchain_loader,
            swapchain,
            present_images,
            present_image_views,
        })
    }

    pub fn resolution(&self) -> vk::Extent2D {
        self.surface_data.resolution
    }

    pub fn width(&self) -> u32 {
        self.surface_data.resolution.width
    }

    pub fn height(&self) -> u32 {
        self.surface_data.resolution.height
    }
}
