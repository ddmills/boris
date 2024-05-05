use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{EntityCommands, Query, Res},
};

use crate::{
    colonists::{
        is_reachable, job_access_points_many, Actor, ActorRef, Behavior, BehaviorNode, HasBehavior,
        IsJobAccessible, IsJobCancelled, Job, JobBuild, JobLocation, NavigationFlags,
        NavigationGraph, PartitionPathRequest, Score, ScorerBuilder, TaskBuild, TaskGetJobLocation,
        TaskJobAssign, TaskJobComplete, TaskJobUnassign, TaskLookAt, TaskMoveTo,
    },
    common::Distance,
    structures::PartSlots,
    Position, Terrain,
};

#[derive(Component, Clone, Default)]
pub struct ScorerBuild {
    job: Option<Entity>,
    structure: Option<Entity>,
}

impl ScorerBuilder for ScorerBuild {
    fn insert(&self, cmd: &mut EntityCommands) {
        cmd.insert(self.clone());
    }

    fn label(&self) -> String {
        "Build".to_string()
    }

    fn build(&self) -> Behavior {
        Behavior::new(
            "Build",
            BehaviorNode::Try(
                Box::new(BehaviorNode::Sequence(vec![
                    BehaviorNode::Task(Arc::new(TaskJobAssign(self.job.unwrap()))),
                    // tree_aquire_item(vec![ItemTag::Axe]),
                    // BehaviorNode::Task(Arc::new(TaskItemEquip)),
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                        BehaviorNode::Task(Arc::new(TaskMoveTo::default())),
                        BehaviorNode::Task(Arc::new(TaskLookAt)),
                        BehaviorNode::Task(Arc::new(TaskBuild {
                            progress: 0.,
                            structure: self.structure.unwrap(),
                        })),
                        BehaviorNode::Task(Arc::new(TaskJobComplete)),
                    ]),
                ])),
                Box::new(BehaviorNode::Task(Arc::new(TaskJobUnassign))),
            ),
        )
    }
}

pub fn score_build(
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    q_jobs: Query<
        (Entity, &Job, &JobBuild, &JobLocation),
        (
            With<IsJobAccessible>,
            Without<IsJobCancelled>,
            Without<TaskJobComplete>,
        ),
    >,
    q_slots: Query<&PartSlots>,
    q_actors: Query<(&Position, &NavigationFlags), (With<Actor>, Without<HasBehavior>)>,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &mut ScorerBuild)>,
) {
    for (ActorRef(actor), mut score, mut scorer) in q_behaviors.iter_mut() {
        let Ok((position, flags)) = q_actors.get(*actor) else {
            *score = Score(0.);
            continue;
        };

        let pos = [position.x, position.y, position.z];

        let mut best = None;
        let mut best_structure = None;
        let mut best_dist = 100000.;

        for (e, job, job_build, job_location) in q_jobs.iter() {
            if job.assignee.is_some() {
                continue;
            }

            if let Ok(slots) = q_slots.get(job_build.structure) {
                if slots.as_vec().iter().any(|s| s.is_empty()) {
                    continue;
                }
            };

            let goals = job_access_points_many(&job_location.targets, job.job_type);
            let request = PartitionPathRequest {
                start: pos,
                goals,
                flags: *flags,
            };

            if !is_reachable(&request, &terrain, &graph) {
                continue;
            }

            let job_distance = Distance::manhattan(
                [
                    job_location.primary_target[0] as i32,
                    job_location.primary_target[1] as i32,
                    job_location.primary_target[2] as i32,
                ],
                [pos[0] as i32, pos[1] as i32, pos[2] as i32],
            );

            if job_distance < best_dist {
                best = Some(e);
                best_dist = job_distance;
                best_structure = Some(job_build.structure);
                if job_distance < 2. {
                    break;
                }
            }
        }

        if best.is_none() || best_structure.is_none() {
            *score = Score(0.);
            continue;
        };

        scorer.job = best;
        scorer.structure = best_structure;

        *score = Score(7.);
    }
}
