use bevy::{
    asset::AssetServer,
    ecs::{
        event::EventReader,
        system::{Commands, Res},
    },
};

use crate::{
    colonists::{ItemTag, NavigationFlags},
    structures::{
        Blueprint, BlueprintHotspot, BlueprintTile, BlueprintType, Blueprints, BuildSlot,
        BuildSlots, BuiltStructureEvent, DirectionSimple, TileRequirement,
    },
    Lamp,
};

use bevy::ecs::system::ResMut;

pub fn setup_blueprint_torches(mut blueprints: ResMut<Blueprints>, asset_server: Res<AssetServer>) {
    blueprints.0.insert(
        BlueprintType::TorchWall,
        Blueprint {
            name: "Wall torch".to_string(),
            slots: BuildSlots {
                slot_0: Some(BuildSlot {
                    flags: vec![ItemTag::Log],
                }),
                slot_1: Some(BuildSlot {
                    flags: vec![ItemTag::Log],
                }),
                slot_2: None,
            },
            center: [0, 0, 0],
            tiles: vec![
                BlueprintTile {
                    position: [0, 0, 0],
                    hotspot: Some(BlueprintHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [0, -1, 0],
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
                    position: [0, 0, 1],
                    hotspot: None,
                    requirements: TileRequirement::IS_ATTACHABLE,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
            ],
            texture: None,
            mesh: asset_server.load("torch_wall.gltf#Mesh0/Primitive0"),
        },
    );

    blueprints.0.insert(
        BlueprintType::TorchStanding,
        Blueprint {
            name: "Standing torch".to_string(),
            slots: BuildSlots {
                slot_0: Some(BuildSlot {
                    flags: vec![ItemTag::Log],
                }),
                slot_1: None,
                slot_2: None,
            },
            center: [0, 0, 0],
            tiles: vec![
                BlueprintTile {
                    position: [-1, 0, 0],
                    hotspot: Some(BlueprintHotspot {
                        is_optional: true,
                        direction: DirectionSimple::East,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                BlueprintTile {
                    position: [1, 0, 0],
                    hotspot: Some(BlueprintHotspot {
                        is_optional: true,
                        direction: DirectionSimple::West,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                BlueprintTile {
                    position: [0, 0, 1],
                    hotspot: Some(BlueprintHotspot {
                        is_optional: true,
                        direction: DirectionSimple::South,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
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
                    position: [0, 0, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_EMPTY | TileRequirement::IS_WALKABLE,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
            ],
            texture: None,
            mesh: asset_server.load("torch_standing.gltf#Mesh0/Primitive0"),
        },
    );
}

pub fn setup_structure_torch(
    mut cmd: Commands,
    mut ev_built_structure: EventReader<BuiltStructureEvent>,
) {
    for ev in ev_built_structure.read() {
        if matches!(ev.blueprint_type, BlueprintType::TorchStanding) {
            cmd.entity(ev.entity).insert(Lamp {
                value: 12,
                offset: [0, 1, 0],
            });
        } else if matches!(ev.blueprint_type, BlueprintType::TorchWall) {
            cmd.entity(ev.entity).insert(Lamp {
                value: 12,
                offset: [0, 0, 0],
            });
        }
    }
}
