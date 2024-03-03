use bevy::{
    ecs::system::{Local, Res, ResMut},
    input::{mouse::MouseButton, ButtonInput},
};

use crate::{controls::Raycast, Block, Terrain};

use super::Toolbar;

pub enum Tool {
    PlaceBlocks(Block),
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
) {
    match toolbar.tool {
        Tool::PlaceBlocks(block) => {
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

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        for z in min_z..=max_z {
                            terrain.set_block(x, y, z, block);
                        }
                    }
                }
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
