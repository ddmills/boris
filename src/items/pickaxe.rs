use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        event::{Event, EventReader, EventWriter},
        system::{Commands, Res, ResMut},
    },
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{
        texture::{
            Image, ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
            ImageSamplerDescriptor,
        },
        view::Visibility,
    },
    transform::components::Transform,
};

use crate::{
    colonists::{Faller, Item, ItemTag},
    rendering::{
        BasicMaterial,
        SlotIndex::{self, Slot0},
    },
    structures::{PartSlot, PartSlots},
    Position,
};

use super::{Commodity, SpawnCommodityEvent};

#[derive(Event)]
pub struct SpawnPickaxeEvent {
    pub pos: [u32; 3],
}

pub fn image_loader_settings(s: &mut ImageLoaderSettings) {
    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    })
}

pub fn on_spawn_pickaxe(
    mut cmd: Commands,
    mut ev_spawn_pickaxe: EventReader<SpawnPickaxeEvent>,
    mut ev_spawn_commodity: EventWriter<SpawnCommodityEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_pickaxe.read() {
        let terrain_texture: Handle<Image> =
            asset_server.load_with_settings("textures/comfy.png", image_loader_settings);
        let mesh = asset_server.load("pickaxe.gltf#Mesh0/Primitive0");
        let material = materials.add(BasicMaterial {
            slots_texture: Some(terrain_texture),
            ..Default::default()
        });

        let entity = cmd
            .spawn((
                Name::new("Pickaxe"),
                MaterialMeshBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(
                        ev.pos[0] as f32 + 0.5,
                        ev.pos[1] as f32,
                        ev.pos[2] as f32 + 0.5,
                    ),
                    visibility: Visibility::Visible,
                    ..default()
                },
                Item {
                    tags: vec![ItemTag::Pickaxe],
                    reserved: None,
                },
                PartSlots {
                    slot_0: Some(PartSlot {
                        idx: SlotIndex::Slot0,
                        flags: vec![ItemTag::Stone],
                        content: None,
                    }),
                    slot_1: Some(PartSlot {
                        idx: SlotIndex::Slot1,
                        flags: vec![ItemTag::Log],
                        content: None,
                    }),
                    slot_2: None,
                },
                Faller,
                Position::default(),
            ))
            .id();

        ev_spawn_commodity.send(SpawnCommodityEvent {
            commodity: Commodity::StoneShaleBoulder,
            position: ev.pos,
            slot_entity: Some(entity),
            slot_idx: Some(SlotIndex::Slot0),
        });
        ev_spawn_commodity.send(SpawnCommodityEvent {
            commodity: Commodity::WoodBirchLog,
            position: ev.pos,
            slot_entity: Some(entity),
            slot_idx: Some(SlotIndex::Slot1),
        });
    }
}
