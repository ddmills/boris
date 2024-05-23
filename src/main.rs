use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::{gltf::GltfPlugin, utils::hashbrown::HashMap};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_mod_picking::debug::DebugPickingMode;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_obj::ObjPlugin;
use colonists::{
    apply_falling, behavior_pick_system, behavior_system, block_move_system, check_job_build_valid,
    check_job_supply_valid, colonist_animations, destroy_items, fatigue_system, job_accessibility,
    job_despawn_cancelled, job_despawn_complete, on_cancel_job, on_spawn_colonist,
    on_spawn_job_build, on_spawn_job_chop, on_spawn_job_mine, on_spawn_job_place_block,
    on_spawn_job_supply, partition, partition_debug, score_build, score_chop, score_mine,
    score_place_block, score_supply, score_wander, task_animate, task_build, task_check_has_item,
    task_chop_tree, task_debug, task_find_bed, task_find_nearest_item, task_get_job_location,
    task_idle, task_is_target_empty, task_item_equip, task_item_pick_up, task_job_assign,
    task_job_cancel, task_job_complete, task_job_unassign, task_look_at, task_mine_block,
    task_move_to, task_pick_random_spot, task_place_block, task_sleep, task_supply, ActorRef,
    Blackboard, ColonistAnimations, DestroyItemEvent, HasBehavior, InInventory, Inventory, Item,
    ItemTag, JobCancelEvent, NavigationGraph, PartitionDebug, PartitionPathRequest, Path, Score,
    ScorerPlugin, Scorers, SpawnColonistEvent, SpawnJobBuildEvent, SpawnJobChopEvent,
    SpawnJobMineEvent, SpawnJobPlaceBlockEvent, SpawnJobSupplyEvent, TaskState,
};
use common::Rand;
use controls::{
    raycast, setup_camera, toggle_prepass_view, update_camera, PrepassOutputMaterial, Raycast,
};
use debug::{debug_settings::DebugSettings, fps::FpsPlugin, pathfinding::path_debug};
use items::{
    on_set_slot, on_spawn_axe, on_spawn_commodity, on_spawn_pickaxe,
    setup_commodity_stone_shale_boulder, setup_commodity_wood_birch_log, Commodities, SetSlotEvent,
    SpawnAxeEvent, SpawnCommodityEvent, SpawnPickaxeEvent,
};
use rendering::{
    setup_gltf_objects, update_basic_material_children_lighting, update_basic_material_lighting,
    BasicMaterial, ATTRIBUTE_SLOTS,
};
use structures::{
    check_structures, on_build_structure, on_remove_structure, on_spawn_structure,
    setup_blueprint_door, setup_blueprint_ladder, setup_blueprint_torches,
    setup_blueprint_workbench, setup_structure_torch, structure_material_update, Blueprints,
    BuildStructureEvent, BuiltStructureEvent, RemoveStructureEvent, SpawnStructureEvent,
};
use terrain::*;
use ui::{
    job_toolbar, on_inspectable_clicked, on_inspector_close, on_toolbar_submenu_btn,
    on_toolbar_tool_btn, setup_block_toolbar_ui, setup_inspectables, setup_inspector_ui,
    tool_block_info, tool_chop, tool_clear_block, tool_mine, tool_place_blocks, tool_place_stone,
    tool_spawn_axe, tool_spawn_colonist, tool_spawn_pickaxe, tool_spawn_structure,
    tool_toggle_path, ui_capture_pointer, update_inspector, GameSpeed, InspectableClickedEvent,
    Tool, Toolbar, Ui,
};

mod colonists;
mod common;
mod controls;
mod debug;
mod items;
mod rendering;
mod structures;
mod terrain;
mod ui;

