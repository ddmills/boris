use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, ResMut},
};

use super::{
    jobs::Job::Mine, Actor, ActorRef, Behavior, BehaviorNode, Fatigue, HasBehavior, ItemTag, Job,
    JobList, Path, TaskCheckHasItem, TaskDebug, TaskFindNearestItem, TaskGetJobLocation, TaskIdle,
    TaskMineBlock, TaskMoveTo, TaskPickRandomSpot, TaskPickUpItem, TaskReturnJob, TaskSetJob,
    TaskState,
};

#[derive(Component, Default)]
pub struct Blackboard {
    pub job: Option<Job>,
    pub bed: u8,
    pub move_goals: Vec<[u32; 3]>,
    pub item: Option<Entity>,
    pub path: Option<Path>,
}

pub fn behavior_pick_system(
    mut cmd: Commands,
    q_actors: Query<(Entity, &Fatigue), (With<Actor>, Without<HasBehavior>)>,
    mut jobs: ResMut<JobList>,
) {
    for (actor, fatigue) in q_actors.iter() {
        let behavior = get_behavior(fatigue, &mut jobs);

        println!("==== START {}", behavior.label);

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

pub fn get_behavior(fatigue: &Fatigue, jobs: &mut JobList) -> Behavior {
    // if fatigue.value > 75. {
    //     return Behavior::new(
    //         "Sleep",
    //         BehaviorNode::Sequence(vec![
    //             BehaviorNode::Task(Arc::new(TaskFindBed)),
    //             BehaviorNode::Task(Arc::new(TaskSleep)),
    //         ]),
    //     );
    // }

    if let Some(job) = jobs.pop() {
        return match job {
            Mine(pos) => Behavior::new(
                "Mine",
                BehaviorNode::Try(
                    Box::new(BehaviorNode::Sequence(vec![
                        BehaviorNode::Task(Arc::new(TaskSetJob(job))),
                        tree_aquire_item(vec![ItemTag::PickAxe]),
                        BehaviorNode::Sequence(vec![
                            BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                            BehaviorNode::Task(Arc::new(TaskMoveTo)),
                            BehaviorNode::Task(Arc::new(TaskMineBlock { pos, progress: 0. })),
                        ]),
                    ])),
                    Box::new(BehaviorNode::Task(Arc::new(TaskReturnJob))),
                ),
            ),
        };
    }

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

fn tree_aquire_item(tags: Vec<ItemTag>) -> BehaviorNode {
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
            BehaviorNode::Task(Arc::new(TaskDebug("We did found em!".to_string()))),
        ])),
    )
}
