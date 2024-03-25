use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res, ResMut},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Blackboard, TaskBuilder, TaskState},
    Block, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMineBlock {
    pub progress: f32,
}

pub fn task_mine_block(
    time: Res<Time>,
    mut terrain: ResMut<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &Blackboard, &mut TaskMineBlock)>,
) {
    for (mut state, blackboard, mut task) in q_behavior.iter_mut() {
        let Some([x, y, z]) = blackboard.target_block else {
            println!("Blackboard is missing target_block, cannot mine!");
            *state = TaskState::Failed;
            continue;
        };

        if terrain.get_block(x, y, z).is_empty() {
            *state = TaskState::Success;
            continue;
        }

        if task.progress >= 1. {
            terrain.set_block(x, y, z, Block::EMPTY);
            *state = TaskState::Success;
            continue;
        }

        task.progress += time.delta_seconds();
    }
}
