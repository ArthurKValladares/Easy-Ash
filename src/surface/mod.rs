use crate::{device::Device, entry::Entry};
use anyhow::Result;
use ash::vk;
use raw_window_handle::HasRawWindowHandle;

// TODO: We should not expose surface to the app, it should be a part of the Swpchain.

pub struct Surface {
    pub loader: ash::extensions::khr::Surface,
    pub raw: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(entry: &Entry, window_handle: &dyn HasRawWindowHandle) -> Result<Self> {
        let loader = ash::extensions::khr::Surface::new(&entry.entry, &entry.instance);
        let raw = unsafe {
            ash_window::create_surface(&entry.entry, &entry.instance, window_handle, None)?
        };

        Ok(Self { loader, raw })
    }
}

pub struct SurfaceData {
    pub format: vk::SurfaceFormatKHR,
    pub resolution: vk::Extent2D,
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub desired_image_count: u32,
}

impl SurfaceData {
    pub(crate) fn new(surface: &Surface, device: &Device, width: u32, height: u32) -> Result<Self> {
        let format = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device.p_device, surface.raw)?[0]
        };

        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.p_device, surface.raw)?
        };

        let desired_image_count = {
            let mut desired_count = capabilities.min_image_count + 1;
            if capabilities.max_image_count > 0 && desired_count > capabilities.max_image_count {
                desired_count = capabilities.max_image_count;
            }
            desired_count
        };

        let resolution = match capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D { width, height },
            _ => capabilities.current_extent,
        };

        Ok(SurfaceData {
            format,
            resolution,
            capabilities,
            desired_image_count,
        })
    }
}
