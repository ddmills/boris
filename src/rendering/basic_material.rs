use bevy::{
    asset::{Asset, Assets, Handle},
    ecs::{
        query::Changed,
        system::{Query, Res, ResMut},
    },
    pbr::Material,
    reflect::TypePath,
    render::{
        color::Color,
        render_resource::{AsBindGroup, ShaderRef},
        texture::Image,
    },
};

use crate::{colonists::ChildMaterials, Position, Terrain};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BasicMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[uniform[2]]
    pub sunlight: u32,
    #[uniform[3]]
    pub torchlight: u32,
    #[uniform[4]]
    pub color: Color,
}

impl Material for BasicMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/basic.wgsl".into()
    }
}

pub fn update_basic_material_lighting(
    terrain: Res<Terrain>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    q_moved: Query<(&Position, &Handle<BasicMaterial>), Changed<Position>>,
) {
    for (pos, mat_handle) in q_moved.iter() {
        let Some(material) = basic_materials.get_mut(mat_handle) else {
            continue;
        };

        let block = terrain.get_block_by_idx(pos.chunk_idx, pos.block_idx);

        material.torchlight = block.light as u32;
        material.sunlight = block.sunlight as u32;
    }
}

pub fn update_basic_material_children_lighting(
    terrain: Res<Terrain>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    q_moved: Query<(&Position, &ChildMaterials), Changed<Position>>,
) {
    for (pos, child) in q_moved.iter() {
        let Some(material) = basic_materials.get_mut(child.0.clone()) else {
            continue;
        };

        let block = terrain.get_block_by_idx(pos.chunk_idx, pos.block_idx);

        material.torchlight = block.light as u32;
        material.sunlight = block.sunlight as u32;
    }
}
