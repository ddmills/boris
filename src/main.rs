use bevy::input::ButtonState;
use bevy::math::vec3;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::{input::mouse::MouseButtonInput, window::PrimaryWindow};
use controls::{setup_camera, update_camera, MainCamera};
use debug::fps::FpsPlugin;
use terrain::*;
use ui::{setup_block_toolbar_ui, toolbar_select, ui_capture_pointer, Toolbar, Ui};

mod common;
mod controls;
mod debug;
mod terrain;
mod ui;

fn main() {
    App::new()
        .insert_resource(Terrain::new(8, 4, 8, 16))
        .insert_resource(Toolbar {
            block: Block::STONE,
        })
        .insert_resource(Ui {
            pointer_captured: false,
        })
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
                setup_chunks,
                setup_camera,
                setup_block_toolbar_ui,
            )
                .chain(),
        )
        .add_systems(Update, ui_capture_pointer)
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, cursor_raycasting)
        .add_systems(Update, scroll_events)
        .add_systems(Update, process_dirty_chunks)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .add_systems(Update, toolbar_select)
        .run();
}

fn cursor_raycasting(
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut terrain: ResMut<Terrain>,
    terrain_slice: Res<TerrainSlice>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut click_evt: EventReader<MouseButtonInput>,
    mut ev_terrain_mod: EventWriter<TerrainModifiedEvent>,
    mut cursors: Query<&mut Transform, With<Cursor>>,
    selected_block: Res<Toolbar>,
    ui: Res<Ui>,
) {
    if ui.pointer_captured {
        return;
    }

    for window in windows.iter() {
        let mut cursor = cursors.single_mut();
        let (camera, transform) = cameras.single();

        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };

        let Some(ray) = camera.viewport_to_world(transform, cursor_pos) else {
            return;
        };

        let origin = ray.origin;
        let dir = vec3(ray.direction.x, ray.direction.y, ray.direction.z);

        let slice_y = terrain_slice.get_value();
        let radius = 200;

        let rc = terrain.raycast(
            origin.x, origin.y, origin.z, dir.x, dir.y, dir.z, slice_y, radius,
        );

        if !rc.is_hit {
            click_evt.clear();
        }

        let offset = rc.face.offset();

        let new_x = (rc.x as i32 + offset[0]) as i32;
        let new_y = (rc.y as i32 + offset[1]) as i32;
        let new_z = (rc.z as i32 + offset[2]) as i32;

        if !terrain.is_oob(new_x, new_y, new_z) {
            cursor.translation =
                Vec3::new(new_x as f32 + 0.5, new_y as f32 + 0.5, new_z as f32 + 0.5);
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

                    if !terrain.is_oob(new_x, new_y, new_z) {
                        let clamped_x = new_x as u32;
                        let clamped_y = new_y as u32;
                        let clamped_z = new_z as u32;

                        terrain.set_block(clamped_x, clamped_y, clamped_z, selected_block.block);

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
}

#[derive(Component, Default)]
struct Cursor {}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1., 1., 1.));
    let material = materials.add(Color::RED);

    commands.spawn((
        MaterialMeshBundle {
            mesh,
            material,
            ..default()
        },
        Cursor::default(),
    ));
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X * 256., Color::RED);
    gizmos.line(Vec3::ZERO, Vec3::Y * 256., Color::GREEN);
    gizmos.line(Vec3::ZERO, Vec3::Z * 256., Color::BLUE);
}
