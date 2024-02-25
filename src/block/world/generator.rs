use crate::block::{
    meshing::chunk_meshing::{on_slice_changed, process_dirty_chunks, setup_chunk_meshes},
    slice::slice::{scroll_events, setup_terrain_slice, update_slice_mesh, TerrainSliceChanged},
};

use super::{
    block::Block,
    chunk::{Chunk, DirtyChunk},
    terrain::{Terrain, TerrainModifiedEvent},
};
use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        entity::Entity,
        event::EventReader,
        query::Without,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    math::Vec3,
};
use bracket_noise::prelude::FastNoise;
use ndshape::AbstractShape;

pub struct TerrainGenerator;

impl Plugin for TerrainGenerator {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Terrain::new(5, 3, 5, 64))
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
            .add_systems(Update, on_terrain_modified);
    }
}

fn setup_terrain(mut terrain: ResMut<Terrain>) {
    let mut nz = FastNoise::new();
    nz.set_frequency(0.4);

    for chunk_idx in 0..terrain.chunk_count {
        let chunk_pos = terrain.shape.delinearize(chunk_idx);
        let chunk_world_x = terrain.chunk_size * chunk_pos[0];
        let chunk_world_y = terrain.chunk_size * chunk_pos[1];
        let chunk_world_z = terrain.chunk_size * chunk_pos[2];
        let chunk_size = terrain.chunk_size;

        if let Some(chunk) = terrain.get_chunk_mut(chunk_idx) {
            chunk.chunk_idx = chunk_idx;
            chunk.world_x = chunk_world_x;
            chunk.world_y = chunk_world_y;
            chunk.world_z = chunk_world_z;
            chunk.chunk_size = chunk_size;

            for block_idx in 0..chunk.block_count {
                let pos = chunk.shape.delinearize(block_idx);
                let pvec = Vec3::new(
                    (pos[0] + chunk_world_x) as f32,
                    (pos[1] + chunk_world_y) as f32,
                    (pos[2] + chunk_world_z) as f32,
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
    }
}

fn on_terrain_modified(
    mut commands: Commands,
    mut ev_terrain_mod: EventReader<TerrainModifiedEvent>,
    terrain: Res<Terrain>,
    chunks: Query<(Entity, &Chunk), Without<DirtyChunk>>,
) {
    if ev_terrain_mod.is_empty() {
        return;
    }

    for ev in ev_terrain_mod.read() {
        let [chunk_idx, block_idx] = terrain.get_block_indexes(ev.x, ev.y, ev.z);

        for (entity, chunk) in chunks.iter() {
            if chunk.chunk_idx == chunk_idx {
                commands.entity(entity).insert(DirtyChunk);
            }
        }
    }
}
