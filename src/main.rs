use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_obj::ObjPlugin;
use colonists::{
    apply_falling, behavior_pick_system, behavior_system, block_move_system, colonist_animations,
    destroy_items, fatigue_system, job_accessibility, job_despawn_cancelled, job_despawn_complete,
    on_spawn_colonist, on_spawn_job_build, on_spawn_job_mine, partition, partition_debug,
    score_build, score_mine, score_wander, setup_colonists, task_animate, task_build_block,
    task_check_has_item, task_debug, task_find_bed, task_find_nearest_item, task_get_job_location,
    task_idle, task_is_target_empty, task_item_equip, task_item_pick_up, task_job_assign,
    task_job_cancel, task_job_complete, task_job_unassign, task_mine_block, task_move_to,
    task_pick_random_spot, task_sleep, ColonistAnimations, DestroyItemEvent, NavigationGraph,
    PartitionDebug, ScorerPlugin, SpawnColonistEvent, SpawnJobBuildEvent, SpawnJobMineEvent,
};
use common::Rand;
use controls::{raycast, setup_camera, update_camera, Raycast};
use debug::{debug_settings::DebugSettings, fps::FpsPlugin, pathfinding::path_debug};
use items::{on_spawn_pickaxe, on_spawn_stone, SpawnPickaxeEvent, SpawnStoneEvent};
use rendering::BasicMaterial;
use terrain::*;
use ui::{
    setup_block_toolbar_ui, tool_system, toolbar_select, ui_capture_pointer, Tool, Toolbar, Ui,
};

mod colonists;
mod common;
mod controls;
mod debug;
mod items;
mod rendering;
mod terrain;
mod ui;

fn main() {
    App::new()
        .insert_resource(Terrain::new(6, 2, 6, 16))
        .insert_resource(Rand::new())
        .insert_resource(DebugSettings::default())
        .insert_resource(Toolbar {
            tool: Tool::PlaceBlocks(BlockType::STONE),
        })
        .insert_resource(Ui {
            pointer_captured: false,
        })
        .insert_resource(Raycast {
            is_hit: false,
            hit_pos: [0, 0, 0],
            is_adj_hit: false,
            adj_pos: [0, 0, 0],
            hit_block: Block::OOB,
        })
        .add_event::<SpawnColonistEvent>()
        .add_event::<SpawnPickaxeEvent>()
        .add_event::<DestroyItemEvent>()
        .add_event::<SpawnStoneEvent>()
        .add_event::<SpawnJobBuildEvent>()
        .add_event::<SpawnJobMineEvent>()
        .add_event::<TerrainSliceChanged>()
        .init_resource::<NavigationGraph>()
        .init_resource::<PartitionDebug>()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(ScorerPlugin)
        .add_plugins(MaterialPlugin::<ChunkMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(MaterialPlugin::<SliceMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(MaterialPlugin::<BasicMaterial> {
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
                setup_camera,
                setup_block_toolbar_ui,
            )
                .chain(),
        )
        .add_systems(Update, ui_capture_pointer)
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, raycast)
        .add_systems(Update, scroll_events)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .add_systems(Update, toolbar_select)
        .add_systems(Update, path_debug)
        .add_systems(Update, tool_system)
        .add_systems(Update, on_spawn_colonist)
        .add_systems(Update, on_spawn_pickaxe)
        .add_systems(Update, on_spawn_stone)
        .add_systems(Update, apply_falling)
        .add_systems(Update, partition_debug)
        .add_systems(Update, job_accessibility)
        .add_systems(Update, fatigue_system)
        .add_systems(Update, destroy_items)
        .add_systems(Update, block_move_system)
        .add_systems(PreUpdate, job_despawn_complete)
        .add_systems(PreUpdate, job_despawn_cancelled)
        .add_systems(PreUpdate, behavior_system)
        .add_systems(Update, on_spawn_job_build)
        .add_systems(Update, on_spawn_job_mine)
        .add_systems(Update, behavior_pick_system)
        .add_systems(
            Update,
            (score_wander, score_mine, score_build).before(behavior_pick_system),
        )
        .add_systems(Update, task_job_assign)
        .add_systems(Update, task_find_bed)
        .add_systems(Update, task_sleep)
        .add_systems(Update, task_idle)
        .add_systems(Update, task_pick_random_spot)
        .add_systems(Update, task_move_to)
        .add_systems(Update, task_get_job_location)
        .add_systems(Update, task_mine_block)
        .add_systems(Update, task_build_block)
        .add_systems(Update, task_debug)
        .add_systems(Update, task_job_unassign)
        .add_systems(Update, task_job_cancel)
        .add_systems(Update, task_job_complete)
        .add_systems(Update, task_check_has_item)
        .add_systems(Update, task_find_nearest_item)
        .add_systems(Update, task_item_pick_up)
        .add_systems(Update, task_item_equip)
        .add_systems(Update, task_is_target_empty)
        .add_systems(Update, task_animate)
        .add_systems(Update, colonist_animations)
        .add_systems(Update, setup_colonists)
        .add_systems(
            PostUpdate,
            (chunk_meshing, partition, update_positions).chain(),
        )
        .run();
}

#[derive(Component, Default)]
struct Cursor {}

#[derive(Resource)]
struct HumanGltf(Handle<Scene>);

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gltf = asset_server.load("human.gltf#Scene0");
    cmd.insert_resource(HumanGltf(gltf));

    cmd.insert_resource(ColonistAnimations {
        base: asset_server.load("human.gltf#Animation0"),
        idle: asset_server.load("human.gltf#Animation1"),
        swing_pick: asset_server.load("human.gltf#Animation2"),
        pick_up: asset_server.load("human.gltf#Animation3"),
        run: asset_server.load("human.gltf#Animation4"),
    });

    let mesh = asset_server.load("meshes/cube_offcenter.obj");
    let material = materials.add(StandardMaterial {
        base_color: Color::YELLOW,
        unlit: true,
        depth_bias: 10.,
        ..default()
    });

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
