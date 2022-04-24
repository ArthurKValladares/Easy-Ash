use crate::{device::Device, swapchain::Swapchain};
use anyhow::Result;
use ash::vk;

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl RenderPass {
    pub fn new(device: &Device, swapchain: &Swapchain) -> Result<Self> {
        // TODO: hard-coded to only take a color attachment with a single subpass for now. More work on better abstraction later
        let renderpass_attachments = [vk::AttachmentDescription {
            format: swapchain.surface_data.format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
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
                let framebuffer_attachments = [present_image_view];
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

        Ok(Self {
            render_pass,
            framebuffers,
        })
    }
}
