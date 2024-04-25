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
pub struct TaskJobAssign(pub Entity);

pub fn task_job_assign(
    mut cmd: Commands,
    mut q_jobs: Query<(&mut Job, Option<&JobLocation>)>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut Blackboard, &TaskJobAssign)>,
) {
    for (ActorRef(actor), mut state, mut blackboard, task) in q_behavior.iter_mut() {
        // todo: Make this safe and possibly check for actor and job to exist first!
        // check if actor already has a job
        // check if job is already assigned

        let Ok((mut job, job_location)) = q_jobs.get_mut(task.0) else {
            println!("no job location!");
            *state = TaskState::Failed;
            continue;
        };

        if job.assignee.is_some() {
            println!("Duplicate assignment prevented!");
            *state = TaskState::Failed;
            continue;
        }

        if let Some(pos) = job_location {
            blackboard.target_block = Some(pos.primary_target);
        }

        job.assignee = Some(*actor);
        cmd.entity(*actor).insert(JobAssignment { job: task.0 });

        blackboard.job = Some(task.0);
        *state = TaskState::Success;
    }
}
