use crate::{device::Device, entry::Entry};
use anyhow::Result;
use ash::vk;
use raw_window_handle::HasRawWindowHandle;

pub struct SurfaceBuilder {
    pub loader: ash::extensions::khr::Surface,
    pub raw: vk::SurfaceKHR,
    window_width: u32,
    window_height: u32,
}

impl SurfaceBuilder {
    pub fn new(
        entry: &Entry,
        window_handle: &dyn HasRawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self> {
        let loader = ash::extensions::khr::Surface::new(&entry.entry, &entry.instance);
        let raw = unsafe {
            ash_window::create_surface(&entry.entry, &entry.instance, window_handle, None)?
        };

        Ok(Self {
            loader,
            raw,
            window_width,
            window_height,
        })
    }

    pub fn build(self, device: &Device) -> Result<Surface> {
        let format = unsafe {
            self.loader
                .get_physical_device_surface_formats(device.p_device, self.raw)?[0]
        };

        let surface_capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(device.p_device, self.raw)?
        };

        let desired_image_count = {
            let mut desired_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0
                && desired_count > surface_capabilities.max_image_count
            {
                desired_count = surface_capabilities.max_image_count;
            }
            desired_count
        };

        let resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: self.window_width,
                height: self.window_height,
            },
            _ => surface_capabilities.current_extent,
        };

        Ok(Surface {
            loader: self.loader,
            raw: self.raw,
            format,
            resolution,
        })
    }
}

pub struct Surface {
    pub loader: ash::extensions::khr::Surface,
    pub raw: vk::SurfaceKHR,
    pub format: vk::SurfaceFormatKHR,
    pub resolution: vk::Extent2D,
}

impl Surface {
    pub fn builder(
        entry: &Entry,
        window_handle: &dyn HasRawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> Result<SurfaceBuilder> {
        SurfaceBuilder::new(entry, window_handle, window_width, window_height)
    }
}
