use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Query, Res},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::colonists::{ActorRef, Blackboard, Fatigue, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskSleep;

pub fn task_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_behavior: Query<(&ActorRef, &Blackboard, &mut TaskState), With<TaskSleep>>,
) {
    for (ActorRef(entity), blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(mut fatigue) = q_fatigues.get_mut(*entity) else {
            println!("Actor entity does not have a fatigue");
            *state = TaskState::Failed;
            continue;
        };

        if fatigue.value > 0. {
            fatigue.value -= time.delta_seconds() * 40.;
        }

        if fatigue.value <= 0. {
            println!("slept in bed {}", blackboard.bed);
            fatigue.value = 0.;
            *state = TaskState::Success;
        }
    }
}
