use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    render::view::Visibility,
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        Actor, ActorRef, Blackboard, InInventory, InPartition, Inventory, Item, NavigationGraph,
        TaskBuilder, TaskState,
    },
    Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskItemPickUp;

pub fn task_item_pick_up(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    mut graph: ResMut<NavigationGraph>,
    q_items: Query<&Transform, With<Item>>,
    mut q_actors: Query<&mut Inventory, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &Blackboard), With<TaskItemPickUp>>,
) {
    for (ActorRef(actor), mut state, blackboard) in q_behavior.iter_mut() {
        let Some(item) = blackboard.item else {
            println!("No item assign in blackboard, cannot pick anything up!");
            *state = TaskState::Failed;
            continue;
        };

        let Ok(mut inventory) = q_actors.get_mut(*actor) else {
            println!("Actor does not have an inventory, cannot pick anything up!");
            *state = TaskState::Failed;
            continue;
        };

        let Ok(item_transform) = q_items.get(item) else {
            println!("Item does not exist, cannot pick up!");
            *state = TaskState::Failed;
            continue;
        };

        let item_x = item_transform.translation.x as u32;
        let item_y = item_transform.translation.y as u32;
        let item_z = item_transform.translation.z as u32;

        let Some(partition_id) = terrain.get_partition_id_u32(item_x, item_y, item_z) else {
            panic!("Missing partition_id?");
        };

        let Some(partition) = graph.get_partition_mut(&partition_id) else {
            panic!("Missing partition!? {}", partition_id);
        };

        if !partition.items.remove(&item) {
            println!("Item not here!");
            *state = TaskState::Failed;
            return;
        }

        let mut ecmd = cmd.entity(item);
        ecmd.remove::<InPartition>();

        inventory.items.push(item);
        ecmd.insert(Visibility::Hidden);
        ecmd.insert(InInventory { holder: *actor });

        *state = TaskState::Success;
    }
}
