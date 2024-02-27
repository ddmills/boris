use crate::block::{
    light::light_system,
    meshing::chunk_meshing::{on_slice_changed, process_dirty_chunks, setup_chunk_meshes},
    slice::slice::{scroll_events, setup_terrain_slice, update_slice_mesh, TerrainSliceChanged},
};

use super::{
    block::Block,
    terrain::{Terrain, TerrainModifiedEvent},
};
use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{schedule::IntoSystemConfigs, system::ResMut},
    math::Vec3,
};
use bracket_noise::prelude::FastNoise;
use ndshape::AbstractShape;

pub struct TerrainGenerator;

impl Plugin for TerrainGenerator {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Terrain::new(4, 3, 4, 32))
            .add_event::<TerrainSliceChanged>()
            .add_event::<TerrainModifiedEvent>()
            .add_systems(
                Startup,
                (setup_terrain, setup_terrain_slice, setup_chunk_meshes).chain(),
            )
            .add_systems(Update, scroll_events)
            .add_systems(Update, process_dirty_chunks)
            .add_systems(Update, on_slice_changed)
            .add_systems(Update, update_slice_mesh)
            .add_systems(Update, light_system);
    }
}

fn setup_terrain(mut terrain: ResMut<Terrain>) {
    let mut nz = FastNoise::new();
    nz.set_frequency(0.4);

    for chunk_idx in 0..terrain.chunk_count {
        terrain.init_chunk(chunk_idx);

        let chunk = terrain.get_chunk_mut(chunk_idx).unwrap();

        for block_idx in 0..chunk.block_count {
            let pos_local = chunk.shape.delinearize(block_idx);
            let pos_world = [
                pos_local[0] + chunk.world_x,
                pos_local[1] + chunk.world_y,
                pos_local[2] + chunk.world_z,
            ];

            let pvec = Vec3::new(
                pos_world[0] as f32,
                pos_world[1] as f32,
                pos_world[2] as f32,
            );

            let v = nz.get_noise3d(pvec.x / 18., pvec.y / 18., pvec.z / 18.);

            if v < -0.1 {
                chunk.set(block_idx, Block::EMPTY);
            } else if v < 0. {
                chunk.set(block_idx, Block::GRASS);
            } else if v < 0.4 {
                chunk.set(block_idx, Block::DIRT);
            } else if v < 0.7 {
                chunk.set(block_idx, Block::EMPTY);
            } else {
                chunk.set(block_idx, Block::STONE);
            }
        }
    }

    let top = terrain.world_size_y() - 1;
    for x in 0..terrain.world_size_x() {
        for z in 0..terrain.world_size_z() {
            terrain.add_sunlight(x, top, z, 15);
        }
    }
}
