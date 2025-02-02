use bevy::{asset::AssetServer, ecs::system::Res};

use crate::{
    colonists::{ItemTag, NavigationFlags},
    structures::{
        Blueprint, BlueprintHotspot, BlueprintTile, BlueprintType, Blueprints, BuildSlot,
        BuildSlots, DirectionSimple, TileRequirement,
    },
};

use bevy::ecs::system::ResMut;

pub fn setup_blueprint_workbench(
    mut blueprints: ResMut<Blueprints>,
    asset_server: Res<AssetServer>,
) {
    blueprints.0.insert(
        BlueprintType::Workbench,
        Blueprint {
            name: "Workbench".to_string(),
            slots: BuildSlots {
                slot_0: Some(BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                }),
                slot_1: Some(BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                }),
                slot_2: Some(BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                }),
            },
            center: [0, 0, 0],
            tiles: vec![
                BlueprintTile {
                    position: [0, 0, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_WALKABLE | TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [1, 0, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_WALKABLE | TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [0, 0, -1],
                    hotspot: Some(BlueprintHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                BlueprintTile {
                    position: [1, 0, -1],
                    hotspot: Some(BlueprintHotspot {
                        is_optional: false,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                BlueprintTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [1, 1, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
            ],
            texture: None,
            mesh: asset_server.load("workbench.gltf#Mesh0/Primitive0"),
        },
    );
}
