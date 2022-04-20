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

    pub fn begin(&self, device: &Device) -> Result<()> {
        unsafe {
            device.device.reset_command_buffer(
                self.command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?
        };

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device
                .device
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)?
        };

        Ok(())
    }

    pub fn end(&self, device: &Device) -> Result<()> {
        unsafe { device.device.end_command_buffer(self.command_buffer)? };
        Ok(())
    }

    pub fn record<F>(&self, device: &Device, f: F) -> Result<()>
    where
        F: FnOnce(&Device, &Context),
    {
        self.begin(device)?;
        f(device, &self);
        self.end(&device)?;
        Ok(())
    }
}
