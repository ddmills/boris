use bevy::input::ButtonState;
use bevy::math::vec3;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::{input::mouse::MouseButtonInput, window::PrimaryWindow};
use controls::{spawn_camera, update_camera, MainCamera};
use debug::fps::FpsPlugin;
use terrain::*;

mod common;
mod controls;
mod debug;
mod terrain;

fn main() {
    App::new()
        .insert_resource(Terrain::new(6, 4, 6, 16))
        .add_event::<TerrainSliceChanged>()
        .add_event::<TerrainModifiedEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ChunkMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(MaterialPlugin::<SliceMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(WireframePlugin)
        .add_plugins(FpsPlugin)
        .add_systems(
            Startup,
            (
                setup,
                setup_terrain,
                setup_terrain_slice,
                setup_chunk_meshes,
                spawn_camera,
            )
                .chain(),
        )
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, cursor_raycasting)
        .add_systems(Update, scroll_events)
        .add_systems(Update, process_dirty_chunks)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .run();
}

fn cursor_raycasting(
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut terrain: ResMut<Terrain>,
    terrain_slice: Res<TerrainSlice>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut click_evt: EventReader<MouseButtonInput>,
    mut ev_terrain_mod: EventWriter<TerrainModifiedEvent>,
) {
    let window = windows.single();
    let (camera, transform) = cameras.single();
    let slice_y = terrain_slice.get_value();

    // let mut ray: Ray3d = Ray3d::new(vec3(x, y, z), direction)
    let mut origin = vec3(0., 0., 0.);
    let mut dir = vec3(0., 0., 0.);

    if let Some(cursor) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(transform, cursor) {
            origin = ray.origin;
            dir = vec3(ray.direction.x, ray.direction.y, ray.direction.z);
        } else {
            return;
        }
    } else {
        return;
    }

    // let origin = transform.translation;
    let radius = 200;

    let rc = terrain.raycast(
        origin.x, origin.y, origin.z, dir.x, dir.y, dir.z, slice_y, radius,
    );

    if !rc.is_hit {
        click_evt.clear();
    }

    for ev in click_evt.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }

        match ev.button {
            MouseButton::Right => {
                println!(
                    "remove block {},{},{},{}",
                    rc.x,
                    rc.y,
                    rc.z,
                    rc.block.name()
                );
                // terrain.set_block(rc.x, rc.y, rc.z, Block::GRASS);
                terrain.set_block(rc.x, rc.y, rc.z, Block::EMPTY);
            }
            MouseButton::Left => {
                println!(
                    "place block {},{},{}. face={}",
                    rc.x,
                    rc.y,
                    rc.z,
                    rc.face.bit()
                );
                let offset = rc.face.offset();
                let new_x = rc.x as i32 + offset[0];
                let new_y = rc.y as i32 + offset[1];
                let new_z = rc.z as i32 + offset[2];

                if !terrain.is_oob(new_x, new_y, new_z) {
                    let clamped_x = new_x as u32;
                    let clamped_y = new_y as u32;
                    let clamped_z = new_z as u32;

                    terrain.set_block(clamped_x, clamped_y, clamped_z, Block::LAMP);

                    ev_terrain_mod.send(TerrainModifiedEvent {
                        x: clamped_x,
                        y: clamped_y,
                        z: clamped_z,
                    });
                }
            }
            _ => {}
        }
    }
}

fn setup(mut commands: Commands) {}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X * 256., Color::RED);
    gizmos.line(Vec3::ZERO, Vec3::Y * 256., Color::GREEN);
    gizmos.line(Vec3::ZERO, Vec3::Z * 256., Color::BLUE);
}
