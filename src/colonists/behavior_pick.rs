use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, ResMut},
};

use super::{
    jobs::Job::Mine, Actor, ActorRef, Behavior, BehaviorNode, Fatigue, HasBehavior, Job, JobList,
    Path, TaskFindBed, TaskGetJobLocation, TaskIdle, TaskMineBlock, TaskMoveTo, TaskPickRandomSpot,
    TaskReturnJob, TaskSetJob, TaskSleep, TaskState,
};

#[derive(Component, Default)]
pub struct Blackboard {
    pub job: Option<Job>,
    pub bed: u8,
    pub move_goals: Vec<[u32; 3]>,
    pub path: Option<Path>,
}

pub fn behavior_pick_system(
    mut commands: Commands,
    q_actors: Query<(Entity, &Fatigue), (With<Actor>, Without<HasBehavior>)>,
    mut jobs: ResMut<JobList>,
) {
    for (actor, fatigue) in q_actors.iter() {
        let behavior = get_behavior(fatigue, &mut jobs);

        let b_entity = commands
            .spawn((
                Blackboard::default(),
                TaskState::Success,
                ActorRef(actor),
                behavior,
            ))
            .id();

        commands.entity(actor).insert(HasBehavior {
            behavior_entity: b_entity,
        });
    }
}

pub fn get_behavior(fatigue: &Fatigue, jobs: &mut JobList) -> Behavior {
    if fatigue.value > 75. {
        return Behavior::new(
            "Sleep",
            BehaviorNode::Sequence(vec![
                BehaviorNode::Task(Arc::new(TaskFindBed)),
                BehaviorNode::Task(Arc::new(TaskSleep)),
            ]),
        );
    }

    if let Some(job) = jobs.pop() {
        return match job {
            Mine(pos) => Behavior::new(
                "Mine",
                BehaviorNode::Try(
                    Box::new(BehaviorNode::Sequence(vec![
                        BehaviorNode::Task(Arc::new(TaskSetJob(job))),
                        BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                        BehaviorNode::Task(Arc::new(TaskMoveTo)),
                        BehaviorNode::Task(Arc::new(TaskMineBlock { pos, progress: 0. })),
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
                duration_s: 2.,
                progress: 0.,
            })),
        ]),
    )
}
