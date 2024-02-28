use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use block::light::light_system;
use block::meshing::chunk_material::ChunkMaterial;
use block::meshing::chunk_meshing::{on_slice_changed, process_dirty_chunks, setup_chunk_meshes};
use block::slice::slice::{
    scroll_events, setup_terrain_slice, update_slice_mesh, SliceMaterial, TerrainSlice,
    TerrainSliceChanged,
};
use block::world::block::Block;
use block::world::generator::setup_terrain;
use block::world::terrain::{Terrain, TerrainModifiedEvent};
use camera::{CameraPlugin, FlyCamera};
use debug::fps::FpsPlugin;

mod block;
mod camera;
mod common;
mod debug;

fn main() {
    App::new()
        .insert_resource(Terrain::new(6, 6, 6, 16))
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
        .add_plugins(CameraPlugin)
        .add_plugins(WireframePlugin)
        .add_plugins(FpsPlugin)
        .add_systems(
            Startup,
            (
                setup,
                setup_terrain,
                setup_terrain_slice,
                setup_chunk_meshes,
            )
                .chain(),
        )
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, camera_raycasting)
        .add_systems(Update, scroll_events)
        .add_systems(Update, process_dirty_chunks)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, light_system)
        .run();
}

fn camera_raycasting(
    mut cameras: Query<&Transform, With<FlyCamera>>,
    mut terrain: ResMut<Terrain>,
    terrain_slice: Res<TerrainSlice>,
    mut click_evt: EventReader<MouseButtonInput>,
    mut ev_terrain_mod: EventWriter<TerrainModifiedEvent>,
) {
    let slice_y = terrain_slice.get_value();

    for transform in cameras.iter_mut() {
        let dir = transform.forward();
        let origin = transform.translation;
        let radius = 20;

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
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-10., 10., -10.)
                .looking_at(Vec3::new(0., 5., 0.), Vec3::Y),
            ..default()
        },
        FlyCamera,
    ));
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X * 100., Color::RED);
    gizmos.line(Vec3::ZERO, Vec3::Y * 100., Color::GREEN);
    gizmos.line(Vec3::ZERO, Vec3::Z * 100., Color::BLUE);
}
