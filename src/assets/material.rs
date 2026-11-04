use crate::engine::math::Vec3;
pub use crate::renderer::shaders::ShaderKind;
pub use crate::renderer::Material;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(usize);

impl MaterialHandle {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(super) fn index(self) -> usize {
        self.0
    }
}

pub fn solid(base_color: Vec3, shader: ShaderKind) -> Material {
    Material {
        base_color,
        emissive: Vec3::default(),
        shader,
        outline: true,
    }
}

pub fn glowing(base_color: Vec3, emissive: Vec3, shader: ShaderKind) -> Material {
    Material {
        emissive,
        ..solid(base_color, shader)
    }
}
