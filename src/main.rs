use std::collections::VecDeque;

use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_obj::ObjPlugin;
use colonists::{
    behavior_pick_system, behavior_system, block_move_system, fatigue_system, on_spawn_colonist,
    partition, partition_debug, partition_setup, path_debug, task_debug, task_find_bed,
    task_get_job_location, task_idle, task_mine_block, task_move_to, task_pick_random_spot,
    task_return_job, task_set_job, task_sleep, JobList, PartitionDebug, PartitionEvent,
    PartitionGraph, SpawnColonistEvent,
};
use common::Rand;
use controls::{raycast, setup_camera, update_camera, Raycast};
use debug::fps::FpsPlugin;
use terrain::*;
use ui::{
    setup_block_toolbar_ui, tool_system, toolbar_select, ui_capture_pointer, Tool, Toolbar, Ui,
};

mod colonists;
mod common;
mod controls;
mod debug;
mod terrain;
mod ui;

fn main() {
    App::new()
        .insert_resource(Terrain::new(8, 4, 8, 16))
        .insert_resource(Rand::new())
        .insert_resource(Toolbar {
            tool: Tool::PlaceBlocks(Block::STONE),
        })
        .insert_resource(JobList {
            jobs: VecDeque::new(),
        })
        .insert_resource(Ui {
            pointer_captured: false,
        })
        .insert_resource(Raycast {
            is_hit: false,
            hit_pos: [0, 0, 0],
            is_adj_hit: false,
            adj_pos: [0, 0, 0],
            hit_block: Block::EMPTY,
        })
        .add_event::<SpawnColonistEvent>()
        .add_event::<TerrainSliceChanged>()
        .add_event::<PartitionEvent>()
        .init_resource::<PartitionGraph>()
        .init_resource::<PartitionDebug>()
        .add_plugins((DefaultPlugins, ObjPlugin))
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
                partition_setup,
                setup_camera,
                setup_block_toolbar_ui,
            )
                .chain(),
        )
        .add_systems(Update, ui_capture_pointer)
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, raycast)
        .add_systems(Update, scroll_events)
        .add_systems(Update, process_dirty_chunks)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .add_systems(Update, toolbar_select)
        .add_systems(Update, path_debug)
        .add_systems(Update, tool_system)
        .add_systems(Update, on_spawn_colonist)
        .add_systems(Update, partition_debug)
        .add_systems(Update, partition)
        .add_systems(Update, fatigue_system)
        .add_systems(Update, block_move_system)
        .add_systems(
            Update,
            (
                behavior_pick_system,
                behavior_system,
                task_set_job,
                task_find_bed,
                task_sleep,
                task_idle,
                task_pick_random_spot,
                task_move_to,
                task_get_job_location,
                task_mine_block,
                task_debug,
                task_return_job,
            )
                .chain(),
        )
        .run();
}

#[derive(Component, Default)]
struct Cursor {}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = asset_server.load("meshes/cube.obj");
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
