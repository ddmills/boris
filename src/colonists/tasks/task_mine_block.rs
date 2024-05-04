use bevy::{
    ecs::{
        component::Component,
        event::EventWriter,
        query::With,
        system::{Query, Res, ResMut},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, AnimClip, Animator, Blackboard, TaskBuilder, TaskState},
    common::Rand,
    items::{Commodity::StoneShaleBoulder, SpawnCommodityEvent},
    ui::GameSpeed,
    BlockType, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMineBlock {
    pub progress: f32,
}

pub fn task_mine_block(
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut terrain: ResMut<Terrain>,
    mut q_animators: Query<&mut Animator, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &Blackboard, &mut TaskMineBlock)>,
    mut ev_spawn_commodity: EventWriter<SpawnCommodityEvent>,
    mut rand: ResMut<Rand>,
) {
    for (ActorRef(actor), mut state, blackboard, mut task) in q_behavior.iter_mut() {
        let Some([x, y, z]) = blackboard.target_block else {
            println!("Blackboard is missing target_block, cannot mine!");
            *state = TaskState::Failed;
            continue;
        };

        if terrain.get_block(x, y, z).is_empty() {
            *state = TaskState::Success;
            continue;
        }

        if task.progress >= 3. {
            terrain.set_block_type(x, y, z, BlockType::EMPTY);
            terrain.set_flag_mine(x, y, z, false);

            if rand.bool(0.15) {
                ev_spawn_commodity.send(SpawnCommodityEvent {
                    commodity: StoneShaleBoulder,
                    position: [x, y, z],
                });
            }

            *state = TaskState::Success;
            continue;
        }

        if let Ok(mut animator) = q_animators.get_mut(*actor) {
            animator.clip = AnimClip::SwingPick;
        };

        task.progress += time.delta_seconds() * game_speed.speed();
    }
}
