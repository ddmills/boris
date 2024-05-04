use bevy::{
    asset::{Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
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
    rendering::BasicMaterial,
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
}

pub fn on_spawn_commodity(
    mut cmd: Commands,
    mut ev_spawn_commodity: EventReader<SpawnCommodityEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    commodities: Res<Commodities>,
) {
    for ev in ev_spawn_commodity.read() {
        let Some(commodity) = commodities.0.get(&ev.commodity) else {
            continue;
        };

        let material = materials.add(BasicMaterial {
            texture: Some(commodity.texture.clone()),
            sunlight: 8,
            torchlight: 8,
            color: commodity.color,
        });

        cmd.spawn((
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
    }
}

#[derive(Event)]
pub struct SetSlotEvent {
    pub target_slot_idx: usize,
    pub target: Entity,
    pub content: Entity,
}

pub fn on_set_slot(
    mut cmd: Commands,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    mut ev_set_slot: EventReader<SetSlotEvent>,
    mut q_structures: Query<(&mut PartSlots, &Handle<BasicMaterial>)>,
    q_commodities: Query<&Commodity>,
    commodities: Res<Commodities>,
) {
    for ev in ev_set_slot.read() {
        let Ok((mut part_slots, mat_handle)) = q_structures.get_mut(ev.target) else {
            println!("Structure slot does not exist, cannot set slot!");
            continue;
        };

        let Some(slot) = part_slots.slots.get_mut(ev.target_slot_idx) else {
            println!("Target slot does not exist! cannot set slot!");
            continue;
        };

        if !slot.is_empty() {
            println!("Target slot already has content! cannot set slot!");
            continue;
        }

        println!("Setting slot {} content!", ev.target_slot_idx);

        slot.content = Some(ev.content);

        let mut ecmd = cmd.entity(ev.content);
        ecmd.insert(Visibility::Hidden);
        ecmd.insert(InSlot {
            holder: ev.target,
            slot_idx: ev.target_slot_idx,
        });

        if ev.target_slot_idx == 0 {
            let Ok(commodity_type) = q_commodities.get(ev.content) else {
                continue;
            };

            let Some(commodity_data) = commodities.0.get(commodity_type) else {
                continue;
            };

            let Some(material) = basic_materials.get_mut(mat_handle) else {
                continue;
            };

            material.color = commodity_data.color;
            material.texture = Some(commodity_data.texture.clone());
        }
    }
}
