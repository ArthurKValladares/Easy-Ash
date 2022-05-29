use crate::ShaderStage;
use ash::vk;

pub struct PushConstant {
    pub stage: ShaderStage,
    pub offset: u32,
    pub size: u32,
}

impl PushConstant {
    pub fn to_raw(&self) -> vk::PushConstantRange {
        vk::PushConstantRange {
            stage_flags: self.stage.into(),
            offset: self.offset,
            size: self.size,
        }
    }
}
