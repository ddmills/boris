use bevy::ecs::{component::Component, event::EventWriter, query::With, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{Blackboard, JobCancelEvent, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskJobCancel;

pub fn task_job_cancel(
    mut q_actors: Query<(&Blackboard, &mut TaskState), With<TaskJobCancel>>,
    mut ev_job_cancel: EventWriter<JobCancelEvent>,
) {
    for (blackboard, mut state) in q_actors.iter_mut() {
        let Some(job_entity) = blackboard.job else {
            println!("no job on blackboard, cannot cancel!");
            *state = TaskState::Failed;
            continue;
        };

        ev_job_cancel.send(JobCancelEvent(job_entity));
        *state = TaskState::Success;
    }
}
