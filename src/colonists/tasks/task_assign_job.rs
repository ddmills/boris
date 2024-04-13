use bevy::ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query},
};
use task_derive::TaskBuilder;

use crate::colonists::{
    ActorRef, Blackboard, Job, JobAssignment, JobLocation, TaskBuilder, TaskState,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskAssignJob(pub Entity);

pub fn task_assign_job(
    mut cmd: Commands,
    mut q_jobs: Query<(&mut Job, Option<&JobLocation>)>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut Blackboard, &TaskAssignJob)>,
) {
    for (ActorRef(actor), mut state, mut blackboard, task) in q_behavior.iter_mut() {
        // todo: Make this safe and possibly check for actor and job to exist first!
        // check if actor already has a job
        // check if job is already assigned

        let Ok((mut job, job_location)) = q_jobs.get_mut(task.0) else {
            *state = TaskState::Failed;
            continue;
        };

        if job.assignee.is_some() {
            println!("Duplicate assignment prevented!");
            *state = TaskState::Failed;
            continue;
        }

        if let Some(pos) = job_location {
            blackboard.target_block = Some(pos.pos);
        }

        job.assignee = Some(*actor);
        cmd.entity(*actor).insert(JobAssignment { job: task.0 });

        blackboard.job = Some(task.0);
        *state = TaskState::Success;
    }
}
