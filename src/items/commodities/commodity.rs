use bevy::{
    asset::{Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{color::Color, mesh::Mesh, texture::Image, view::Visibility},
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    colonists::{Faller, InSlot, Item, ItemTag},
    rendering::{
        BasicMaterial,
        SlotIndex::{self},
    },
    structures::PartSlots,
    Position,
};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
    pub struct CommodityFlag: u32 {
        const NONE = 0;
        const STONE = 1;
        const WOOD = 2;
        const BOULDER = 4;
        const LOG = 8;
        const BASIC_BUILD = Self::STONE.bits() | Self::WOOD.bits();
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Commodity {
    WoodBirchLog,
    StoneShaleBoulder,
}

pub struct CommodityData {
    pub name: String,
    pub texture: Handle<Image>,
    pub texture_idx: u8,
    pub mesh: Handle<Mesh>,
    pub color: Color,
    pub flags: CommodityFlag,
    pub tags: Vec<ItemTag>,
}

#[derive(Resource, Default)]
pub struct Commodities(pub HashMap<Commodity, CommodityData>);

#[derive(Event)]
pub struct SpawnCommodityEvent {
    pub commodity: Commodity,
    pub position: [u32; 3],
    pub slot_entity: Option<Entity>,
    pub slot_idx: Option<SlotIndex>,
}

pub fn on_spawn_commodity(
    mut cmd: Commands,
    mut ev_spawn_commodity: EventReader<SpawnCommodityEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    mut ev_set_slot: EventWriter<SetSlotEvent>,
    commodities: Res<Commodities>,
) {
    for ev in ev_spawn_commodity.read() {
        let Some(commodity) = commodities.0.get(&ev.commodity) else {
            continue;
        };

        let material = materials.add(BasicMaterial {
            texture: Some(commodity.texture.clone()),
            color: commodity.color,
            ..Default::default()
        });

        // let entity = ev.entity_id.unwrap_or_else(|| cmd.spawn_empty().id());
        let entity = cmd.spawn_empty().id();

        let mut ecmd = cmd.entity(entity);

        ecmd.insert((
            Name::new(commodity.name.clone()),
            MaterialMeshBundle {
                mesh: commodity.mesh.clone(),
                material,
                transform: Transform::from_xyz(
                    ev.position[0] as f32 + 0.5,
                    ev.position[1] as f32,
                    ev.position[2] as f32 + 0.5,
                ),
                visibility: Visibility::Visible,
                ..default()
            },
            ev.commodity,
            Item {
                tags: commodity.tags.clone(),
                reserved: None,
            },
            Faller,
            Position::default(),
        ));

        if let Some(slot_entity) = ev.slot_entity {
            if let Some(slot_idx) = ev.slot_idx {
                ev_set_slot.send(SetSlotEvent {
                    target: slot_entity,
                    target_slot: slot_idx,
                    content: entity,
                });
            } else {
                println!("Err: Slot entity provided, but no index provided");
            }
        }
    }
}

#[derive(Event)]
pub struct SetSlotEvent {
    pub target_slot: SlotIndex,
    pub target: Entity,
    pub content: Entity,
}

pub fn on_set_slot(
    mut cmd: Commands,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    mut ev_set_slot: EventReader<SetSlotEvent>,
    mut q_structures: Query<(&mut PartSlots, &Handle<BasicMaterial>)>,
    mut q_items: Query<&mut Item>,
    q_commodities: Query<&Commodity>,
    commodities: Res<Commodities>,
) {
    for ev in ev_set_slot.read() {
        let Ok((mut part_slots, mat_handle)) = q_structures.get_mut(ev.target) else {
            println!("Structure slot does not exist, cannot set slot!");
            continue;
        };

        let Some(slot) = part_slots.get_mut(ev.target_slot) else {
            println!("Target slot does not exist! cannot set slot!");
            continue;
        };

        if !slot.is_empty() {
            println!("Target slot already has content! cannot set slot!");
            continue;
        }

        slot.content = Some(ev.content);

        let mut ecmd = cmd.entity(ev.content);
        ecmd.insert(Visibility::Hidden);
        ecmd.insert(InSlot {
            holder: ev.target,
            slot_idx: ev.target_slot,
        });

        if let Ok(mut item) = q_items.get_mut(ev.content) {
            item.reserved = None;
        };

        let Ok(commodity_type) = q_commodities.get(ev.content) else {
            println!("Trying to set slot, no commodity_type");
            continue;
        };

        let Some(commodity_data) = commodities.0.get(commodity_type) else {
            println!("Trying to set slot, no commodity_data");
            continue;
        };

        let Some(material) = basic_materials.get_mut(mat_handle) else {
            println!("Trying to set slot, no material");
            continue;
        };

        material.with_slot(
            ev.target_slot,
            commodity_data.texture_idx,
            commodity_data.color,
        );
    }
}
