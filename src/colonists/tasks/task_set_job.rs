use bevy::ecs::{component::Component, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{Blackboard, Job, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskSetJob(pub Job);

pub fn task_set_job(mut q_behavior: Query<(&mut TaskState, &mut Blackboard, &TaskSetJob)>) {
    for (mut state, mut blackboard, task) in q_behavior.iter_mut() {
        blackboard.job = Some(task.0);
        *state = TaskState::Success;
    }
}
