use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::Commands,
    },
    hierarchy::DespawnRecursiveExt,
};

#[derive(Component, Default)]
pub struct Inventory {
    pub items: Vec<Entity>,
}

#[derive(Component)]
pub struct Item {
    pub tags: Vec<ItemTag>,
    pub reserved: Option<Entity>,
}

#[derive(Component)]
pub struct InInventory {
    pub holder: Entity,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemTag {
    Pickaxe,
    Stone,
}

pub fn test_item_tags(all: &[ItemTag], test: &[ItemTag]) -> bool {
    test.iter().all(|tag| all.contains(tag))
}

#[derive(Event)]
pub struct DestroyItemEvent {
    pub entity: Entity,
}

pub fn destroy_items(mut cmd: Commands, mut ev_destroy_item: EventReader<DestroyItemEvent>) {
    for ev in ev_destroy_item.read() {
        cmd.entity(ev.entity).despawn_recursive();
    }
}
