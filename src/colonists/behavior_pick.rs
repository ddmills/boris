use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query},
};
use bevy_trait_query::One;

use super::{
    Actor, ActorRef, BehaviorNode, HasBehavior, ItemTag, Path, Score, ScorerBuilder, Scorers,
    TaskCheckHasItem, TaskFindNearestItem, TaskIdle, TaskMoveTo, TaskPickUpItem, TaskState,
};

#[derive(Component, Default)]
pub struct Blackboard {
    pub job: Option<Entity>,
    pub bed: u8,
    pub move_goals: Vec<[u32; 3]>,
    pub item: Option<Entity>,
    pub path: Option<Path>,
    pub target_block: Option<[u32; 3]>,
}

pub fn behavior_pick_system(
    mut cmd: Commands,
    q_scores: Query<(&Score, One<&dyn ScorerBuilder>)>,
    q_actors: Query<(Entity, &Scorers), (With<Actor>, Without<HasBehavior>)>,
) {
    for (actor, scorers) in q_actors.iter() {
        let mut high_score = 0.;
        let mut high_score_builder = None;

        for scorer in scorers.scorers.iter() {
            let Ok((score, builder)) = q_scores.get(*scorer) else {
                println!("missing score?");
                continue;
            };

            if score.0 > high_score {
                high_score = score.0;
                high_score_builder = Some(builder);
            }
        }

        if high_score == 0. || high_score_builder.is_none() {
            println!("no high score. missing behaviors?");
            continue;
        }

        let builder = high_score_builder.unwrap();
        let behavior = builder.build();
        let b_entity = cmd
            .spawn((
                Blackboard::default(),
                TaskState::Success,
                ActorRef(actor),
                behavior,
            ))
            .id();

        cmd.entity(actor).insert(HasBehavior {
            behavior_entity: b_entity,
        });
    }
}

pub fn tree_aquire_item(tags: Vec<ItemTag>) -> BehaviorNode {
    BehaviorNode::Try(
        Box::new(BehaviorNode::Task(Arc::new(TaskCheckHasItem(tags.clone())))),
        Box::new(BehaviorNode::Sequence(vec![
            BehaviorNode::Task(Arc::new(TaskFindNearestItem(tags))),
            BehaviorNode::Task(Arc::new(TaskMoveTo)),
            BehaviorNode::Task(Arc::new(TaskPickUpItem)),
            BehaviorNode::Task(Arc::new(TaskIdle {
                duration_s: 0.5,
                progress: 0.,
            })),
        ])),
    )
}
