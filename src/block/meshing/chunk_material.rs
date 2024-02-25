use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::system::{Commands, Res, ResMut, Resource},
    pbr::{AlphaMode, Material, MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        color::Color,
        mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
        texture::Image,
    },
};

use crate::block::{block_face::BlockFace, world::block::Block};

const ATTRIBUTE_PACKED_BLOCK: MeshVertexAttribute =
    MeshVertexAttribute::new("PackedBlock", 9985136798, VertexFormat::Uint32);

#[derive(Resource)]
pub struct ChunkMaterialRes {
    pub handle: Handle<ChunkMaterial>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform[2]]
    pub color: Color,
    #[uniform[3]]
    pub texture_count: u32,
    #[uniform[4]]
    pub terrain_slice_y: u32,
}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_PACKED_BLOCK.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub fn pack_block(block: Block, dir: BlockFace, ao: VertexAo) -> u32 {
    let t_id = block.texture_idx(); // four bits, 0-15
    let f_id = dir.bit(); // three bits, 0-7
    let ao_id = ao.bit(); // two bits, 0-3

    return (t_id & 15) | ((f_id & 7) << 4) | ((ao_id & 3) << 7);
}

pub enum VertexAo {
    TouchNone,
    TouchOne,
    TouchTwo,
    TouchThree,
}

impl VertexAo {
    pub fn bit(&self) -> u32 {
        match self {
            VertexAo::TouchNone => 0,
            VertexAo::TouchOne => 1,
            VertexAo::TouchTwo => 2,
            VertexAo::TouchThree => 3,
        }
    }

    pub fn from_bit(val: u32) -> VertexAo {
        match val {
            1 => VertexAo::TouchOne,
            2 => VertexAo::TouchTwo,
            3 => VertexAo::TouchThree,
            _ => VertexAo::TouchNone,
        }
    }
}
