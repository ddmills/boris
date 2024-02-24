use bevy::input::mouse::MouseButtonInput;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use block::meshing::chunk_material::ChunkMaterial;
use block::slice::slice::{SliceMaterial, TerrainSlice};
use block::world::generator::TerrainGenerator;
use block::world::terrain::Terrain;
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
        .add_systems(Update, camera_raycasting)
        .run();
}

fn camera_raycasting(
    mut cameras: Query<&Transform, With<FlyCamera>>,
    terrain: Res<Terrain>,
    terrain_slice: Res<TerrainSlice>,
) {
    let slice_y = terrain_slice.get_value();

    for transform in cameras.iter_mut() {
        let dir = transform.forward();
        let origin = transform.translation;
        let radius = 20;

        let rc = terrain.raycast(
            origin.x, origin.y, origin.z, dir.x, dir.y, dir.z, slice_y, radius,
        );

        if rc.is_hit {
            println!(
                "Found block {},{},{}={} in {} attempts",
                rc.x,
                rc.y,
                rc.z,
                rc.block.name(),
                rc.attempts
            );
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
