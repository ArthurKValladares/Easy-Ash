use crate::{context::Context, device::Device, swapchain::Swapchain, Image};
use anyhow::Result;
use ash::vk;
use math::vec::Vec4;

#[derive(Debug, Copy, Clone)]
pub enum ClearValue {
    Color(Vec4),
    Depth { depth: f32, stencil: u32 },
}

impl From<ClearValue> for vk::ClearValue {
    fn from(value: ClearValue) -> Self {
        match value {
            ClearValue::Color(vec) => vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: vec.into(),
                },
            },
            ClearValue::Depth { depth, stencil } => vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue { depth, stencil },
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RenderPassAttachment {
    ColorClear,
    ColorLoad,
    DepthClear,
    DepthLoad,
}

impl RenderPassAttachment {
    fn to_vk(self, format: vk::Format) -> vk::AttachmentDescription {
        match self {
            RenderPassAttachment::ColorClear => vk::AttachmentDescription {
                format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
            RenderPassAttachment::ColorLoad => vk::AttachmentDescription {
                format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::LOAD,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            RenderPassAttachment::DepthClear => vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
            RenderPassAttachment::DepthLoad => vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::LOAD,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        }
    }
}

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    color_attachment: RenderPassAttachment,
    depth_attachment: RenderPassAttachment,
    clear_values: Vec<vk::ClearValue>,
    render_area: vk::Rect2D,
}

impl RenderPass {
    fn create_render_pass_structures(
        device: &Device,
        swapchain: &Swapchain,
        color_attachment: RenderPassAttachment,
        depth_attachment: RenderPassAttachment,
    ) -> Result<(vk::RenderPass, Vec<vk::Framebuffer>, vk::Rect2D)> {
        // TODO: hard-coded for now
        let renderpass_attachments = [
            color_attachment.to_vk(swapchain.surface_data.format.format),
            depth_attachment.to_vk(swapchain.surface_data.format.format),
        ];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass));

        let render_pass = unsafe {
            device
                .device
                .create_render_pass(&renderpass_create_info, None)?
        };

        let framebuffers = swapchain
            .present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, swapchain.depth_image.view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(swapchain.surface_data.resolution.width)
                    .height(swapchain.surface_data.resolution.height)
                    .layers(1);

                unsafe {
                    device
                        .device
                        .create_framebuffer(&frame_buffer_create_info, None)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: Make this configurable
        let render_area = swapchain.surface_data.resolution.into();

        Ok((render_pass, framebuffers, render_area))
    }

    pub fn new(
        device: &Device,
        swapchain: &Swapchain,
        color_attachment: RenderPassAttachment,
        depth_attachment: RenderPassAttachment,
        clear_values: &[ClearValue],
    ) -> Result<Self> {
        let clear_values = clear_values
            .iter()
            .map(|val| {
                let ret: vk::ClearValue = (*val).into();
                ret
            })
            .collect::<Vec<_>>();
        let (render_pass, framebuffers, render_area) = Self::create_render_pass_structures(
            device,
            swapchain,
            color_attachment,
            depth_attachment,
        )?;

        Ok(Self {
            render_pass,
            framebuffers,
            color_attachment,
            depth_attachment,
            clear_values,
            render_area,
        })
    }

    pub fn resize(&mut self, device: &Device, swapchain: &Swapchain) -> Result<()> {
        unsafe {
            self.clean(device);
        }
        let (render_pass, framebuffers, render_area) = Self::create_render_pass_structures(
            device,
            swapchain,
            self.color_attachment,
            self.depth_attachment,
        )?;
        self.render_pass = render_pass;
        self.framebuffers = framebuffers;
        self.render_area = render_area;
        Ok(())
    }

    pub fn begin(&self, device: &Device, context: &Context, present_index: u32) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[present_index as usize])
            .render_area(self.render_area)
            .clear_values(&self.clear_values);

        unsafe {
            device.device.cmd_begin_render_pass(
                context.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            )
        };
    }

    pub fn end(&self, device: &Device, context: &Context) {
        unsafe { device.device.cmd_end_render_pass(context.command_buffer) };
    }

    unsafe fn clean_framebuffers(&self, device: &Device) {
        for framebuffer in &self.framebuffers {
            device.device.destroy_framebuffer(*framebuffer, None);
        }
    }

    pub unsafe fn clean(&self, device: &Device) {
        self.clean_framebuffers(device);
        device.device.destroy_render_pass(self.render_pass, None);
    }
}
