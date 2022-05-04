use crate::{
    device::Device,
    entry::Entry,
    surface::{Surface, SurfaceData},
    sync::Semaphore,
};
use anyhow::Result;
use ash::vk;

pub struct Swapchain {
    pub surface: Surface,
    pub surface_data: SurfaceData,
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    fn create_swapchain_structures(
        entry: &Entry,
        device: &Device,
        surface: &Surface,
        old_swapchain: Option<vk::SwapchainKHR>,
        width: u32,
        height: u32,
    ) -> Result<(
        SurfaceData,
        ash::extensions::khr::Swapchain,
        vk::SwapchainKHR,
        Vec<vk::Image>,
        Vec<vk::ImageView>,
    )> {
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
        let loader = ash::extensions::khr::Swapchain::new(&entry.instance, &device.device);

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
            .image_array_layers(1)
            .old_swapchain(if let Some(old_swapchain) = old_swapchain {
                old_swapchain
            } else {
                vk::SwapchainKHR::null()
            });

        let swapchain = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        let present_images = unsafe { loader.get_swapchain_images(swapchain)? };
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

        if let Some(old_swapchain) = old_swapchain {
            unsafe {
                loader.destroy_swapchain(old_swapchain, None);
            }
        }

        Ok((
            surface_data,
            loader,
            swapchain,
            present_images,
            present_image_views,
        ))
    }

    pub fn new(
        entry: &Entry,
        device: &Device,
        surface: Surface,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let (surface_data, loader, swapchain, present_images, present_image_views) =
            Self::create_swapchain_structures(entry, device, &surface, None, width, height)?;
        Ok(Self {
            surface,
            surface_data,
            loader,
            swapchain,
            present_images,
            present_image_views,
        })
    }

    pub fn resize(
        &mut self,
        entry: &Entry,
        device: &Device,
        width: u32,
        height: u32,
    ) -> Result<()> {
        unsafe {
            self.clean_image_views(device);
        }
        let (surface_data, loader, swapchain, present_images, present_image_views) =
            Self::create_swapchain_structures(
                entry,
                device,
                &self.surface,
                Some(self.swapchain),
                width,
                height,
            )?;
        self.surface_data = surface_data;
        self.loader = loader;
        self.swapchain = swapchain;
        self.present_images = present_images;
        self.present_image_views = present_image_views;
        Ok(())
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

    // TODO: Wrapper type
    pub fn viewport(&self) -> vk::Viewport {
        vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.surface_data.resolution.width as f32,
            height: self.surface_data.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    // TODO: Wrapper type
    pub fn scissor(&self) -> vk::Rect2D {
        self.surface_data.resolution.into()
    }

    pub fn acquire_next_image_index(&self, semaphore: &Semaphore) -> Result<u32> {
        // TODO: Hanbdle the bool return
        let (index, _) = unsafe {
            self.loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                semaphore.semaphore,
                vk::Fence::null(),
            )?
        };
        Ok(index)
    }

    pub fn present(
        &self,
        device: &Device,
        wait_semaphores: &[&Semaphore],
        image_indices: &[u32],
    ) -> Result<()> {
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|semaphore| semaphore.semaphore)
            .collect::<Vec<_>>();
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(std::slice::from_ref(&self.swapchain))
            .image_indices(&image_indices);

        unsafe {
            self.loader
                .queue_present(device.present_queue, &present_info)?;
        }
        Ok(())
    }

    unsafe fn clean_image_views(&self, device: &Device) {
        for image_view in &self.present_image_views {
            device.device.destroy_image_view(*image_view, None);
        }
    }

    pub unsafe fn clean(&self, device: &Device) {
        self.clean_image_views(device);
        self.loader.destroy_swapchain(self.swapchain, None);
        self.surface.clean();
    }
}
