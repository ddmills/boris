use bevy::{
    asset::AssetServer,
    ecs::system::{Res, ResMut},
    render::color::Color,
};

use crate::{colonists::ItemTag, items::image_loader_settings};

use super::{Commodities, Commodity, CommodityData, CommodityFlag};

pub fn setup_commodity_wood_birch_log(
    mut commodities: ResMut<Commodities>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load_with_settings("textures/wood.png", image_loader_settings);
    let mesh = asset_server.load("log.gltf#Mesh0/Primitive0");

    commodities.0.insert(
        Commodity::WoodBirchLog,
        CommodityData {
            name: "Birch log".to_string(),
            texture,
            texture_idx: 34,
            mesh,
            color: Color::rgb(1.0, 0.6, 0.6),
            flags: CommodityFlag::WOOD | CommodityFlag::LOG,
            tags: vec![ItemTag::Log, ItemTag::BasicBuildMaterial],
        },
    );
}
