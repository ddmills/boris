use crate::block::{
    meshing::chunk_meshing::{
        on_slice_changed, process_dirty_chunks, setup_chunk_meshes, update_chunk_mesh,
    },
    slice::slice::{scroll_events, setup_terrain_slice, TerrainSliceChanged},
};

use super::{block::Block, terrain::Terrain};
use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{schedule::IntoSystemConfigs, system::ResMut},
    math::Vec3,
};
use ndshape::AbstractShape;

pub struct TerrainGenerator;

impl Plugin for TerrainGenerator {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Terrain::new(5, 3, 6, 32))
            .add_event::<TerrainSliceChanged>()
            .add_systems(
                Startup,
                (setup_terrain, setup_terrain_slice, setup_chunk_meshes).chain(),
            )
            .add_systems(Update, scroll_events)
            .add_systems(Update, process_dirty_chunks)
            .add_systems(Update, on_slice_changed);
    }
}

fn setup_terrain(mut terrain: ResMut<Terrain>) {
    let chunk_size: f32 = terrain.chunk_size as f32;
    let rad = chunk_size / 2.;
    let center = Vec3::new(chunk_size / 2., chunk_size / 2., chunk_size / 2.);

    for chunk_idx in 0..terrain.chunk_count {
        let chunk_pos = terrain.shape.delinearize(chunk_idx);
        let chunk_world_x = terrain.chunk_size * chunk_pos[0];
        let chunk_world_y = terrain.chunk_size * chunk_pos[1];
        let chunk_world_z = terrain.chunk_size * chunk_pos[2];

        if let Some(chunk) = terrain.get_chunk_mut(chunk_idx) {
            chunk.chunk_idx = chunk_idx;
            chunk.world_x = chunk_world_x;
            chunk.world_y = chunk_world_y;
            chunk.world_z = chunk_world_z;

            for block_idx in 0..chunk.block_count {
                let pos = chunk.shape.delinearize(block_idx);
                let pvec = Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32);

                if pvec.distance(center) < rad {
                    chunk.set(block_idx, Block::STONE);
                }
            }
        }
    }
}
