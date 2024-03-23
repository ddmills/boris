use bevy::ecs::{component::Component, query::With, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{ActorRef, Blackboard, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskFindBed;

pub fn task_find_bed(
    mut q_behavior: Query<(&ActorRef, &mut Blackboard, &mut TaskState), With<TaskFindBed>>,
) {
    for (ActorRef(entity), mut blackboard, mut state) in q_behavior.iter_mut() {
        println!("find a bed for {}", entity.index());
        blackboard.bed = 3;
        *state = TaskState::Success;
    }
}
