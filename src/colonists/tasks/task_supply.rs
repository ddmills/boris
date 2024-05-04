use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    query::With,
    system::{Commands, Query},
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, Blackboard, InInventory, Inventory, TaskBuilder, TaskState},
    items::SetSlotEvent,
    structures::PartSlots,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskSupply {
    pub target: Entity,
    pub target_slot_idx: usize,
}

pub fn task_supply(
    mut cmd: Commands,
    mut q_structures: Query<&mut PartSlots>,
    mut q_actors: Query<&mut Inventory, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &TaskSupply, &Blackboard)>,
    mut ev_set_slot: EventWriter<SetSlotEvent>,
) {
    for (ActorRef(actor), mut state, task_supply, blackboard) in q_behavior.iter_mut() {
        let Some(item) = blackboard.item else {
            println!("No item assign in blackboard, cannot supply!");
            *state = TaskState::Failed;
            continue;
        };

        let Ok(mut inventory) = q_actors.get_mut(*actor) else {
            println!("Actor does not have an inventory, cannot supply anything!");
            *state = TaskState::Failed;
            continue;
        };

        let Ok(mut part_slots) = q_structures.get_mut(task_supply.target) else {
            println!("Structure slot does not exist, cannot supply!");
            *state = TaskState::Failed;
            continue;
        };

        let Some(slot) = part_slots.slots.get_mut(task_supply.target_slot_idx) else {
            println!("Target slot does not exist! cannot supply!");
            *state = TaskState::Failed;
            continue;
        };

        if !slot.is_empty() {
            println!("Target slot already has content! cannot supply!");
            *state = TaskState::Failed;
            continue;
        }

        inventory.items.remove(&item);

        let mut ecmd = cmd.entity(item);
        ecmd.remove::<InInventory>();

        ev_set_slot.send(SetSlotEvent {
            target_slot_idx: task_supply.target_slot_idx,
            target: task_supply.target,
            content: item,
        });

        *state = TaskState::Success;
    }
}
