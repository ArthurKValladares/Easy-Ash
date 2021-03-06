pub mod graphics_program;

use crate::device::Device;
use anyhow::Result;
use ash::vk;
use std::{fs::File, path::Path};

// TODO: We should get a lot of reflection data at compile-time
// TODO: Type safery for shader kind (Vertex, Fragment, Compute)?
pub struct Shader {
    pub module: vk::ShaderModule,
}

impl Shader {
    pub fn new(device: &Device, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut spv_file = File::open(path)?;
        let code = ash::util::read_spv(&mut spv_file)?;
        let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);
        let module = unsafe { device.device.create_shader_module(&shader_info, None)? };
        Ok(Self { module })
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_shader_module(self.module, None);
    }
}
