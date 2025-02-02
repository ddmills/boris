use std::fmt::{Display, Formatter, Result};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::{Commands, Query, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    reflect::Reflect,
    utils::HashSet,
};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};

use crate::{rendering::SlotIndex, Position, Terrain};

use super::NavigationGraph;

#[derive(Component, Default, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Inventory {
    pub items: HashSet<Entity>,
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Item {
    pub tags: Vec<ItemTag>,
    pub reserved: Option<Entity>,
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct InInventory {
    pub holder: Entity,
}

#[derive(Component)]
pub struct InSlot {
    pub holder: Entity,
    pub slot_idx: SlotIndex,
}

#[derive(Clone, Copy, PartialEq, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub enum ItemTag {
    Axe,
    Pickaxe,
    Stone,
    Log,
    BasicBuildMaterial,
}

impl Display for ItemTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

pub fn test_item_tags(all: &[ItemTag], test: &[ItemTag]) -> bool {
    test.iter().all(|tag| all.contains(tag))
}

#[derive(Event)]
pub struct DestroyItemEvent {
    pub entity: Entity,
}

pub fn destroy_items(
    mut terrain: ResMut<Terrain>,
    mut graph: ResMut<NavigationGraph>,
    mut cmd: Commands,
    q_items: Query<&Position>,
    mut ev_destroy_item: EventReader<DestroyItemEvent>,
) {
    for ev in ev_destroy_item.read() {
        cmd.entity(ev.entity).despawn_recursive();

        let Ok(position) = q_items.get(ev.entity) else {
            continue;
        };

        terrain.remove_item(position.chunk_idx, position.block_idx, &ev.entity);

        let Some(partition_id) = position.partition_id else {
            continue;
        };

        if !graph.remove_item_from_partition(&partition_id, &ev.entity) {
            println!(
                "Item not in expected partition! item={} partition_id={}",
                ev.entity.index(),
                partition_id
            );
        }
    }
}
