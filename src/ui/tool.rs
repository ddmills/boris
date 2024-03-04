use bevy::{
    ecs::{
        query::With,
        system::{Local, Query, Res, ResMut},
    },
    input::{mouse::MouseButton, ButtonInput},
    math::Vec3,
    transform::components::Transform,
};

use crate::{controls::Raycast, Block, Cursor, Terrain};

use super::Toolbar;

#[derive(PartialEq, Clone)]
pub enum Tool {
    PlaceBlocks(Block),
    ClearBlocks,
}

#[derive(Default)]
pub struct ToolState {
    is_dragging: bool,
    start: [u32; 3],
}

pub fn tool_system(
    toolbar: Res<Toolbar>,
    raycast: Res<Raycast>,
    mut terrain: ResMut<Terrain>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut state: Local<ToolState>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
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
                cursor.translation =
                    Vec3::new(min_x as f32, min_y as f32, min_z as f32) + scale / 2.;
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
                cursor.translation =
                    Vec3::new(min_x as f32, min_y as f32, min_z as f32) + scale / 2.;
            }
        }
    }
}

fn min_max(a: u32, b: u32) -> [u32; 2] {
    if a > b {
        [b, a]
    } else {
        [a, b]
    }
}
