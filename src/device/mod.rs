use crate::{
    context::Context,
    entry::Entry,
    pipeline::{GraphicsPipeline, PipelineStages},
    resources::buffer::Buffer,
    surface::Surface,
    swapchain::Swapchain,
    sync::{Fence, ImageMemoryBarrier, Semaphore},
    PushConstant,
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
    pub properties: vk::PhysicalDeviceProperties,
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
        let properties = unsafe { entry.instance.get_physical_device_properties(p_device) };
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
            properties,
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

    pub fn draw_indexed(&self, context: &Context, first_index: u32, index_count: u32) {
        unsafe {
            self.device
                .cmd_draw_indexed(context.command_buffer, index_count, 1, first_index, 0, 1);
        }
    }

    pub fn push_constant(
        &self,
        context: &Context,
        pipeline: &GraphicsPipeline,
        push_constant: &PushConstant,
        data: &[u8],
    ) {
        unsafe {
            self.device.cmd_push_constants(
                context.command_buffer,
                pipeline.layout,
                push_constant.stage.into(),
                push_constant.offset,
                data,
            );
        }
    }

    pub fn queue_submit(
        &self,
        context: &Context,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        fence: &Fence,
        wait_mask: &[vk::PipelineStageFlags],
    ) -> Result<()> {
        let command_buffers = vec![context.command_buffer];

        // TODO: This is pretty bad. Find a better way to handle this and other similar
        // cases with wrapper structs, like the one below
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|semaphore| semaphore.semaphore)
            .collect::<Vec<_>>();
        let signal_semaphores = signal_semaphores
            .iter()
            .map(|semaphore| semaphore.semaphore)
            .collect::<Vec<_>>();

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        unsafe {
            self.device
                .queue_submit(self.present_queue, &[submit_info], fence.fence)?
        };
        Ok(())
    }

    pub fn pipeline_image_barrier(
        &self,
        context: &Context,
        src_stage: PipelineStages,
        dst_stage: PipelineStages,
        image_barriers: &[ImageMemoryBarrier],
    ) {
        let image_barriers = image_barriers
            .iter()
            .map(|image_barrier| image_barrier.raw)
            .collect::<Vec<_>>();
        // TODO: Hook up dependency flags
        unsafe {
            self.device.cmd_pipeline_barrier(
                context.command_buffer,
                src_stage.into(),
                dst_stage.into(),
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }
    }

    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device.device_wait_idle()?;
        }
        Ok(())
    }

    pub unsafe fn clean(&self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
        }
    }
}
