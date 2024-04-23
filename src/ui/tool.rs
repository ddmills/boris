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
    colonists::{
        Job, NavigationGraph, PartitionDebug, SpawnColonistEvent, SpawnJobBuildEvent,
        SpawnJobMineEvent,
    },
    common::min_max,
    controls::Raycast,
    debug::debug_settings::DebugSettings,
    items::SpawnPickaxeEvent,
    BlockType, Cursor, Terrain,
};

use super::{game_speed, GameSpeed, Toolbar};

#[derive(PartialEq, Clone)]
pub enum Tool {
    PlaceBlocks(BlockType),
    TogglePathDebug,
    ClearBlocks,
    SpawnColonist,
    SpawnPickaxe,
    BuildStone,
    BlockInfo,
    Mine,
    Pause,
}

#[derive(Default)]
pub struct ToolState {
    is_dragging: bool,
    start: [u32; 3],
}

pub fn tool_system(
    toolbar: Res<Toolbar>,
    raycast: Res<Raycast>,
    mut game_speed: ResMut<GameSpeed>,
    graph: Res<NavigationGraph>,
    mut terrain: ResMut<Terrain>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut state: Local<ToolState>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
    mut ev_spawn_colonist: EventWriter<SpawnColonistEvent>,
    mut ev_spawn_pickaxe: EventWriter<SpawnPickaxeEvent>,
    mut ev_spawn_job_build: EventWriter<SpawnJobBuildEvent>,
    mut ev_spawn_job_mine: EventWriter<SpawnJobMineEvent>,
    mut partition_debug: ResMut<PartitionDebug>,
    mut debug_settings: ResMut<DebugSettings>,
    q_jobs: Query<&Job>,
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
                            terrain.set_block_type(x, y, z, block);
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
                            terrain.set_block_type(x, y, z, BlockType::EMPTY);
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

                let count = q_jobs.iter().len();
                println!("JOB COUNT {}", count);

                let hit = raycast.hit_block;
                println!("block {}. blueprint={}", hit.name(), hit.flag_blueprint);

                let [chunk_idx, block_idx] = terrain.get_block_indexes(
                    raycast.adj_pos[0],
                    raycast.adj_pos[1],
                    raycast.adj_pos[2],
                );

                let Some(partition_id) = terrain.get_partition_id(chunk_idx, block_idx) else {
                    println!("no partition");
                    return;
                };

                let partition = graph.get_partition(&partition_id).unwrap();
                partition_debug.partition_id = Some(partition_id);

                println!(
                    "partition_id={}, region_id={}, flags={}",
                    partition_id, partition.region_id, partition.flags
                );

                let region = graph.get_region(&partition.region_id).unwrap();

                for group_id in region.group_ids.iter() {
                    let group = graph.get_group(group_id).unwrap();
                    println!("--> group {} = {}", group_id, group.flags);
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
                            if !terrain.get_block(x, y, z).is_empty() {
                                ev_spawn_job_mine.send(SpawnJobMineEvent { pos: [x, y, z] });
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
        Tool::BuildStone => {
            if !raycast.is_adj_hit {
                return;
            }

            if mouse_input.just_released(MouseButton::Left) {
                ev_spawn_job_build.send(SpawnJobBuildEvent {
                    pos: raycast.adj_pos,
                });
            }
        }
        Tool::Pause => {
            if mouse_input.just_released(MouseButton::Left) {
                game_speed.is_paused = !game_speed.is_paused;
                println!("paused? {}", game_speed.is_paused);
            }
        }
    }
}
