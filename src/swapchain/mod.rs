use crate::{device::Device, entry::Entry, surface::Surface};
use anyhow::Result;
use ash::vk;

pub struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
}

impl Swapchain {
    pub fn new(entry: &Entry, device: &Device, surface: &Surface) -> Result<Self> {
        let pre_transform = if surface
            .capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface.capabilities.current_transform
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
        let loader = ash::extensions::khr::Swapchain::new(&entry.instance, &device.device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.raw)
            .min_image_count(surface.desired_image_count)
            .image_color_space(surface.format.color_space)
            .image_format(surface.format.format)
            .image_extent(surface.resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        Ok(Self { loader, swapchain })
    }
}
