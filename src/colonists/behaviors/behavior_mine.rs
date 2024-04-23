use std::sync::Arc;

use bevy::ecs::{
    self,
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Query, Res},
};

use crate::{
    colonists::{
        is_reachable, job_access_points, test_item_tags, tree_aquire_item, Actor, ActorRef,
        Behavior, BehaviorNode, HasBehavior, InInventory, Inventory, IsJobAccessible,
        IsJobCancelled, Item, ItemTag, Job, JobLocation, JobMine, NavigationFlags, NavigationGraph,
        PartitionPathRequest, Score, ScorerBuilder, TaskGetJobLocation, TaskItemEquip,
        TaskJobAssign, TaskJobComplete, TaskJobUnassign, TaskMineBlock, TaskMoveTo,
    },
    common::Distance,
    Position, Terrain,
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
                    BehaviorNode::Task(Arc::new(TaskJobAssign(self.job.unwrap()))),
                    tree_aquire_item(vec![ItemTag::Pickaxe]),
                    BehaviorNode::Task(Arc::new(TaskItemEquip)),
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                        BehaviorNode::Task(Arc::new(TaskMoveTo::default())),
                        BehaviorNode::Task(Arc::new(TaskMineBlock { progress: 0. })),
                        BehaviorNode::Task(Arc::new(TaskJobComplete)),
                    ]),
                ])),
                Box::new(BehaviorNode::Task(Arc::new(TaskJobUnassign))),
            ),
        )
    }
}

pub fn score_mine(
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    q_jobs: Query<
        (Entity, &Job, &JobLocation),
        (
            With<JobMine>,
            With<IsJobAccessible>,
            Without<IsJobCancelled>,
            Without<TaskJobComplete>,
        ),
    >,
    q_items: Query<&Item>,
    q_free_items: Query<(&Item, &Position), Without<InInventory>>,
    q_actors: Query<(&Inventory, &Position, &NavigationFlags), (With<Actor>, Without<HasBehavior>)>,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &mut ScorerMine)>,
) {
    for (ActorRef(actor), mut score, mut scorer) in q_behaviors.iter_mut() {
        let Ok((inventory, position, flags)) = q_actors.get(*actor) else {
            *score = Score(0.);
            continue;
        };

        let pos = [position.x, position.y, position.z];

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

        let item_tags = &[ItemTag::Pickaxe];

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
        if q_free_items.iter().any(|(i, p)| {
            test_item_tags(&i.tags, item_tags)
                && i.reserved.is_none()
                && is_reachable(
                    &PartitionPathRequest {
                        start: pos,
                        goals: vec![[p.x, p.y, p.z]],
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
