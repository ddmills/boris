use std::sync::Arc;

use bevy::ecs::{
    self,
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Query, Res},
};

use crate::colonists::{
    test_item_tags, tree_aquire_item, Actor, ActorRef, Behavior, BehaviorNode, InInventory,
    Inventory, IsJobAssigned, Item, ItemTag, Job, JobType, Score, ScorerBuilder, TaskAssignJob,
    TaskDebug, TaskGetJobLocation, TaskMineBlock, TaskMoveTo, TaskUnassignJob,
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
    q_jobs: Query<(Entity, &Job), Without<IsJobAssigned>>,
    q_items: Query<&Item>,
    q_free_items: Query<&Item, Without<InInventory>>,
    q_actors: Query<&Inventory, With<Actor>>,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &mut ScorerMine)>,
) {
    for (ActorRef(actor), mut score, mut scorer) in q_behaviors.iter_mut() {
        let Ok(inventory) = q_actors.get(*actor) else {
            *score = Score(0.);
            continue;
        };

        let mine_jobs = q_jobs.iter().filter(|(_, j)| match j.job_type {
            JobType::Mine(_) => true,
        });

        let mut best = None;

        for (e, job) in mine_jobs {
            match job.job_type {
                JobType::Mine([x, y, z]) => {
                    best = Some(e);
                }
            }
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
