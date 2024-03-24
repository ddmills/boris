use bevy::prelude::*;
use task_derive::TaskBuilder;

use crate::colonists::{TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskDebug(pub String);

pub fn task_debug(mut q_behavior: Query<(&mut TaskState, &TaskDebug)>) {
    for (mut state, task) in q_behavior.iter_mut() {
        println!("TaskDebug: {}", task.0);
        *state = TaskState::Success;
    }
}
