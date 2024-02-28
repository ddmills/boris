use std::cmp::min;

use super::{
    block::Block,
    terrain::{Terrain, TerrainModifiedEvent},
};
use crate::block::{
    light::light_system,
    meshing::chunk_meshing::{on_slice_changed, process_dirty_chunks, setup_chunk_meshes},
    slice::slice::{scroll_events, setup_terrain_slice, update_slice_mesh, TerrainSliceChanged},
};
use crate::common::noise::noise::FractalNoise;
use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{schedule::IntoSystemConfigs, system::ResMut},
    math::Vec3,
};

pub struct TerrainGenerator;

impl Plugin for TerrainGenerator {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Terrain::new(12, 6, 12, 16))
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
    let seed = 200;
    let mut height = FractalNoise::new(seed, 0.01, 7);
    let mut caves = FractalNoise::new(seed + 1, 0.02, 4);

    let top = terrain.world_size_y() - 1;
    let mountain_height = min(top - 4, 32);
    let dirt_depth = 3;

    for chunk_idx in 0..terrain.chunk_count {
        terrain.init_chunk(chunk_idx);
    }

    println!("generating world..");

    for x in 0..terrain.world_size_x() {
        for y in 0..terrain.world_size_y() {
            for z in 0..terrain.world_size_z() {
                let x_f32 = x as f32;
                let y_f32 = y as f32;
                let z_f32 = z as f32;
                let h = height.get_2d(x_f32, z_f32);

                let surface = top - (((h.clamp(0.1, 0.6)) * (mountain_height) as f32) as u32); // 0 to 28

                if y > surface {
                    terrain.init_block(x, y, z, Block::EMPTY);
                    if y == surface + 1 {
                        terrain.add_sunlight(x, y, z, 15);
                    } else {
                        terrain.set_sunlight(x, y, z, 15);
                    }
                } else {
                    if y <= 3 {
                        terrain.init_block(x, y, z, Block::LAVA);
                        terrain.add_light(x, y, z, 8);
                        continue;
                    }

                    let c = caves.get_3d(x_f32, y_f32, z_f32);

                    if c < 0.36 {
                        terrain.init_block(x, y, z, Block::EMPTY);
                    } else {
                        if y == surface {
                            terrain.init_block(x, y, z, Block::GRASS);
                        } else if y > surface - dirt_depth {
                            terrain.init_block(x, y, z, Block::DIRT);
                        } else {
                            terrain.init_block(x, y, z, Block::STONE);
                        }
                    }
                }
            }
        }
    }

    println!("..done generating world");

    // for x in 0..terrain.world_size_x() {
    //     for z in 0..terrain.world_size_z() {
    //         terrain.add_sunlight(x, top, z, 15);
    //     }
    // }
}
