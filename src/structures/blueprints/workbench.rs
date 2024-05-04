use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::Res,
    render::texture::Image,
};

use crate::{
    colonists::{ItemTag, NavigationFlags},
    items::{image_loader_settings, CommodityFlag},
    structures::{
        Blueprint, BlueprintHotspot, BlueprintTile, BlueprintType, Blueprints, BuildSlot,
        DirectionSimple, TileRequirement,
    },
};

use bevy::ecs::system::ResMut;

pub fn setup_blueprint_workbench(
    mut blueprints: ResMut<Blueprints>,
    asset_server: Res<AssetServer>,
) {
    let stone_texture: Handle<Image> =
        asset_server.load_with_settings("textures/stone.png", image_loader_settings);

    blueprints.0.insert(
        BlueprintType::Workbench,
        Blueprint {
            name: "Workbench".to_string(),
            slots: vec![
                BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                },
                BuildSlot {
                    flags: vec![ItemTag::BasicBuildMaterial],
                },
            ],
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
            mesh: asset_server.load("workbench.gltf#Mesh0/Primitive0"),
        },
    );
}
