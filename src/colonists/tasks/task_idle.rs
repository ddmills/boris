use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::colonists::{TaskBuilder, TaskState};

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

pub fn task_idle(time: Res<Time>, mut q_behavior: Query<(&mut TaskState, &mut TaskIdle)>) {
    for (mut state, mut task) in q_behavior.iter_mut() {
        if task.progress >= task.duration_s {
            *state = TaskState::Success;
            continue;
        }

        task.progress += time.delta_seconds();
    }
}
