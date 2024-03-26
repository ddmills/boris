use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    system::{EntityCommands, Query},
};

use crate::colonists::{
    Actor, ActorRef, Behavior, BehaviorNode, Score, ScorerBuilder, TaskIdle, TaskMoveTo,
    TaskPickRandomSpot,
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
                BehaviorNode::Task(Arc::new(TaskMoveTo)),
                BehaviorNode::Task(Arc::new(TaskIdle {
                    duration_s: 1.,
                    progress: 0.,
                })),
            ]),
        )
    }
}

pub fn score_wander(
    q_actors: Query<&Actor>,
    mut q_behaviors: Query<(&ActorRef, &mut Score, &ScorerWander)>,
) {
    for (ActorRef(actor), mut score, wander) in q_behaviors.iter_mut() {
        // let Ok()
        *score = Score(0.1);
    }
}