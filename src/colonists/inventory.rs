use bevy::ecs::{component::Component, entity::Entity, system::Commands};

#[derive(Component, Default)]
pub struct Inventory {
    pub items: Vec<Entity>,
}

impl Inventory {
    pub fn add_item(&mut self, mut cmd: Commands, target: Entity, item: Entity) {
        self.items.push(item);
        cmd.entity(item).insert(InInventory { holder: target });
    }

    pub fn remove_item(&mut self, mut cmd: Commands, item: Entity) {
        self.items.retain(|i| *i != item);
        cmd.entity(item).remove::<InInventory>();
    }
}

#[derive(Component)]
pub struct Item {
    pub tags: Vec<ItemTag>,
}

#[derive(Component)]
pub struct InInventory {
    pub holder: Entity,
}

#[derive(Clone, PartialEq)]
pub enum ItemTag {
    PickAxe,
}

pub fn test_item_flags(all: &Vec<ItemTag>, test: &Vec<ItemTag>) -> bool {
    test.iter().all(|tag| all.contains(tag))
}
