use bevy::{
    asset::Asset,
    ecs::system::Resource,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

use crate::{Block, BlockFace, ATTRIBUTE_BLOCK_LIGHT, ATTRIBUTE_BLOCK_PACKED};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform[2]]
    pub color: Color,
    #[uniform[3]]
    pub texture_count: u32,
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
            ATTRIBUTE_BLOCK_PACKED.at_shader_location(1),
            ATTRIBUTE_BLOCK_LIGHT.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub fn pack_block(block: Block, dir: BlockFace, ao: VertexCornerCount) -> u32 {
    let t_id = block.texture_idx(); // eight bits, 0-256
    let f_id = dir.bit(); // three bits, 0-7
    let ao_id = ao.bit(); // two bits, 0-3
    let mine_bit = if block.flag_mine { 1 } else { 0 }; // one bit;
    let chop_bit = if block.flag_chop { 1 } else { 0 }; // one bit;

    (t_id & 255)
        | ((f_id & 7) << 8)
        | ((ao_id & 3) << 11)
        | ((mine_bit & 1) << 13)
        | ((chop_bit & 1) << 14)
}

pub enum VertexCornerCount {
    None,
    One,
    Two,
    Three,
}

impl VertexCornerCount {
    pub fn bit(&self) -> u32 {
        match self {
            VertexCornerCount::None => 0,
            VertexCornerCount::One => 1,
            VertexCornerCount::Two => 2,
            VertexCornerCount::Three => 3,
        }
    }

    pub fn from_bit(val: u32) -> VertexCornerCount {
        match val {
            1 => VertexCornerCount::One,
            2 => VertexCornerCount::Two,
            3 => VertexCornerCount::Three,
            _ => VertexCornerCount::None,
        }
    }
}
