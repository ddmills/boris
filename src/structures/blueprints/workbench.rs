use bevy::{
    asset::{AssetLoader, AssetServer, Handle},
    ecs::system::Res,
    gltf::{GltfLoader, GltfLoaderSettings},
    render::{mesh::Mesh, texture::Image},
};

use crate::{
    colonists::{ItemTag, NavigationFlags},
    items::image_loader_settings,
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
    let stone_texture: Handle<Image> =
        asset_server.load_with_settings("textures/stone.png", image_loader_settings);
    let mesh: Handle<Mesh> = asset_server.load("workbench.gltf#Mesh0/Primitive0");

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
            texture: stone_texture.clone(),
            mesh,
        },
    );
}
