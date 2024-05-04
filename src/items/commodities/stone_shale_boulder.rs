use bevy::{
    asset::AssetServer,
    ecs::system::{Res, ResMut},
    render::color::Color,
};

use crate::{colonists::ItemTag, items::image_loader_settings};

use super::{Commodities, Commodity, CommodityData, CommodityFlag};

pub fn setup_commodity_stone_shale_boulder(
    mut commodities: ResMut<Commodities>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load_with_settings("textures/stone.png", image_loader_settings);
    let mesh = asset_server.load("sphere.gltf#Mesh0/Primitive0");

    commodities.0.insert(
        Commodity::StoneShaleBoulder,
        CommodityData {
            name: "Shale boulder".to_string(),
            texture,
            mesh,
            color: Color::GRAY,
            flags: CommodityFlag::STONE | CommodityFlag::BOULDER,
            tags: vec![ItemTag::Stone, ItemTag::BasicBuildMaterial],
        },
    );
}
