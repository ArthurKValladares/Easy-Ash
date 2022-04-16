use crate::entry::Entry;
use anyhow::Result;
use raw_window_handle::HasRawWindowHandle;

pub struct SurfaceBuilder {
    loader: ash::extensions::khr::Surface,
}

impl SurfaceBuilder {
    pub fn new(entry: &Entry, window_handle: &dyn HasRawWindowHandle) -> Result<Self> {
        let surface = unsafe {
            ash_window::create_surface(&entry.entry, &entry.instance, window_handle, None)?
        };
        let loader = ash::extensions::khr::Surface::new(&entry.entry, &entry.instance);
        Ok(Self { loader })
    }
}
