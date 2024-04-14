use bevy::ecs::{
    component::Component,
    query::With,
    system::{Query, Res},
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Blackboard, TaskBuilder, TaskState},
    Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskIsTargetEmpty;

pub fn task_is_target_empty(
    terrain: Res<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &Blackboard), With<TaskIsTargetEmpty>>,
) {
    for (mut state, blackboard) in q_behavior.iter_mut() {
        let Some([x, y, z]) = blackboard.target_block else {
            println!("Blackboard is missing target_block, cannot check if empty!");
            *state = TaskState::Failed;
            continue;
        };

        if terrain.get_block_type(x, y, z).is_empty() {
            *state = TaskState::Success;
            continue;
        }

        *state = TaskState::Failed;
    }
}
