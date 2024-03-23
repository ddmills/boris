use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res, ResMut},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{TaskBuilder, TaskState},
    Block, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMineBlock {
    pub pos: [u32; 3],
    pub progress: f32,
}

pub fn task_mine_block(
    time: Res<Time>,
    mut terrain: ResMut<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &mut TaskMineBlock)>,
) {
    for (mut state, mut task) in q_behavior.iter_mut() {
        let [x, y, z] = task.pos;

        if terrain.get_block(x, y, z).is_empty() {
            *state = TaskState::Success;
        }

        if task.progress >= 1. {
            terrain.set_block(x, y, z, Block::EMPTY);
            *state = TaskState::Success;
            continue;
        }

        task.progress += time.delta_seconds();
    }
}
