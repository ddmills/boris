use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Query, Res},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, AnimClip, Animator, TaskBuilder, TaskState},
    ui::GameSpeed,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskIdle {
    pub progress: f32,
    pub duration_s: f32,
}

impl Default for TaskIdle {
    fn default() -> Self {
        Self {
            progress: 0.,
            duration_s: 0.5,
        }
    }
}

pub fn task_idle(
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut q_animators: Query<&mut Animator, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut TaskIdle)>,
) {
    for (ActorRef(actor), mut state, mut task) in q_behavior.iter_mut() {
        if task.progress >= task.duration_s {
            *state = TaskState::Success;
            continue;
        }

        if let Ok(mut animator) = q_animators.get_mut(*actor) {
            animator.clip = AnimClip::Idle;
        };

        task.progress += time.delta_seconds() * game_speed.speed();
    }
}
