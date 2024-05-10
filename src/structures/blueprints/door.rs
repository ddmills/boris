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
};

use bevy::ecs::system::ResMut;

pub fn setup_blueprint_door(mut blueprints: ResMut<Blueprints>, asset_server: Res<AssetServer>) {
    blueprints.0.insert(
        BlueprintType::Door,
        Blueprint {
            name: "Door".to_string(),
            slots: BuildSlots {
                slot_0: Some(BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                }),
                slot_1: Some(BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                }),
                slot_2: None,
            },
            center: [0, 0, 0],
            tiles: vec![
                BlueprintTile {
                    position: [0, 0, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::DOOR,
                    is_blocker: true,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TileRequirement::IS_EMPTY,
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
            ],
            texture: None,
            mesh: asset_server.load("door.gltf#Mesh0/Primitive0"),
        },
    );
}

pub fn setup_structure_door(
    mut cmd: Commands,
    mut ev_built_structure: EventReader<BuiltStructureEvent>,
) {
    for ev in ev_built_structure.read() {
        if !matches!(ev.blueprint_type, BlueprintType::Door) {
            continue;
        }

        println!("Door spawned");

        // let mut ecmd = cmd.entity(ev.entity);

        // ecmd.try_insert(GltfBinding {
        //     armature_name: "Door_Armature".into(),
        //     mesh_name: "Door_01".into(),
        //     texture_path: None,
        // });
    }
}
