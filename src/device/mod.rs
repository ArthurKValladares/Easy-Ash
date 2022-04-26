use crate::{
    context::Context, entry::Entry, resources::buffer::Buffer, surface::Surface,
    swapchain::Swapchain,
};
use anyhow::Result;
use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceCreationError {
    #[error("no device found that fullfills all requirements")]
    NoSuitableDevice,
}

pub struct Device {
    pub p_device: vk::PhysicalDevice,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub device: ash::Device,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,
    pub command_pool: vk::CommandPool,
}

impl Device {
    pub fn new(entry: &Entry, surface: &Surface) -> Result<Self> {
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
        let memory_properties = unsafe {
            entry
                .instance
                .get_physical_device_memory_properties(p_device)
        };

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

        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let command_pool = unsafe { device.create_command_pool(&pool_create_info, None)? };

        Ok(Self {
            p_device,
            memory_properties,
            device,
            queue_family_index,
            present_queue,
            command_pool,
        })
    }

    // TODO: Better name/abstraction
    pub fn set_viewport_and_scissor(&self, context: &Context, swapchain: &Swapchain) {
        // TODO: Don't calculate viewport/scissor on-demand, maybe don't tie to swapchain
        unsafe {
            self.device
                .cmd_set_viewport(context.command_buffer, 0, &[swapchain.viewport()]);
            self.device
                .cmd_set_scissor(context.command_buffer, 0, &[swapchain.scissor()]);
        }
    }

    pub fn bind_index_buffer(&self, context: &Context, buffer: &Buffer) {
        unsafe {
            self.device.cmd_bind_index_buffer(
                context.command_buffer,
                buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
        }
    }
}
