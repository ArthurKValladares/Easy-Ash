use crate::{
    device::Device,
    pipeline::PipelineStages,
    sync::{Fence, Semaphore},
};
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

    pub fn begin(&self, device: &Device, fence: &Fence) -> Result<()> {
        fence.wait(device);
        fence.reset(device);

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

    pub fn end(
        &self,
        device: &Device,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        fence: &Fence,
        wait_mask: &[PipelineStages],
    ) -> Result<()> {
        unsafe { device.device.end_command_buffer(self.command_buffer)? };
        // TODO: better abstraction for wait masks
        let wait_mask = wait_mask
            .iter()
            .map(|stage| (*stage).into())
            .collect::<Vec<vk::PipelineStageFlags>>();
        device.queue_submit(self, wait_semaphores, signal_semaphores, fence, &wait_mask)?;
        Ok(())
    }

    pub fn record<F>(
        &self,
        device: &Device,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        fence: &Fence,
        wait_mask: &[PipelineStages],
        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&Device, &Context),
    {
        self.begin(device, fence)?;
        f(device, &self);
        self.end(
            &device,
            wait_semaphores,
            signal_semaphores,
            fence,
            wait_mask,
        )?;
        Ok(())
    }
}
