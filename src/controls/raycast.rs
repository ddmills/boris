use bevy::{
    ecs::{
        query::With,
        system::{Query, Res, ResMut, Resource},
    },
    render::camera::Camera,
    transform::components::GlobalTransform,
    window::{PrimaryWindow, Window},
};

use crate::{ui::Ui, Terrain, TerrainSlice};

use super::MainCamera;

#[derive(Resource)]
pub struct Raycast {
    /// True if a block is hit
    pub is_hit: bool,
    /// The coordinates directly under the raycast
    pub hit_pos: [u32; 3],
    /// True if an adjacent block is hit
    pub is_adj_hit: bool,
    /// The coordinates adjacent to the hit
    pub adj_pos: [u32; 3],
}

pub fn raycast(
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    terrain: Res<Terrain>,
    terrain_slice: Res<TerrainSlice>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui: Res<Ui>,
    mut raycast: ResMut<Raycast>,
) {
    if ui.pointer_captured {
        raycast.is_adj_hit = false;
        raycast.is_hit = false;
        return;
    }

    let (camera, transform) = cameras.single();

    for window in windows.iter() {
        let Some(cursor_pos) = window.cursor_position() else {
            raycast.is_adj_hit = false;
            raycast.is_hit = false;
            return;
        };

        let Some(ray3d) = camera.viewport_to_world(transform, cursor_pos) else {
            raycast.is_adj_hit = false;
            raycast.is_hit = false;
            return;
        };

        let slice_y = terrain_slice.get_value();
        let radius = 256;

        let ray = terrain.raycast(
            ray3d.origin.x,
            ray3d.origin.y,
            ray3d.origin.z,
            ray3d.direction.x,
            ray3d.direction.y,
            ray3d.direction.z,
            slice_y,
            radius,
        );

        if !ray.is_hit {
            raycast.is_adj_hit = false;
            raycast.is_hit = false;
            return;
        }

        raycast.is_hit = true;
        raycast.hit_pos = [ray.x, ray.y, ray.z];

        let offset = ray.face.offset();
        let new_x = ray.x as i32 + offset[0];
        let new_y = ray.y as i32 + offset[1];
        let new_z = ray.z as i32 + offset[2];

        if terrain.is_oob(new_x, new_y, new_z) {
            raycast.is_adj_hit = false;
            return;
        }

        raycast.is_adj_hit = true;
        raycast.adj_pos = [new_x as u32, new_y as u32, new_z as u32];
    }
}
