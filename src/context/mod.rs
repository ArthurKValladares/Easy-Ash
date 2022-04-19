use crate::device::Device;
use anyhow::Result;
use ash::vk;

pub struct Context {
    pub command_buffer: vk::CommandBuffer,
}

impl Context {
    pub fn new(device: &Device) -> Result<Self> {
        // TODO: Allocate buffers in bulk, let Device manage it

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(device.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)?
        };

        Ok(Self {
            command_buffer: command_buffers[0],
        })
    }
}
