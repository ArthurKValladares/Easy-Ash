use crate::{
    context::Context, descriptors::DescriptorSet, device::Device, push_constant::PushConstant,
    render_pass::RenderPass, shader::graphics_program::GraphicsProgram, swapchain::Swapchain,
};
use anyhow::Result;
use ash::vk;
use std::ffi::CStr;
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
        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: program.vertex_shader.module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: program.fragment_shader.module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let vertex_input_state_ci = vk::PipelineVertexInputStateCreateInfo::builder().build();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [swapchain.viewport()];
        let scissors = [swapchain.scissor()];

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: if cull_back_faces {
                vk::CullModeFlags::BACK
            } else {
                vk::CullModeFlags::NONE
            },
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

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

        let graphic_pipeline_info_builder = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_ci)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .render_pass(render_pass.render_pass);

        let graphic_pipeline_info = if let Some(vertex_input_state) = &vertex_iput_state {
            let create_info = vertex_input_state.create_info();
            graphic_pipeline_info_builder
                .vertex_input_state(&create_info)
                .build()
        } else {
            graphic_pipeline_info_builder.build()
        };

        let graphics_pipelines = unsafe {
            device
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .map_err(|(pipelines, err)| PipelineCrationError::CouldNotCreatePipelines(err))?
        };

        let pipeline = graphics_pipelines[0];

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
