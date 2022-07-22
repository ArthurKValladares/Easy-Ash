use crate::{
    context::Context, descriptors::DescriptorSet, device::Device, push_constant::PushConstant,
    render_pass::RenderPass, shader::graphics_program::GraphicsProgram, swapchain::Swapchain,
};
use anyhow::Result;
use ash::vk;
use std::ffi::{CStr, CString};
use thiserror::Error;

#[derive(Debug)]
pub struct VertexInputData {
    pub bindings: Vec<vk::VertexInputBindingDescription>,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}

impl VertexInputData {
    pub fn create_info(&self) -> vk::PipelineVertexInputStateCreateInfo {
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&self.attributes)
            .vertex_binding_descriptions(&self.bindings)
            .build()
    }
}

#[derive(Debug, Error)]
pub enum PipelineCrationError {
    #[error("Could not create pipeline: {0}")]
    CouldNotCreatePipelines(vk::Result),
}

pub struct GraphicsPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl GraphicsPipeline {
    pub fn new(
        device: &Device,
        swapchain: &Swapchain,
        render_pass: &RenderPass,
        program: &GraphicsProgram,
        vertex_iput_state: Option<VertexInputData>,
        descriptor_sets: &[&DescriptorSet],
        push_constants: &[&PushConstant],
        cull_back_faces: bool,
    ) -> Result<Self> {
        let main_function_name = CString::new("main").unwrap();
        let pipeline_shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(program.vertex_shader.module)
                .name(&main_function_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(program.fragment_shader.module)
                .name(&main_function_name)
                .build(),
        ];
        let viewports = [swapchain.viewport()];
        let scissors = [swapchain.scissor()];

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(if cull_back_faces {
                vk::CullModeFlags::BACK
            } else {
                vk::CullModeFlags::NONE
            })
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);
        let stencil_op = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::ALWAYS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(stencil_op)
            .back(stencil_op);
        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .build()];
        let color_blend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&color_blend_attachments);
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);
        let vertex_input_state = vertex_iput_state.map_or_else(
            || vk::PipelineVertexInputStateCreateInfo::builder().build(),
            |state| state.create_info(),
        );
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let descriptor_set_layouts = descriptor_sets
            .iter()
            .map(|set| set.layout)
            .collect::<Vec<_>>();
        let push_constant_ranges = push_constants
            .iter()
            .map(|pc| pc.to_raw())
            .collect::<Vec<_>>();
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        let layout = unsafe {
            device
                .device
                .create_pipeline_layout(&layout_create_info, None)?
        };

        let pipeline_create_info = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&pipeline_shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .render_pass(render_pass.render_pass)
            .subpass(0)
            .build()];

        let pipeline = unsafe {
            device.device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &pipeline_create_info,
                None,
            )
        }
        .expect("Failed to create graphics pipeline.")[0];

        Ok(Self { layout, pipeline })
    }

    pub fn bind(&self, device: &Device, context: &Context) {
        unsafe {
            device.device.cmd_bind_pipeline(
                context.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            )
        };
    }

    pub fn bind_descriptor_set(
        &self,
        device: &Device,
        context: &Context,
        descriptor_set: &DescriptorSet,
    ) {
        unsafe {
            device.device.cmd_bind_descriptor_sets(
                context.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &[descriptor_set.descriptor_set],
                &[],
            );
        }
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_pipeline(self.pipeline, None);
        device.device.destroy_pipeline_layout(self.layout, None);
    }
}
