use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query},
};
use task_derive::TaskBuilder;

use crate::colonists::{
    ActorRef, Blackboard, IsJobAssigned, Job, JobAssignment, JobType::Mine, TaskBuilder, TaskState,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskAssignJob(pub Entity);

pub fn task_assign_job(
    mut cmd: Commands,
    q_jobs: Query<&Job>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut Blackboard, &TaskAssignJob)>,
) {
    for (ActorRef(actor), mut state, mut blackboard, task) in q_behavior.iter_mut() {
        // todo: Make this safe and possibly check for actor and job to exist first!
        // check if actor already has a job
        // check if job is already assigned

        let Ok(job) = q_jobs.get(task.0) else {
            println!("ERR: Job does not exist, cannot be assigned.");
            *state = TaskState::Failed;
            continue;
        };

        match job.job_type {
            Mine(pos) => {
                blackboard.target_block = Some(pos);
            }
        }

        cmd.entity(*actor).insert(JobAssignment { job: task.0 });
        cmd.entity(task.0)
            .insert(IsJobAssigned { assignee: *actor });

        blackboard.job = Some(task.0);
        *state = TaskState::Success;
    }
}
