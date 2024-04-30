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
    ui::GameSpeed,
    BlockType, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskPlaceBlock {
    pub progress: f32,
    pub block_type: BlockType,
}

pub fn task_place_block(
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut terrain: ResMut<Terrain>,
    mut q_behavior: Query<(&mut TaskState, &Blackboard, &mut TaskPlaceBlock)>,
    mut ev_destroy_item: EventWriter<DestroyItemEvent>,
) {
    for (mut state, blackboard, mut task) in q_behavior.iter_mut() {
        let Some([x, y, z]) = blackboard.target_block else {
            println!("Blackboard is missing target_block, cannot mine!");
            *state = TaskState::Failed;
            continue;
        };

        let current_block = terrain.get_block(x, y, z);

        if !current_block.is_empty() {
            *state = TaskState::Failed;
            continue;
        }

        if !current_block.flag_blueprint {
            println!("Block is not a blueprint and cannot be built!");
            *state = TaskState::Failed;
            continue;
        }

        if blackboard.item.is_none() {
            println!("Blackboard is missing item, cannot place!");
            *state = TaskState::Failed;
            continue;
        }

        if task.progress >= 1. {
            terrain.set_flag_blueprint(x, y, z, false);
            terrain.set_block_type(x, y, z, task.block_type);

            let item = blackboard.item.unwrap();
            ev_destroy_item.send(DestroyItemEvent { entity: item });

            *state = TaskState::Success;
            continue;
        }

        task.progress += time.delta_seconds() * game_speed.speed();
    }
}
