use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{AnyOf, With, Without},
    system::{Commands, Query, ResMut},
};
use bevy_trait_query::One;

use super::{
    Actor, ActorRef, Behavior, BehaviorNode, Fatigue, HasBehavior, ItemTag, Path, Score,
    ScoreBuilderRef, ScorerBuilder, ScorerMine, ScorerWander, Scorers, TaskAssignJob,
    TaskCheckHasItem, TaskFindNearestItem, TaskGetJobLocation, TaskIdle, TaskMineBlock, TaskMoveTo,
    TaskPickRandomSpot, TaskPickUpItem, TaskState, TaskUnassignJob, Thinker,
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
    // q_scores: Query<(&Score, One<&dyn ScorerBuilder>)>,
    q_scores: Query<(&Score, AnyOf<(&ScorerWander, &ScorerMine)>)>,
    q_actors: Query<(Entity, &Scorers, &Thinker), (With<Actor>, Without<HasBehavior>)>,
) {
    for (actor, scorers, thinker) in q_actors.iter() {
        // let behavior = get_behavior(fatigue, &mut jobs);

        println!("pick behavior");

        let mut high_score = 0.;
        // let mut high_score_builder: Option<ScorerBuilder> = None;
        let mut high_score_builder: Option<Box<&dyn ScorerBuilder>> = None;
        // let mut high_score_builder: ScorerBuilder;

        for scorer in scorers.scorers.iter() {
            let Ok((score, builder)) = q_scores.get(*scorer) else {
                println!("missing score?");
                continue;
            };

            if score.0 > high_score {
                high_score = score.0;
                // high_score_builder = Some(builder);
                if builder.0.is_some() {
                    high_score_builder = Some(Box::new(builder.0.unwrap()));
                } else if builder.1.is_some() {
                    high_score_builder = Some(Box::new(builder.1.unwrap()));
                }
            }
        }

        if high_score == 0. {
            println!("no high score. missing behaviors?");
            continue;
        }

        // let builder = thinker.score_builders.get(high_scorer_idx).unwrap();
        let builder = high_score_builder.unwrap();

        println!("==== START {}, Score({})", builder.label(), high_score);
        // println!("pick behavior");

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

pub fn get_behavior(fatigue: &Fatigue) -> Behavior {
    // if fatigue.value > 75. {
    //     return Behavior::new(
    //         "Sleep",
    //         BehaviorNode::Sequence(vec![
    //             BehaviorNode::Task(Arc::new(TaskFindBed)),
    //             BehaviorNode::Task(Arc::new(TaskSleep)),
    //         ]),
    //     );
    // }

    // if let Some(job) = jobs.remove() {
    //     return match job {
    //         Mine(pos) => Behavior::new(
    //             "Mine",
    //             BehaviorNode::Try(
    //                 Box::new(BehaviorNode::Sequence(vec![
    //                     BehaviorNode::Task(Arc::new(TaskSetJob(job))),
    //                     tree_aquire_item(vec![ItemTag::PickAxe]),
    //                     BehaviorNode::Sequence(vec![
    //                         BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
    //                         BehaviorNode::Task(Arc::new(TaskMoveTo)),
    //                         BehaviorNode::Task(Arc::new(TaskMineBlock { pos, progress: 0. })),
    //                     ]),
    //                 ])),
    //                 Box::new(BehaviorNode::Task(Arc::new(TaskReturnJob))),
    //             ),
    //         ),
    //     };
    // }

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
