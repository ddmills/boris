use std::cmp::min;

use super::{block::Block, terrain::Terrain};
use crate::common::noise::noise::FractalNoise;
use bevy::ecs::system::ResMut;

pub fn setup_terrain(mut terrain: ResMut<Terrain>) {
    let seed = 123;
    let mut height = FractalNoise::new(seed, 0.01, 7);
    let mut caves = FractalNoise::new(seed + 1, 0.015, 4);

    let top = terrain.world_size_y() - 1;
    let mountain_height = min(top - 4, 32);
    let magma_level = 2;
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

                // above ground
                if y > surface {
                    terrain.init_block(x, y, z, Block::EMPTY);
                    if y == surface + 1 {
                        terrain.add_sunlight(x, y, z, 15);
                    } else {
                        terrain.set_sunlight(x, y, z, 15);
                    }
                    continue;
                }

                // magma
                if y <= magma_level {
                    terrain.init_block(x, y, z, Block::MAGMA);
                    continue;
                }

                // below ground
                let c = caves.get_3d(x_f32, y_f32, z_f32);

                if c < 0.25 {
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

    println!("..done generating world");
}
