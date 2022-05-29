pub mod graphics_pipeline;

pub use graphics_pipeline::GraphicsPipeline;

#[derive(Debug, Copy, Clone)]
pub enum PipelineStages {
    TopOfPipe,
    DrawIndirect,
    VertexInput,
    VertexShader,
    TesselationControlShader,
    TessalationEvaluationShader,
    GeometryShader,
    FragmentShader,
    EarlyFragmentTests,
    LateFragmentTests,
    ColorAttachmentOutput,
    ComputeShader,
    Transfer,
    BottomOfPipe,
    Host,
    AllGraphics,
    AllCommands,
}

impl From<PipelineStages> for ash::vk::PipelineStageFlags {
    fn from(stages: PipelineStages) -> Self {
        match stages {
            PipelineStages::TopOfPipe => ash::vk::PipelineStageFlags::TOP_OF_PIPE,
            PipelineStages::DrawIndirect => ash::vk::PipelineStageFlags::DRAW_INDIRECT,
            PipelineStages::VertexInput => ash::vk::PipelineStageFlags::VERTEX_INPUT,
            PipelineStages::VertexShader => ash::vk::PipelineStageFlags::VERTEX_SHADER,
            PipelineStages::TesselationControlShader => {
                ash::vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER
            }
            PipelineStages::TessalationEvaluationShader => {
                ash::vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER
            }
            PipelineStages::GeometryShader => ash::vk::PipelineStageFlags::GEOMETRY_SHADER,
            PipelineStages::FragmentShader => ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
            PipelineStages::EarlyFragmentTests => ash::vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            PipelineStages::LateFragmentTests => ash::vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            PipelineStages::ColorAttachmentOutput => {
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            }
            PipelineStages::ComputeShader => ash::vk::PipelineStageFlags::COMPUTE_SHADER,
            PipelineStages::Transfer => ash::vk::PipelineStageFlags::TRANSFER,
            PipelineStages::BottomOfPipe => ash::vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            PipelineStages::Host => ash::vk::PipelineStageFlags::HOST,
            PipelineStages::AllGraphics => ash::vk::PipelineStageFlags::ALL_GRAPHICS,
            PipelineStages::AllCommands => ash::vk::PipelineStageFlags::ALL_COMMANDS,
        }
    }
}
