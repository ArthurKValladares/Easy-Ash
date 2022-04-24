use crate::Shader;

pub struct GraphicsProgram {
    vertex_shader: Shader,
    fragment_shader: Shader,
}

impl GraphicsProgram {
    pub fn new(vertex_shader: Shader, fragment_shader: Shader) -> Self {
        Self {
            vertex_shader,
            fragment_shader,
        }
    }
}
