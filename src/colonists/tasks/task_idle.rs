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
    pub timer: f32,
    pub duration_s: f32,
}

pub fn task_idle(time: Res<Time>, mut q_behavior: Query<(&mut TaskState, &mut TaskIdle)>) {
    for (mut state, mut task) in q_behavior.iter_mut() {
        if task.timer >= task.duration_s {
            *state = TaskState::Success;
            continue;
        }

        task.timer += time.delta_seconds();
    }
}
