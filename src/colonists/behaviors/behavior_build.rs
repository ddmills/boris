use std::sync::Arc;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{EntityCommands, Query, Res},
    },
    transform::components::Transform,
};

use crate::{
    colonists::{
        is_reachable, job_access_points, test_item_tags, tree_aquire_item, Actor, ActorRef,
        Behavior, BehaviorNode, HasBehavior, InInventory, Inventory, IsJobAccessible,
        IsJobCancelled, IsJobCompleted, Item, ItemTag, Job, JobBuild, JobLocation, NavigationFlags,
        NavigationGraph, PartitionPathRequest, Score, ScorerBuilder, TaskBuildBlock,
        TaskGetJobLocation, TaskIsTargetEmpty, TaskJobAssign, TaskJobCancel, TaskJobComplete,
        TaskJobUnassign, TaskMoveTo,
    },
    common::Distance,
    BlockType, Terrain,
};

#[derive(Component, Clone, Default)]
pub struct ScorerBuild {
    job: Option<Entity>,
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
                    BehaviorNode::IfElse(
                        Box::new(BehaviorNode::Task(Arc::new(TaskIsTargetEmpty))),
                        Box::new(BehaviorNode::Sequence(vec![
                            tree_aquire_item(vec![ItemTag::Stone]),
                            BehaviorNode::Sequence(vec![
                                BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                                BehaviorNode::Task(Arc::new(TaskMoveTo::default())),
                                BehaviorNode::Task(Arc::new(TaskBuildBlock {
                                    progress: 0.,
                                    block: BlockType::STONE,
                                })),
                                BehaviorNode::Task(Arc::new(TaskJobComplete)),
                            ]),
                        ])),
                        Box::new(BehaviorNode::Task(Arc::new(TaskJobCancel))),
                    ),
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
        (Entity, &Job, &JobLocation),
        (
            With<JobBuild>,
            With<IsJobAccessible>,
            Without<IsJobCancelled>,
            Without<IsJobCompleted>,
        ),
    >,
    q_items: Query<&Item>,
    q_free_items: Query<(&Item, &Transform), Without<InInventory>>,
    q_actors: Query<
        (&Inventory, &Transform, &NavigationFlags),
        (With<Actor>, Without<HasBehavior>),
    >,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &mut ScorerBuild)>,
) {
    for (ActorRef(actor), mut score, mut scorer) in q_behaviors.iter_mut() {
        let Ok((inventory, transform, flags)) = q_actors.get(*actor) else {
            *score = Score(0.);
            continue;
        };

        let pos = [
            transform.translation.x as u32,
            transform.translation.y as u32,
            transform.translation.z as u32,
        ];

        let mut best = None;
        let mut best_dist = 100000.;

        for (e, job, job_location) in q_jobs.iter() {
            if job.assignee.is_some() {
                continue;
            }

            let goals = job_access_points(job_location.pos, job.job_type);
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
                    job_location.pos[0] as i32,
                    job_location.pos[1] as i32,
                    job_location.pos[2] as i32,
                ],
                [pos[0] as i32, pos[1] as i32, pos[2] as i32],
            );

            if job_distance < best_dist {
                best = Some(e);
                best_dist = job_distance;
                if job_distance < 2. {
                    break;
                }
            }
        }

        if best.is_none() {
            *score = Score(0.);
            continue;
        };

        scorer.job = best;

        let item_tags = &[ItemTag::Stone];

        let has_stone = inventory.items.iter().any(|e| {
            let Ok(item) = q_items.get(*e) else {
                return false;
            };

            test_item_tags(&item.tags, item_tags)
        });

        // if we have stone, score is higher
        if has_stone {
            *score = Score(0.6);
            continue;
        }

        // check if any of the items are unreserved and accessible
        if q_free_items.iter().any(|(i, t)| {
            test_item_tags(&i.tags, item_tags)
                && i.reserved.is_none()
                && is_reachable(
                    &PartitionPathRequest {
                        start: pos,
                        goals: vec![[
                            t.translation.x as u32,
                            t.translation.y as u32,
                            t.translation.z as u32,
                        ]],
                        flags: *flags,
                    },
                    &terrain,
                    &graph,
                )
        }) {
            *score = Score(0.2);
            continue;
        } else {
            *score = Score(0.0);
        }
    }
}
