use std::sync::Arc;

use bevy::{
    ecs::{
        self,
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    transform::components::Transform,
};

use crate::{
    colonists::{
        is_reachable, test_item_tags, tree_aquire_item, Actor, ActorRef, Behavior, BehaviorNode,
        HasBehavior, InInventory, Inventory, IsJobAccessible, Item, ItemTag, Job, JobLocation,
        JobMine, NavigationFlags, NavigationGraph, PartitionPathRequest, Score, ScorerBuilder,
        TaskAssignJob, TaskGetJobLocation, TaskMineBlock, TaskMoveTo, TaskUnassignJob,
    },
    common::Distance,
    Terrain,
};

#[derive(Component, Clone, Default)]
pub struct ScorerMine {
    job: Option<Entity>,
}

impl ScorerBuilder for ScorerMine {
    fn insert(&self, cmd: &mut ecs::system::EntityCommands) {
        cmd.insert(self.clone());
    }

    fn label(&self) -> String {
        "Mine".to_string()
    }

    fn build(&self) -> Behavior {
        Behavior::new(
            "Mine",
            BehaviorNode::Try(
                Box::new(BehaviorNode::Sequence(vec![
                    BehaviorNode::Task(Arc::new(TaskAssignJob(self.job.unwrap()))),
                    tree_aquire_item(vec![ItemTag::Pickaxe]),
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                        BehaviorNode::Task(Arc::new(TaskMoveTo)),
                        BehaviorNode::Task(Arc::new(TaskMineBlock { progress: 0. })),
                    ]),
                ])),
                Box::new(BehaviorNode::Task(Arc::new(TaskUnassignJob))),
            ),
        )
    }
}

pub fn score_mine(
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    q_jobs: Query<(Entity, &Job, &JobLocation), (With<JobMine>, With<IsJobAccessible>)>,
    q_items: Query<&Item>,
    q_free_items: Query<(&Item, &Transform), Without<InInventory>>,
    q_actors: Query<(&Inventory, &Transform), (With<Actor>, Without<HasBehavior>)>,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &mut ScorerMine)>,
) {
    for (ActorRef(actor), mut score, mut scorer) in q_behaviors.iter_mut() {
        let Ok((inventory, transform)) = q_actors.get(*actor) else {
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

            let [x, y, z] = job_location.pos;
            let goals = vec![
                [x + 1, y, z],
                [x - 1, y, z],
                [x, y, z + 1],
                [x, y, z - 1],
                [x + 1, y + 1, z],
                [x - 1, y + 1, z],
                [x, y + 1, z + 1],
                [x, y + 1, z - 1],
                [x + 1, y - 1, z],
                [x - 1, y - 1, z],
                [x, y - 1, z + 1],
                [x, y - 1, z - 1],
                [x, y - 2, z + 1],
                [x, y - 2, z - 1],
                [x - 1, y, z + 1],
                [x - 1, y, z - 1],
                [x + 1, y, z + 1],
                [x + 1, y, z - 1],
            ];
            let request = PartitionPathRequest {
                start: pos,
                goals,
                flags: NavigationFlags::COLONIST,
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

        let item_tags = &vec![ItemTag::Pickaxe];

        let has_pickaxe = inventory.items.iter().any(|e| {
            let Ok(item) = q_items.get(*e) else {
                return false;
            };

            test_item_tags(&item.tags, item_tags)
        });

        // if we have a pickaxe, score is higher
        if has_pickaxe {
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
                        flags: NavigationFlags::COLONIST,
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

pub fn mine_job_checker(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    q_jobs: Query<(Entity, &Job, &JobLocation), With<JobMine>>,
) {
    for (entity, job, job_location) in q_jobs.iter() {
        if job.assignee.is_some() {
            continue;
        }

        let [x, y, z] = job_location.pos;
        let goals = vec![
            [x + 1, y, z],
            [x - 1, y, z],
            [x, y, z + 1],
            [x, y, z - 1],
            [x + 1, y + 1, z],
            [x - 1, y + 1, z],
            [x, y + 1, z + 1],
            [x, y + 1, z - 1],
            [x + 1, y - 1, z],
            [x - 1, y - 1, z],
            [x, y - 1, z + 1],
            [x, y - 1, z - 1],
            [x, y - 2, z + 1],
            [x, y - 2, z - 1],
            [x - 1, y, z + 1],
            [x - 1, y, z - 1],
            [x + 1, y, z + 1],
            [x + 1, y, z - 1],
        ];

        let is_accessible = goals
            .iter()
            .any(|g| terrain.get_partition_id_u32(g[0], g[1], g[2]).is_some());

        if is_accessible {
            cmd.entity(entity).insert(IsJobAccessible);
        } else {
            cmd.entity(entity).remove::<IsJobAccessible>();
        }
    }
}
