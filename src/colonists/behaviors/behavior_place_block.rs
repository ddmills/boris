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
        is_reachable, job_access_points_many, test_item_tags, tree_aquire_item, Actor, ActorRef,
        Behavior, BehaviorNode, HasBehavior, InInventory, InSlot, Inventory, IsJobAccessible,
        IsJobCancelled, IsJobCompleted, Item, ItemTag, Job, JobLocation, JobPlaceBlock, JobType,
        NavigationFlags, NavigationGraph, PartitionPathRequest, Score, ScorerBuilder,
        TaskGetJobLocation, TaskIsTargetEmpty, TaskJobAssign, TaskJobCancel, TaskJobComplete,
        TaskJobUnassign, TaskLookAt, TaskMoveTo, TaskPlaceBlock,
    },
    common::Distance,
    BlockType, Terrain,
};

#[derive(Component, Clone, Default)]
pub struct ScorerPlaceBlock {
    job: Option<Entity>,
    block_type: Option<BlockType>,
}

impl ScorerBuilder for ScorerPlaceBlock {
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
                                BehaviorNode::Task(Arc::new(TaskLookAt)),
                                BehaviorNode::Task(Arc::new(TaskPlaceBlock {
                                    progress: 0.,
                                    block_type: self.block_type.unwrap(),
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

pub fn score_place_block(
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    q_jobs: Query<
        (Entity, &Job, &JobLocation),
        (
            With<JobPlaceBlock>,
            With<IsJobAccessible>,
            Without<IsJobCancelled>,
            Without<IsJobCompleted>,
        ),
    >,
    q_items: Query<&Item>,
    q_free_items: Query<(&Item, &Transform), (Without<InInventory>, Without<InSlot>)>,
    q_actors: Query<
        (&Inventory, &Transform, &NavigationFlags),
        (With<Actor>, Without<HasBehavior>),
    >,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &mut ScorerPlaceBlock)>,
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
        let mut best_block_type = None;
        let mut best_dist = 100000.;

        for (e, job, job_location) in q_jobs.iter() {
            if job.assignee.is_some() {
                continue;
            }

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

                let JobType::PlaceBlock(block_type) = job.job_type else {
                    panic!("mismatch job type!");
                };

                best_block_type = Some(block_type);

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
        scorer.block_type = best_block_type;

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
