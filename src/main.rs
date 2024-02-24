use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use block::meshing::chunk_material::ChunkMaterial;
use block::slice::slice::SliceMaterial;
use block::world::generator::TerrainGenerator;
use camera::{CameraPlugin, FlyCamera};

mod block;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ChunkMaterial>::default())
        .add_plugins(MaterialPlugin::<SliceMaterial>::default())
        .add_plugins(CameraPlugin)
        .add_plugins(TerrainGenerator)
        .add_plugins(WireframePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gizmos)
        .run();
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
