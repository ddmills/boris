use bevy::ecs::{
    component::Component,
    system::{Query, ResMut},
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{TaskBuilder, TaskState},
    Block, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMineBlock(pub [u32; 3]);

pub fn task_mine_block(
    mut terrain: ResMut<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &TaskMineBlock)>,
) {
    for (mut state, task) in q_behavior.iter_mut() {
        let [x, y, z] = task.0;
        terrain.set_block(x, y, z, Block::EMPTY);
        *state = TaskState::Success;
    }
}
