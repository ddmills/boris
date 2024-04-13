use bevy::ecs::{component::Component, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{
    test_item_tags, ActorRef, Blackboard, Inventory, Item, ItemTag, TaskBuilder, TaskState,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskCheckHasItem(pub Vec<ItemTag>);

pub fn task_check_has_item(
    q_items: Query<&Item>,
    q_inventories: Query<&Inventory>,
    mut q_behavior: Query<(
        &ActorRef,
        &mut TaskState,
        &mut Blackboard,
        &TaskCheckHasItem,
    )>,
) {
    for (ActorRef(actor), mut state, mut blackboard, task) in q_behavior.iter_mut() {
        let Ok(inventory) = q_inventories.get(*actor) else {
            *state = TaskState::Failed;
            continue;
        };

        let has_item = inventory.items.iter().any(|e| {
            let Ok(item) = q_items.get(*e) else {
                return false;
            };

            let tag_match = test_item_tags(&item.tags, &task.0);

            if tag_match {
                blackboard.item = Some(*e);
            }

            tag_match
        });

        *state = match has_item {
            true => TaskState::Success,
            false => TaskState::Failed,
        }
    }
}
