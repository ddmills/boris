use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{ActorRef, TaskBuilder, TaskState},
    ui::GameSpeed,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskBuild {
    pub blueprint: Entity,
    pub progress: f32,
}

pub fn task_build(
    mut cmd: Commands,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut TaskBuild)>,
) {
    for (ActorRef(actor), mut state, mut task) in q_behavior.iter_mut() {
        *state = TaskState::Success;
    }
}
