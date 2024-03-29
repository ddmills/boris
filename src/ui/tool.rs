use bevy::{
    ecs::{
        event::EventWriter,
        query::With,
        system::{Command, Commands, Local, Query, Res, ResMut},
    },
    input::{mouse::MouseButton, ButtonInput},
    math::Vec3,
    transform::components::Transform,
};

use crate::{
    colonists::{
        Job, JobLocation, JobMine, JobType, Partition, PartitionDebug, PartitionGraph,
        SpawnColonistEvent,
    },
    common::min_max,
    controls::Raycast,
    debug::debug_settings::DebugSettings,
    items::SpawnPickaxeEvent,
    Block, Cursor, Terrain,
};

use super::Toolbar;

#[derive(PartialEq, Clone)]
pub enum Tool {
    PlaceBlocks(Block),
    TogglePathDebug,
    ClearBlocks,
    SpawnColonist,
    SpawnPickaxe,
    BlockInfo,
    Mine,
}

#[derive(Default)]
pub struct ToolState {
    is_dragging: bool,
    start: [u32; 3],
}

pub fn tool_system(
    mut cmd: Commands,
    toolbar: Res<Toolbar>,
    raycast: Res<Raycast>,
    graph: Res<PartitionGraph>,
    mut terrain: ResMut<Terrain>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut state: Local<ToolState>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
    mut ev_spawn_colonist: EventWriter<SpawnColonistEvent>,
    mut ev_spawn_pickaxe: EventWriter<SpawnPickaxeEvent>,
    mut partition_debug: ResMut<PartitionDebug>,
    mut debug_settings: ResMut<DebugSettings>,
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

                    let flags = graph.get_partition_flags(partition_id);

                    println!(
                        "region id = {}, partition id={}, chunk={}, neighbors={}, flags={}, is_computed={}, cost={}",
                        partition.region_id,
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
        Tool::Mine => {
            let mut cursor = cursor_query.get_single_mut().unwrap();

            if mouse_input.just_released(MouseButton::Right) {
                state.is_dragging = false;
                cursor.scale = Vec3::ZERO;
                return;
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
                            if terrain.get_block(x, y, z).is_filled() {
                                cmd.spawn((
                                    Job {
                                        job_type: JobType::Mine,
                                        assignee: None,
                                    },
                                    JobMine,
                                    JobLocation { pos: [x, y, z] },
                                ));
                            }
                        }
                    }
                }
            }
        }
        Tool::TogglePathDebug => {
            if mouse_input.just_released(MouseButton::Left) {
                debug_settings.path = !debug_settings.path;
            }
        }
        Tool::SpawnPickaxe => {
            if !raycast.is_adj_hit {
                return;
            }

            if mouse_input.just_released(MouseButton::Left) {
                ev_spawn_pickaxe.send(SpawnPickaxeEvent {
                    pos: raycast.adj_pos,
                });
            }
        }
    }
}
