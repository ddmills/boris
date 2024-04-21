use bevy::{
    asset::{Asset, Handle},
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        color::Color,
        mesh::{Mesh, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
        texture::Image,
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BasicMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[uniform[2]]
    pub light: u32,
    #[uniform[3]]
    pub color: Color,
}

impl Material for BasicMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/basic.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/basic.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(2),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
