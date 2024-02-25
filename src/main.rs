use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use block::meshing::chunk_material::ChunkMaterial;
use block::slice::slice::{SliceMaterial, TerrainSlice};
use block::world::block::Block;
use block::world::generator::TerrainGenerator;
use block::world::terrain::{Terrain, TerrainModifiedEvent};
use camera::{CameraPlugin, FlyCamera};
use debug::fps::FpsPlugin;

mod block;
mod camera;
mod debug;

fn main() {
    App::new()
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
        .add_plugins(TerrainGenerator)
        .add_plugins(WireframePlugin)
        .add_plugins(FpsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, camera_raycasting)
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
                    println!("remove block {},{},{}", rc.x, rc.y, rc.z);
                    // terrain.set_block(rc.x, rc.y, rc.z, Block::GRASS);
                    terrain.set_block(rc.x, rc.y, rc.z, Block::EMPTY);
                    ev_terrain_mod.send(TerrainModifiedEvent {
                        x: rc.x,
                        y: rc.y,
                        z: rc.z,
                        value: Block::EMPTY,
                    });
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
                        terrain.set_block(clamped_x, clamped_y, clamped_z, Block::STONE);
                        ev_terrain_mod.send(TerrainModifiedEvent {
                            x: clamped_x,
                            y: clamped_y,
                            z: clamped_z,
                            value: Block::EMPTY,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh: Handle<Mesh> = meshes.add(Cuboid::new(0.75, 0.75, 0.75));
    let cube_material = materials.add(Color::rgb_u8(124, 124, 124));

    commands.spawn((
        MaterialMeshBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Wireframe,
    ));

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
