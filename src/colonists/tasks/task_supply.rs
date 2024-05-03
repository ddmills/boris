use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
    render::view::Visibility,
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        Actor, ActorRef, Blackboard, InInventory, InSlot, Inventory, TaskBuilder, TaskState,
    },
    furniture::BlueprintSlots,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskSupply {
    pub target: Entity,
    pub target_slot_idx: usize,
}

pub fn task_supply(
    mut cmd: Commands,
    mut q_blueprints: Query<&mut BlueprintSlots>,
    mut q_actors: Query<&mut Inventory, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &TaskSupply, &Blackboard)>,
) {
    for (ActorRef(actor), mut state, task_supply, blackboard) in q_behavior.iter_mut() {
        println!("supplying item!");

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

        let Ok(mut blueprint_slots) = q_blueprints.get_mut(task_supply.target) else {
            println!("Blueprint slot does not exist, cannot supply!");
            *state = TaskState::Failed;
            continue;
        };

        let Some(slot) = blueprint_slots.slots.get_mut(task_supply.target_slot_idx) else {
            println!("Target slot does not exist! cannot supply!");
            *state = TaskState::Failed;
            continue;
        };

        if !slot.is_empty() {
            println!("Target slot already has content! cannot supply!");
            *state = TaskState::Failed;
            continue;
        }
        slot.content = Some(item);

        inventory.items.remove(&item);

        let mut ecmd = cmd.entity(item);
        ecmd.remove::<InInventory>();
        ecmd.insert(Visibility::Hidden);
        ecmd.insert(InSlot {
            holder: task_supply.target,
            slot_idx: task_supply.target_slot_idx,
        });

        *state = TaskState::Success;
    }
}
