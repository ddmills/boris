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
    colonists::{Blackboard, DestroyItemEvent, TaskBuilder, TaskState},
    BlockType, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskBuildBlock {
    pub progress: f32,
    pub block: BlockType,
}

pub fn task_build_block(
    time: Res<Time>,
    mut terrain: ResMut<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &Blackboard, &mut TaskBuildBlock)>,
    mut ev_destroy_item: EventWriter<DestroyItemEvent>,
) {
    for (mut state, blackboard, mut task) in q_behavior.iter_mut() {
        let Some([x, y, z]) = blackboard.target_block else {
            println!("Blackboard is missing target_block, cannot mine!");
            *state = TaskState::Failed;
            continue;
        };

        if !terrain.get_block_type(x, y, z).is_empty() {
            *state = TaskState::Failed;
            continue;
        }

        if blackboard.item.is_none() {
            println!("Blackboard is missing item, cannot place!");
            *state = TaskState::Failed;
            continue;
        }

        if task.progress >= 1. {
            terrain.set_block_type(x, y, z, task.block);

            let item = blackboard.item.unwrap();
            ev_destroy_item.send(DestroyItemEvent { entity: item });

            *state = TaskState::Success;
            continue;
        }

        task.progress += time.delta_seconds();
    }
}
