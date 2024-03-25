use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query},
};
use task_derive::TaskBuilder;

use crate::colonists::{Blackboard, IsJobAssigned, JobAssignment, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskUnassignJob;

pub fn task_unassign_job(
    mut cmd: Commands,
    job_holders: Query<Entity>,
    q_jobs: Query<&IsJobAssigned>,
    mut q_actors: Query<(&Blackboard, &mut TaskState), With<TaskUnassignJob>>,
) {
    for (blackboard, mut state) in q_actors.iter_mut() {
        let Some(job) = blackboard.job else {
            println!("no job on blackboard, cannot return to queue!");
            *state = TaskState::Failed;
            continue;
        };

        println!("Returning job to job queue");

        if let Ok(assigned) = q_jobs.get(job) {
            if let Ok(holder) = job_holders.get(assigned.assignee) {
                cmd.entity(holder).remove::<JobAssignment>();
            } else {
                println!("ERR: no holder for job!?");
            };
        } else {
            println!("ERR: job does not exist!?");
        };

        cmd.entity(job).remove::<IsJobAssigned>();
        *state = TaskState::Success;
    }
}
