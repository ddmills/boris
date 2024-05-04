use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::Res,
    render::texture::Image,
};

use crate::{
    colonists::{ItemTag, NavigationFlags},
    items::image_loader_settings,
    structures::{
        Blueprint, BlueprintHotspot, BlueprintTile, BlueprintType, Blueprints, BuildSlot,
        DirectionSimple, TileRequirement,
    },
};

use bevy::ecs::system::ResMut;

pub fn setup_blueprint_ladder(mut blueprints: ResMut<Blueprints>, asset_server: Res<AssetServer>) {
    let wood_texture: Handle<Image> =
        asset_server.load_with_settings("textures/wood.png", image_loader_settings);

    blueprints.0.insert(
        BlueprintType::Ladder,
        Blueprint {
            name: "Ladder".to_string(),
            slots: vec![
                BuildSlot {
                    tags: vec![ItemTag::Log],
                },
                BuildSlot {
                    tags: vec![ItemTag::Log],
                },
            ],
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
                    nav_flags: NavigationFlags::LADDER,
                    is_blocker: false,
                    is_occupied: true,
                },
                BlueprintTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TileRequirement::empty(),
                    nav_flags: NavigationFlags::LADDER,
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
            texture: wood_texture.clone(),
            mesh: asset_server.load("ladder.gltf#Mesh0/Primitive0"),
        },
    );
}
