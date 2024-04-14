use bevy::{
    ecs::{
        component::Component,
        event::EventWriter,
        system::{Query, Res, ResMut},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Blackboard, TaskBuilder, TaskState},
    common::Rand,
    items::SpawnStoneEvent,
    BlockType, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMineBlock {
    pub progress: f32,
}

pub fn task_mine_block(
    time: Res<Time>,
    mut terrain: ResMut<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &Blackboard, &mut TaskMineBlock)>,
    mut ev_spawn_stone: EventWriter<SpawnStoneEvent>,
    mut rand: ResMut<Rand>,
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
            terrain.set_block_type(x, y, z, BlockType::EMPTY);
            terrain.set_flag_mine(x, y, z, false);

            if rand.bool(0.35) {
                ev_spawn_stone.send(SpawnStoneEvent { pos: [x, y, z] });
            }

            *state = TaskState::Success;
            continue;
        }

        task.progress += time.delta_seconds();
    }
}
