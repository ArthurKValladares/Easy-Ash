use crate::device::Device;
use crate::Shader;

pub struct GraphicsProgram {
    pub vertex_shader: Shader,
    pub fragment_shader: Shader,
}

impl GraphicsProgram {
    pub fn new(vertex_shader: Shader, fragment_shader: Shader) -> Self {
        Self {
            vertex_shader,
            fragment_shader,
        }
    }

    pub unsafe fn clean(&self, device: &Device) {
        self.vertex_shader.clean(device);
        self.fragment_shader.clean(device);
    }
}
