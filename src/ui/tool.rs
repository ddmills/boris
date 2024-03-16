use bevy::{
    ecs::{
        event::EventWriter,
        query::With,
        system::{Local, Query, Res, ResMut},
    },
    input::{mouse::MouseButton, ButtonInput},
    math::Vec3,
    transform::components::Transform,
};

use crate::{
    colonists::{Partition, PartitionDebug, PartitionGraph, SpawnColonistEvent},
    common::min_max,
    controls::Raycast,
    Block, Cursor, Terrain,
};

use super::Toolbar;

#[derive(PartialEq, Clone)]
pub enum Tool {
    PlaceBlocks(Block),
    ClearBlocks,
    SpawnColonist,
    Pathfind,
    PathfindNeighbor,
    BlockInfo,
    JobMine,
}

#[derive(Default)]
pub struct ToolState {
    is_dragging: bool,
    start: [u32; 3],
}

pub fn tool_system(
    toolbar: Res<Toolbar>,
    raycast: Res<Raycast>,
    graph: Res<PartitionGraph>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut terrain: ResMut<Terrain>,
    mut state: Local<ToolState>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
    mut ev_spawn_colonist: EventWriter<SpawnColonistEvent>,
    mut partition_debug: ResMut<PartitionDebug>,
) {
    match toolbar.tool {
        Tool::PlaceBlocks(block) => {
            let mut cursor = cursor_query.get_single_mut().unwrap();

            if mouse_input.just_released(MouseButton::Right) {
                state.is_dragging = false;
                cursor.scale = Vec3::ZERO;
                return;
            }

            if mouse_input.just_released(MouseButton::Left) {
                if !raycast.is_adj_hit {
                    state.is_dragging = false;
                    return;
                }

                if !state.is_dragging {
                    state.is_dragging = true;
                    state.start = raycast.adj_pos;
                    return;
                }

                state.is_dragging = false;

                let [min_x, max_x] = min_max(state.start[0], raycast.adj_pos[0]);
                let [min_y, max_y] = min_max(state.start[1], raycast.adj_pos[1]);
                let [min_z, max_z] = min_max(state.start[2], raycast.adj_pos[2]);

                cursor.scale = Vec3::ZERO;

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        for z in min_z..=max_z {
                            terrain.set_block(x, y, z, block);
                        }
                    }
                }
            }

            if state.is_dragging {
                let [min_x, max_x] = min_max(state.start[0], raycast.adj_pos[0]);
                let [min_y, max_y] = min_max(state.start[1], raycast.adj_pos[1]);
                let [min_z, max_z] = min_max(state.start[2], raycast.adj_pos[2]);

                let scale = Vec3::new(
                    ((max_x - min_x) + 1) as f32,
                    ((max_y - min_y) + 1) as f32,
                    ((max_z - min_z) + 1) as f32,
                );
                cursor.scale = scale;
                cursor.translation = Vec3::new(min_x as f32, min_y as f32, min_z as f32);
            }
        }
        Tool::ClearBlocks => {
            let mut cursor = cursor_query.get_single_mut().unwrap();

            if mouse_input.just_released(MouseButton::Right) {
                state.is_dragging = false;
                cursor.scale = Vec3::ZERO;
                return;
            }

            if mouse_input.just_released(MouseButton::Left) {
                if !raycast.is_hit {
                    state.is_dragging = false;
                    return;
                }

                if !state.is_dragging {
                    state.is_dragging = true;
                    state.start = raycast.hit_pos;
                    return;
                }

                state.is_dragging = false;

                let [min_x, max_x] = min_max(state.start[0], raycast.hit_pos[0]);
                let [min_y, max_y] = min_max(state.start[1], raycast.hit_pos[1]);
                let [min_z, max_z] = min_max(state.start[2], raycast.hit_pos[2]);

                cursor.scale = Vec3::ZERO;

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        for z in min_z..=max_z {
                            terrain.set_block(x, y, z, Block::EMPTY);
                        }
                    }
                }
            }

            if state.is_dragging {
                let [min_x, max_x] = min_max(state.start[0], raycast.hit_pos[0]);
                let [min_y, max_y] = min_max(state.start[1], raycast.hit_pos[1]);
                let [min_z, max_z] = min_max(state.start[2], raycast.hit_pos[2]);

                let scale = Vec3::new(
                    ((max_x - min_x) + 1) as f32,
                    ((max_y - min_y) + 1) as f32,
                    ((max_z - min_z) + 1) as f32,
                );
                cursor.scale = scale;
                cursor.translation = Vec3::new(min_x as f32, min_y as f32, min_z as f32);
            }
        }
        Tool::SpawnColonist => {
            if mouse_input.just_released(MouseButton::Left) {
                if !raycast.is_adj_hit {
                    return;
                }

                ev_spawn_colonist.send(SpawnColonistEvent {
                    pos: raycast.adj_pos,
                });
            }
        }
        Tool::Pathfind => {
            if mouse_input.just_released(MouseButton::Left) {
                if !raycast.is_adj_hit {
                    return;
                }

                // ev_pathfind.send(PathfindEvent {
                //     goals: vec![raycast.adj_pos],
                // });
            }
        }
        Tool::PathfindNeighbor => {
            if mouse_input.just_released(MouseButton::Left) {
                if !raycast.is_adj_hit {
                    return;
                }

                let mut goals = vec![];

                if raycast.hit_pos[0] > 1 {
                    goals.push([
                        raycast.hit_pos[0] - 1,
                        raycast.hit_pos[1],
                        raycast.hit_pos[2],
                    ]);
                }
                if raycast.hit_pos[0] < terrain.world_size_x() - 1 {
                    goals.push([
                        raycast.hit_pos[0] + 1,
                        raycast.hit_pos[1],
                        raycast.hit_pos[2],
                    ]);
                }
                if raycast.hit_pos[1] > 1 {
                    goals.push([
                        raycast.hit_pos[0],
                        raycast.hit_pos[1] - 1,
                        raycast.hit_pos[2],
                    ]);
                }
                if raycast.hit_pos[1] < terrain.world_size_y() - 1 {
                    goals.push([
                        raycast.hit_pos[0],
                        raycast.hit_pos[1] + 1,
                        raycast.hit_pos[2],
                    ]);
                }
                if raycast.hit_pos[2] > 1 {
                    goals.push([
                        raycast.hit_pos[0],
                        raycast.hit_pos[1],
                        raycast.hit_pos[2] - 1,
                    ]);
                }
                if raycast.hit_pos[2] < terrain.world_size_z() - 1 {
                    goals.push([
                        raycast.hit_pos[0],
                        raycast.hit_pos[1],
                        raycast.hit_pos[2] + 1,
                    ]);
                }

                // ev_pathfind.send(PathfindEvent { goals,  });
            }
        }
        Tool::BlockInfo => {
            if mouse_input.just_released(MouseButton::Left) {
                if !raycast.is_adj_hit {
                    return;
                }

                let [chunk_idx, block_idx] = terrain.get_block_indexes(
                    raycast.adj_pos[0],
                    raycast.adj_pos[1],
                    raycast.adj_pos[2],
                );
                let partition_id = terrain.get_partition_id(chunk_idx, block_idx);

                if partition_id != Partition::NONE {
                    let partition = graph.partitions.get(&partition_id).unwrap();
                    partition_debug.id = partition_id;
                    partition_debug.show = true;

                    let flags = graph.get_flags(partition_id);

                    println!(
                        "partition id={}, chunk={}, neighbors={}, flags={}, is_computed={}, cost={}",
                        partition_id,
                        chunk_idx,
                        partition.neighbors.len(),
                        flags,
                        partition.is_computed,
                        partition.extents.traversal_distance,
                    );
                } else {
                    println!("no partition");
                }
            }
        }
        Tool::JobMine => {
            if mouse_input.just_released(MouseButton::Left)
                && raycast.is_hit
                && raycast.hit_block.is_filled()
            {
                println!(
                    "mine block... {},{},{}",
                    raycast.hit_pos[0], raycast.hit_pos[1], raycast.hit_pos[2]
                );
            }
        }
    }
}
