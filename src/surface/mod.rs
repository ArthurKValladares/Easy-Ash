use crate::entry::Entry;
use anyhow::Result;
use ash::vk;
use raw_window_handle::HasRawWindowHandle;

pub struct SurfaceBuilder {
    pub loader: ash::extensions::khr::Surface,
    pub raw: vk::SurfaceKHR,
}

impl SurfaceBuilder {
    pub fn new(entry: &Entry, window_handle: &dyn HasRawWindowHandle) -> Result<Self> {
        let loader = ash::extensions::khr::Surface::new(&entry.entry, &entry.instance);
        let raw = unsafe {
            ash_window::create_surface(&entry.entry, &entry.instance, window_handle, None)?
        };
        Ok(Self { loader, raw })
    }
}
