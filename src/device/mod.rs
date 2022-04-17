use crate::{entry::Entry, surface::SurfaceBuilder};
use anyhow::Result;
use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceCreationError {
    #[error("no device found that fullfills all requirements")]
    NoSuitableDevice,
}

pub struct Device {
    p_device: vk::PhysicalDevice,
    queue_family_index: usize,
}

impl Device {
    pub fn new(entry: &Entry, surface: &SurfaceBuilder) -> Result<Self> {
        let pdevices = unsafe { entry.instance.enumerate_physical_devices()? };
        let (p_device, queue_family_index) = pdevices
            .iter()
            .find_map(|pdevice| unsafe {
                entry
                    .instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface
                                    .loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface.raw,
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .ok_or(DeviceCreationError::NoSuitableDevice)?;
        Ok(Self {
            p_device,
            queue_family_index,
        })
    }
}
