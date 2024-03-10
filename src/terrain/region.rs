use bevy::{
    ecs::system::{Res, ResMut, Resource},
    utils::HashMap,
};

use crate::{common::flood_fill, Terrain};

pub struct Region {
    pub blocks: Vec<[u32; 3]>,
}

#[derive(Resource)]
pub struct Regions {
    regions: HashMap<u32, Region>,
}

fn build_regions(mut regions: ResMut<Regions>, terrain: Res<Terrain>) {
    let seed = [0, 0, 0];
    let mut region = Region { blocks: vec![] };

    flood_fill(seed, |p| {
        let block = terrain.get_block_detail_i32(p[0], p[1], p[2]);
        let is_navigable = block.block.is_navigable();

        if is_navigable {
            region.blocks.push([p[0] as u32, p[1] as u32, p[2] as u32]);
        }

        is_navigable
    });
}
