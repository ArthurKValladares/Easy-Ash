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
    device: ash::Device,
    queue_family_index: u32,
    present_queue: vk::Queue,
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
                            Some((*pdevice, index as u32))
                        } else {
                            None
                        }
                    })
            })
            .ok_or(DeviceCreationError::NoSuitableDevice)?;

        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_extension_names_raw = [ash::extensions::khr::Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device = unsafe {
            entry
                .instance
                .create_device(p_device, &device_create_info, None)?
        };

        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            p_device,
            device,
            queue_family_index,
            present_queue,
        })
    }
}
