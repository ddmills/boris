use std::collections::VecDeque;

use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_obj::ObjPlugin;
use colonists::{
    behavior_pick_system, behavior_system, block_move_system, fatigue_system, mine_job_checker,
    on_spawn_colonist, partition_debug, partition_setup, score_mine, score_wander, task_assign_job,
    task_check_has_item, task_debug, task_find_bed, task_find_nearest_item, task_get_job_location,
    task_idle, task_mine_block, task_move_to, task_pick_random_spot, task_pick_up_item, task_sleep,
    task_unassign_job, NavigationGraph, PartitionDebug, PartitionEvent, ScorerPlugin,
    SpawnColonistEvent,
};
use common::Rand;
use controls::{raycast, setup_camera, update_camera, Raycast};
use debug::{debug_settings::DebugSettings, fps::FpsPlugin, pathfinding::path_debug};
use items::{on_spawn_pickaxe, SpawnPickaxeEvent};
use terrain::*;
use ui::{
    setup_block_toolbar_ui, tool_system, toolbar_select, ui_capture_pointer, Tool, Toolbar, Ui,
};

mod colonists;
mod common;
mod controls;
mod debug;
mod items;
mod terrain;
mod ui;

fn main() {
    App::new()
        .insert_resource(Terrain::new(8, 4, 8, 16))
        .insert_resource(Rand::new())
        .insert_resource(DebugSettings::default())
        .insert_resource(Toolbar {
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
            hit_block: Block::EMPTY,
        })
        .add_event::<SpawnColonistEvent>()
        .add_event::<SpawnPickaxeEvent>()
        .add_event::<TerrainSliceChanged>()
        .add_event::<PartitionEvent>()
        .init_resource::<NavigationGraph>()
        .init_resource::<PartitionDebug>()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_plugins(ScorerPlugin)
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
        .add_systems(Update, on_spawn_pickaxe)
        .add_systems(Update, partition_debug)
        .add_systems(Update, mine_job_checker)
        .add_systems(Update, fatigue_system)
        .add_systems(Update, block_move_system)
        .add_systems(Update, behavior_pick_system)
        .add_systems(PreUpdate, behavior_system)
        .add_systems(Update, score_wander)
        .add_systems(Update, score_mine)
        .add_systems(Update, task_assign_job)
        .add_systems(Update, task_find_bed)
        .add_systems(Update, task_sleep)
        .add_systems(Update, task_idle)
        .add_systems(Update, task_pick_random_spot)
        .add_systems(Update, task_move_to)
        .add_systems(Update, task_get_job_location)
        .add_systems(Update, task_mine_block)
        .add_systems(Update, task_debug)
        .add_systems(Update, task_unassign_job)
        .add_systems(Update, task_check_has_item)
        .add_systems(Update, task_find_nearest_item)
        .add_systems(Update, task_pick_up_item)
        .run();
}

#[derive(Component, Default)]
struct Cursor {}

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = asset_server.load("meshes/cube.obj");
    let material = materials.add(Color::RED);

    cmd.spawn((
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
