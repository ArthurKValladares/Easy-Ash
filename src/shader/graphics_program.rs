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
}
