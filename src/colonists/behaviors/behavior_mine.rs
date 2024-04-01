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
        get_partition_path, partition, test_item_tags, tree_aquire_item, Actor, ActorRef, Behavior,
        BehaviorNode, InInventory, Inventory, IsJobAccessible, Item, ItemTag, Job, JobLocation,
        JobMine, NavigationFlags, Partition, PartitionGraph, PartitionPathRequest, Score,
        ScorerBuilder, TaskAssignJob, TaskDebug, TaskGetJobLocation, TaskMineBlock, TaskMoveTo,
        TaskUnassignJob,
    },
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
                    BehaviorNode::Task(Arc::new(TaskDebug("Mining task!".to_string()))),
                    BehaviorNode::Task(Arc::new(TaskAssignJob(self.job.unwrap()))),
                    tree_aquire_item(vec![ItemTag::PickAxe]),
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
    graph: Res<PartitionGraph>,
    q_jobs: Query<(Entity, &Job, &JobLocation), (With<JobMine>, With<IsJobAccessible>)>,
    q_items: Query<&Item>,
    q_free_items: Query<&Item, Without<InInventory>>,
    q_actors: Query<(&Inventory, &Transform), With<Actor>>,
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

            let start_partition_id = terrain.get_partition_id_u32(pos[0], pos[1], pos[2]);
            let Some(start_group_id) =
                graph.get_navigation_group_id(start_partition_id, NavigationFlags::COLONIST)
            else {
                continue;
            };

            let mut visited_partitions = vec![];
            for goal in goals {
                let partition_id = terrain.get_partition_id_u32(goal[0], goal[1], goal[2]);
                if visited_partitions.contains(&partition_id) {
                    continue;
                }

                visited_partitions.push(partition_id);

                let Some(group_id) =
                    graph.get_navigation_group_id(partition_id, NavigationFlags::COLONIST)
                else {
                    continue;
                };

                if group_id == start_group_id {
                    best = Some(e);
                    break;
                }
            }

            // let request = PartitionPathRequest {
            //     start: pos,
            //     goals,
            //     flags: NavigationFlags::TALL | NavigationFlags::LADDER,
            // };

            // generate path
            // this could be cached on the behavior
            // let Some(partition_path) = get_partition_path(&request, &terrain, &graph) else {
            //     // job is not reachable. we should cooldown this score checker
            //     continue;
            // };
        }

        if best.is_none() {
            *score = Score(0.);
            continue;
        };

        scorer.job = best;

        let has_pickaxe = inventory.items.iter().any(|e| {
            let Ok(item) = q_items.get(*e) else {
                return false;
            };

            test_item_tags(&item.tags, &vec![ItemTag::PickAxe])
        });

        // if we have a pickaxe, score is higher
        if has_pickaxe {
            *score = Score(0.6);
            continue;
        }

        if q_free_items.is_empty() {
            *score = Score(0.0);
            continue;
        }

        *score = Score(0.2);
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
            .any(|g| terrain.get_partition_id_u32(g[0], g[1], g[2]) != Partition::NONE);

        if is_accessible {
            cmd.entity(entity).insert(IsJobAccessible);
        } else {
            cmd.entity(entity).remove::<IsJobAccessible>();
        }
    }
}
