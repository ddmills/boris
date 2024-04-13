use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query},
};
use task_derive::TaskBuilder;

use crate::colonists::{Blackboard, IsJobCompleted, Job, JobAssignment, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskJobComplete;

pub fn task_job_complete(
    mut cmd: Commands,
    job_holders: Query<Entity>,
    mut q_jobs: Query<&mut Job>,
    mut q_actors: Query<(&Blackboard, &mut TaskState), With<TaskJobComplete>>,
) {
    for (blackboard, mut state) in q_actors.iter_mut() {
        let Some(job_entity) = blackboard.job else {
            println!("no job on blackboard, cannot complete!");
            *state = TaskState::Failed;
            continue;
        };

        println!("Completing job");
        let Ok(mut job) = q_jobs.get_mut(job_entity) else {
            println!("ERR: job does not exist!?");
            *state = TaskState::Failed;
            continue;
        };

        if let Some(job_assignee) = job.assignee {
            if let Ok(holder) = job_holders.get(job_assignee) {
                cmd.entity(holder).remove::<JobAssignment>();
            } else {
                println!("ERR: no holder for job!?");
            };
        }

        cmd.entity(job_entity).insert(IsJobCompleted);

        job.assignee = None;
        *state = TaskState::Success;
    }
}
