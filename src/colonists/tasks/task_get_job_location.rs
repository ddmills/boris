use bevy::ecs::{component::Component, query::With, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{job_access_points, Blackboard, Job, JobLocation, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskGetJobLocation;

pub fn task_get_job_location(
    q_jobs: Query<(&Job, &JobLocation)>,
    mut q_behavior: Query<(&mut Blackboard, &mut TaskState), With<TaskGetJobLocation>>,
) {
    for (mut blackboard, mut state) in q_behavior.iter_mut() {
        let Some(job_entity) = blackboard.job else {
            *state = TaskState::Failed;
            continue;
        };

        let Ok((job, job_location)) = q_jobs.get(job_entity) else {
            *state = TaskState::Failed;
            continue;
        };

        blackboard.move_goals = job_access_points(job_location.pos, job.job_type);

        *state = TaskState::Success;
    }
}
