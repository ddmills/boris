use bevy::ecs::{
    component::Component,
    query::With,
    system::{Query, ResMut},
};
use task_derive::TaskBuilder;

use crate::colonists::{Blackboard, JobList, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskReturnJob;

pub fn task_return_job(
    mut jobs: ResMut<JobList>,
    mut q_actors: Query<(&Blackboard, &mut TaskState), With<TaskReturnJob>>,
) {
    for (blackboard, mut state) in q_actors.iter_mut() {
        let Some(job) = blackboard.job else {
            println!("no job on blackboard, cannot return to queue!");
            *state = TaskState::Failed;
            continue;
        };

        println!("Returning job to job queue");

        jobs.queue(job);
        *state = TaskState::Success;
    }
}
