use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    query::{With, Without},
    system::{EntityCommands, Query},
};

use crate::colonists::{
    Actor, ActorRef, Behavior, BehaviorNode, HasBehavior, Score, ScorerBuilder, TaskIdle,
    TaskMoveTo, TaskPickRandomSpot,
};

#[derive(Component, Clone)]
pub struct ScorerWander;

impl ScorerBuilder for ScorerWander {
    fn insert(&self, cmd: &mut EntityCommands) {
        cmd.insert(self.clone());
    }

    fn label(&self) -> String {
        "Wander".to_string()
    }

    fn build(&self) -> Behavior {
        Behavior::new(
            "Wander",
            BehaviorNode::Sequence(vec![
                BehaviorNode::Task(Arc::new(TaskPickRandomSpot)),
                BehaviorNode::Task(Arc::new(TaskMoveTo::default())),
                BehaviorNode::Task(Arc::new(TaskIdle {
                    duration_s: 0.5,
                    progress: 0.,
                })),
            ]),
        )
    }
}

pub fn score_wander(
    q_actors: Query<&Actor, Without<HasBehavior>>,
    mut q_behaviors: Query<(&ActorRef, &mut Score), With<ScorerWander>>,
) {
    for (ActorRef(actor), mut score) in q_behaviors.iter_mut() {
        if q_actors.contains(*actor) {
            *score = Score(0.1);
            continue;
        }
        *score = Score(0.);
    }
}
