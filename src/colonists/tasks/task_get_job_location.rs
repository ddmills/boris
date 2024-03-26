use bevy::ecs::{component::Component, query::With, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{Blackboard, JobLocation, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskGetJobLocation;

pub fn task_get_job_location(
    q_jobs: Query<&JobLocation>,
    mut q_behavior: Query<(&mut Blackboard, &mut TaskState), With<TaskGetJobLocation>>,
) {
    for (mut blackboard, mut state) in q_behavior.iter_mut() {
        let Some(job_entity) = blackboard.job else {
            *state = TaskState::Failed;
            continue;
        };

        let Ok(location) = q_jobs.get(job_entity) else {
            *state = TaskState::Failed;
            continue;
        };

        let pos = location.pos;

        blackboard.move_goals = vec![
            [pos[0] + 1, pos[1], pos[2]],
            [pos[0] - 1, pos[1], pos[2]],
            [pos[0], pos[1], pos[2] + 1],
            [pos[0], pos[1], pos[2] - 1],
            [pos[0] + 1, pos[1] + 1, pos[2]],
            [pos[0] - 1, pos[1] + 1, pos[2]],
            [pos[0], pos[1] + 1, pos[2] + 1],
            [pos[0], pos[1] + 1, pos[2] - 1],
            [pos[0] + 1, pos[1] - 1, pos[2]],
            [pos[0] - 1, pos[1] - 1, pos[2]],
            [pos[0], pos[1] - 1, pos[2] + 1],
            [pos[0], pos[1] - 1, pos[2] - 1],
            [pos[0], pos[1] - 2, pos[2] + 1],
            [pos[0], pos[1] - 2, pos[2] - 1],
            [pos[0] - 1, pos[1], pos[2] + 1],
            [pos[0] - 1, pos[1], pos[2] - 1],
            [pos[0] + 1, pos[1], pos[2] + 1],
            [pos[0] + 1, pos[1], pos[2] - 1],
        ];

        *state = TaskState::Success;
    }
}
