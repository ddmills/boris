use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use controls::{raycast, setup_camera, update_camera, Raycast};
use debug::fps::FpsPlugin;
use terrain::*;
use ui::{setup_block_toolbar_ui, toolbar_select, ui_capture_pointer, Tool, Toolbar, Ui};

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
            tool: Tool::PlaceBlocks(Block::STONE),
        })
        .insert_resource(Ui {
            pointer_captured: false,
        })
        .insert_resource(Raycast {
            is_hit: false,
            hit_pos: [0, 0, 0],
            is_adj_hit: false,
            adj_pos: [0, 0, 0],
        })
        .add_event::<TerrainSliceChanged>()
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
        .add_systems(Update, raycast)
        .add_systems(Update, testing)
        .add_systems(Update, scroll_events)
        .add_systems(Update, process_dirty_chunks)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .add_systems(Update, toolbar_select)
        .run();
}

fn testing(
    mut terrain: ResMut<Terrain>,
    mut click_evt: EventReader<MouseButtonInput>,
    selected_block: Res<Toolbar>,
    raycast: Res<Raycast>,
) {
    for ev in click_evt.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }

        match ev.button {
            MouseButton::Right => {
                if !raycast.is_hit {
                    return;
                }

                println!(
                    "remove block {},{},{}",
                    raycast.hit_pos[0], raycast.hit_pos[1], raycast.hit_pos[2],
                );

                terrain.set_block(
                    raycast.hit_pos[0],
                    raycast.hit_pos[1],
                    raycast.hit_pos[2],
                    Block::EMPTY,
                );
            }
            MouseButton::Left => {
                if !raycast.is_adj_hit {
                    return;
                }

                println!(
                    "add block {},{},{}",
                    raycast.adj_pos[0], raycast.adj_pos[1], raycast.adj_pos[2],
                );

                terrain.set_block(
                    raycast.adj_pos[0],
                    raycast.adj_pos[1],
                    raycast.adj_pos[2],
                    selected_block.block,
                );
            }
            _ => {}
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
