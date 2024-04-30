use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_obj::ObjPlugin;
use colonists::{
    apply_falling, behavior_pick_system, behavior_system, block_move_system, colonist_animations,
    destroy_items, fatigue_system, job_accessibility, job_despawn_cancelled, job_despawn_complete,
    on_spawn_colonist, on_spawn_job_build, on_spawn_job_chop, on_spawn_job_mine,
    on_spawn_job_place_block, partition, partition_debug, score_build, score_chop, score_mine,
    score_place_block, score_wander, setup_colonists, task_animate, task_build,
    task_check_has_item, task_chop_tree, task_debug, task_find_bed, task_find_nearest_item,
    task_get_job_location, task_idle, task_is_target_empty, task_item_equip, task_item_pick_up,
    task_job_assign, task_job_cancel, task_job_complete, task_job_unassign, task_mine_block,
    task_move_to, task_pick_random_spot, task_place_block, task_sleep, ActorRef, Blackboard,
    ColonistAnimations, DestroyItemEvent, HasBehavior, InInventory, Inventory, Item, ItemTag,
    NavigationGraph, PartitionDebug, PartitionPathRequest, Path, Score, ScorerPlugin, Scorers,
    SpawnColonistEvent, SpawnJobBuildEvent, SpawnJobChopEvent, SpawnJobMineEvent,
    SpawnJobPlaceBlockEvent, TaskState,
};
use common::Rand;
use controls::{raycast, setup_camera, update_camera, Raycast};
use debug::{debug_settings::DebugSettings, fps::FpsPlugin, pathfinding::path_debug};
use furniture::{
    blueprint_material_update, check_blueprints, on_remove_blueprint, on_spawn_blueprint,
    setup_templates, RemoveBlueprintEvent, SpawnBlueprintEvent,
};
use items::{
    on_spawn_axe, on_spawn_pickaxe, on_spawn_stone, SpawnAxeEvent, SpawnPickaxeEvent,
    SpawnStoneEvent,
};
use rendering::{
    update_basic_material_children_lighting, update_basic_material_lighting, BasicMaterial,
};
use terrain::*;
use ui::{
    job_toolbar, setup_block_toolbar_ui, tool_block_info, tool_chop, tool_clear_block, tool_mine,
    tool_place_blocks, tool_place_stone, tool_spawn_axe, tool_spawn_colonist, tool_spawn_pickaxe,
    tool_spawn_template, tool_toggle_path, toolbar_select, ui_capture_pointer, GameSpeed, Tool,
    Toolbar, Ui,
};

mod colonists;
mod common;
mod controls;
mod debug;
mod furniture;
mod items;
mod rendering;
mod terrain;
mod ui;

fn main() {
    App::new()
        .insert_resource(Terrain::new(6, 4, 6, 16))
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
        .register_type::<Position>()
        .register_type::<HasBehavior>()
        .register_type::<ActorRef>()
        .register_type::<Path>()
        .register_type::<PartitionPathRequest>()
        .register_type::<Score>()
        .register_type::<Scorers>()
        .register_type::<Inventory>()
        .register_type::<Item>()
        .register_type::<InInventory>()
        .register_type::<ItemTag>()
        .register_type::<Blackboard>()
        .register_type::<TaskState>()
        .add_event::<SpawnTreeEvent>()
        .add_event::<SpawnColonistEvent>()
        .add_event::<SpawnAxeEvent>()
        .add_event::<SpawnPickaxeEvent>()
        .add_event::<DestroyItemEvent>()
        .add_event::<SpawnStoneEvent>()
        .add_event::<SpawnJobPlaceBlockEvent>()
        .add_event::<SpawnJobMineEvent>()
        .add_event::<SpawnJobChopEvent>()
        .add_event::<SpawnJobBuildEvent>()
        .add_event::<SpawnBlueprintEvent>()
        .add_event::<RemoveBlueprintEvent>()
        .add_event::<TerrainSliceChangeEvent>()
        .init_resource::<NavigationGraph>()
        .init_resource::<PartitionDebug>()
        .init_resource::<GameSpeed>()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_plugins(EguiPlugin)
        // .add_plugins(WorldInspectorPlugin::default())
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
                setup_templates,
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
        .add_systems(Update, on_remove_blueprint)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, hide_sliced_objects)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .add_systems(Update, toolbar_select)
        .add_systems(Update, job_toolbar)
        .add_systems(Update, path_debug)
        .add_systems(Update, on_spawn_tree)
        .add_systems(Update, on_spawn_colonist)
        .add_systems(Update, on_spawn_pickaxe)
        .add_systems(Update, on_spawn_axe)
        .add_systems(Update, on_spawn_stone)
        .add_systems(Update, on_spawn_blueprint)
        .add_systems(Update, apply_falling)
        .add_systems(Update, partition_debug)
        .add_systems(Update, job_accessibility)
        .add_systems(Update, fatigue_system)
        .add_systems(Update, destroy_items)
        .add_systems(Update, block_move_system)
        .add_systems(PreUpdate, job_despawn_complete)
        .add_systems(PreUpdate, job_despawn_cancelled)
        .add_systems(PreUpdate, behavior_system)
        .add_systems(Update, on_spawn_job_place_block)
        .add_systems(Update, on_spawn_job_mine)
        .add_systems(Update, on_spawn_job_chop)
        .add_systems(Update, on_spawn_job_build)
        .add_systems(Update, behavior_pick_system)
        .add_systems(
            Update,
            (
                score_wander,
                score_mine,
                score_chop,
                score_place_block,
                score_build,
            )
                .before(behavior_pick_system),
        )
        .add_systems(Update, tool_place_blocks)
        .add_systems(Update, tool_clear_block)
        .add_systems(Update, tool_spawn_colonist)
        .add_systems(Update, tool_block_info)
        .add_systems(Update, tool_mine)
        .add_systems(Update, tool_chop)
        .add_systems(Update, tool_toggle_path)
        .add_systems(Update, tool_spawn_pickaxe)
        .add_systems(
            Update,
            (
                tool_spawn_template,
                check_blueprints,
                blueprint_material_update,
            )
                .chain(),
        )
        .add_systems(Update, tool_spawn_axe)
        .add_systems(Update, tool_place_stone)
        .add_systems(Update, task_job_assign)
        .add_systems(Update, task_find_bed)
        .add_systems(Update, task_sleep)
        .add_systems(Update, task_idle)
        .add_systems(Update, task_pick_random_spot)
        .add_systems(Update, task_move_to)
        .add_systems(Update, task_chop_tree)
        .add_systems(Update, task_build)
        .add_systems(Update, task_get_job_location)
        .add_systems(Update, task_mine_block)
        .add_systems(Update, task_place_block)
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
        .add_systems(Update, update_basic_material_lighting)
        .add_systems(Update, update_basic_material_children_lighting)
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
    mut materials: ResMut<Assets<BasicMaterial>>,
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

    let mesh = asset_server.load("cube_offset.gltf#Mesh0/Primitive0");
    let material = materials.add(BasicMaterial {
        color: Color::YELLOW,
        texture: None,
        sunlight: 15,
        torchlight: 15,
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