fn main() {
    App::new()
        .insert_resource(Terrain::new(8, 3, 8, 16))
        .insert_resource(Rand::new())
        .insert_resource(DebugSettings::default())
        .insert_resource(Blueprints::default())
        .insert_resource(Commodities::default())
        .insert_resource(Toolbar {
            tool: Tool::PlaceBlocks(BlockType::STONE),
            submenu: None,
            submenus: HashMap::new(),
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
        .add_event::<SpawnJobPlaceBlockEvent>()
        .add_event::<SpawnJobMineEvent>()
        .add_event::<SpawnJobChopEvent>()
        .add_event::<SpawnJobSupplyEvent>()
        .add_event::<SpawnJobBuildEvent>()
        .add_event::<SpawnStructureEvent>()
        .add_event::<RemoveStructureEvent>()
        .add_event::<BuildStructureEvent>()
        .add_event::<BuiltStructureEvent>()
        .add_event::<TerrainSliceChangeEvent>()
        .add_event::<JobCancelEvent>()
        .add_event::<SpawnCommodityEvent>()
        .add_event::<SetSlotEvent>()
        .add_event::<InspectableClickedEvent>()
        .init_resource::<NavigationGraph>()
        .init_resource::<PartitionDebug>()
        .init_resource::<GameSpeed>()
        .init_resource::<Lamps>()
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins((
            DefaultPlugins
                .set(GltfPlugin::default().add_custom_vertex_attribute("SLOT", ATTRIBUTE_SLOTS))
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            ObjPlugin,
        ))
        .add_plugins(EguiPlugin)
        // .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(ScorerPlugin)
        .add_plugins(DefaultPickingPlugins.build())
        .add_plugins(MaterialPlugin::<ChunkMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(MaterialPlugin::<SliceMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(MaterialPlugin::<BasicMaterial> {
            prepass_enabled: true,
            ..default()
        })
        .add_plugins(MaterialPlugin::<PrepassOutputMaterial> {
            // This material only needs to read the prepass textures,
            // but the meshes using it should not contribute to the prepass render, so we can disable it.
            prepass_enabled: false,
            ..default()
        })
        .add_plugins(WireframePlugin)
        .add_plugins(FpsPlugin)
        .add_systems(
            Startup,
            (
                setup,
                setup_blueprint_ladder,
                setup_blueprint_torches,
                setup_blueprint_workbench,
                setup_blueprint_door,
                setup_commodity_wood_birch_log,
                setup_commodity_stone_shale_boulder,
                setup_terrain,
                setup_terrain_slice,
                setup_chunk_meshes,
                setup_camera,
                setup_inspector_ui,
                setup_block_toolbar_ui,
            )
                .chain(),
        )
        .add_systems(Update, setup_structure_torch)
        .add_systems(Update, (setup_gltf_objects, setup_inspectables).chain())
        .add_systems(Update, ui_capture_pointer)
        .add_systems(Update, draw_gizmos)
        .add_systems(Update, raycast)
        .add_systems(Update, scroll_events)
        .add_systems(Update, on_slice_changed)
        .add_systems(Update, on_remove_structure)
        .add_systems(Update, on_build_structure)
        .add_systems(Update, update_slice_mesh)
        .add_systems(Update, hide_sliced_objects)
        .add_systems(Update, on_removed_lamp)
        .add_systems(Update, light_system)
        .add_systems(Update, update_camera)
        .add_systems(Update, on_toolbar_tool_btn)
        .add_systems(Update, on_toolbar_submenu_btn)
        .add_systems(Update, (on_inspectable_clicked, update_inspector).chain())
        .add_systems(Update, on_inspector_close)
        .add_systems(Update, check_job_supply_valid)
        .add_systems(Update, check_job_build_valid)
        .add_systems(Update, job_toolbar)
        .add_systems(Update, path_debug)
        .add_systems(Update, on_spawn_commodity)
        .add_systems(Update, on_moved_lamp)
        .add_systems(Update, on_spawn_tree)
        .add_systems(Update, on_spawn_colonist)
        .add_systems(Update, on_spawn_pickaxe)
        .add_systems(Update, on_spawn_axe)
        .add_systems(Update, on_spawn_structure)
        .add_systems(Update, on_cancel_job)
        .add_systems(Update, apply_falling)
        .add_systems(Update, partition_debug)
        .add_systems(Update, job_accessibility)
        .add_systems(Update, fatigue_system)
        .add_systems(Update, toggle_prepass_view)
        .add_systems(Update, destroy_items)
        .add_systems(Update, block_move_system)
        .add_systems(PostUpdate, job_despawn_complete)
        .add_systems(PostUpdate, job_despawn_cancelled)
        .add_systems(PreUpdate, behavior_system)
        .add_systems(Update, on_spawn_job_place_block)
        .add_systems(Update, on_spawn_job_mine)
        .add_systems(Update, on_spawn_job_chop)
        .add_systems(Update, on_spawn_job_build)
        .add_systems(Update, on_spawn_job_supply)
        .add_systems(Update, behavior_pick_system)
        .add_systems(
            Update,
            (
                score_wander,
                score_mine,
                score_chop,
                score_place_block,
                score_build,
                score_supply,
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
                tool_spawn_structure,
                check_structures,
                structure_material_update,
            )
                .chain(),
        )
        .add_systems(Update, tool_spawn_axe)
        .add_systems(Update, tool_place_stone)
        .add_systems(Update, task_job_assign)
        .add_systems(Update, (task_supply, on_set_slot).chain())
        .add_systems(Update, task_find_bed)
        .add_systems(Update, task_sleep)
        .add_systems(Update, task_idle)
        .add_systems(Update, task_pick_random_spot)
        .add_systems(Update, task_move_to)
        .add_systems(Update, task_look_at)
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
        .add_systems(Update, update_basic_material_lighting)
        .add_systems(Update, update_basic_material_children_lighting)
        .add_systems(
            PostUpdate,
            (chunk_meshing, partition, update_positions).chain(),
        )
        .insert_resource(Msaa::Off)
        .run();
}

#[derive(Component, Default)]
struct Cursor {}

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BasicMaterial>>,
) {
    cmd.insert_resource(ColonistAnimations {
        swing_axe: asset_server.load("human.gltf#Animation0"),
        base: asset_server.load("human.gltf#Animation1"),
        swing_hammer: asset_server.load("human.gltf#Animation2"),
        idle: asset_server.load("human.gltf#Animation3"),
        swing_pick: asset_server.load("human.gltf#Animation4"),
        pick_up: asset_server.load("human.gltf#Animation5"),
        run: asset_server.load("human.gltf#Animation6"),
    });

    cmd.spawn((
        MaterialMeshBundle {
            mesh: asset_server.load("cube_offset.gltf#Mesh0/Primitive0"),
            material: materials.add(BasicMaterial {
                color: Color::YELLOW,
                is_lit: false,
                ..Default::default()
            }),
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
