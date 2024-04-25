use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, AnimClip, Animator, Blackboard, TaskBuilder, TaskState},
    common::Rand,
    items::SpawnStoneEvent,
    ui::GameSpeed,
    BlockType, Terrain, Tree,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskChopTree {
    pub progress: f32,
    pub tree: Entity,
}

pub fn task_chop_tree(
    mut cmd: Commands,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut terrain: ResMut<Terrain>,
    mut q_animators: Query<&mut Animator, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut TaskChopTree)>,
    q_trees: Query<&Tree>,
    mut rand: ResMut<Rand>,
) {
    for (ActorRef(actor), mut state, mut task) in q_behavior.iter_mut() {
        let Ok(tree) = q_trees.get(task.tree) else {
            *state = TaskState::Failed;
            continue;
        };

        if task.progress >= 6. {
            for part in tree.canopy.iter() {
                let [chunk_idx, block_idx] = terrain.get_block_indexes(part[0], part[1], part[2]);
                terrain.remove_tree(chunk_idx, block_idx, &task.tree);

                let other_trees = terrain.get_trees(chunk_idx, block_idx);

                if other_trees.is_empty() {
                    let block = terrain.get_block_by_idx(chunk_idx, block_idx);
                    if block.block == BlockType::LEAVES {
                        terrain.set_block_type(part[0], part[1], part[2], BlockType::EMPTY);
                    }
                }
            }

            for part in tree.trunk.iter() {
                let [chunk_idx, block_idx] = terrain.get_block_indexes(part[0], part[1], part[2]);
                terrain.remove_tree(chunk_idx, block_idx, &task.tree);

                let other_trees = terrain.get_trees(chunk_idx, block_idx);

                if other_trees.is_empty() {
                    let block = terrain.get_block_by_idx(chunk_idx, block_idx);
                    if block.block == BlockType::TREE_TRUNK {
                        terrain.set_block_type(part[0], part[1], part[2], BlockType::EMPTY);
                    }
                }
            }

            if rand.bool(0.25) {
                // ev_spawn_stone.send(SpawnStoneEvent { pos: [x, y, z] });
            }

            cmd.entity(task.tree).despawn_recursive();
            *state = TaskState::Success;
            continue;
        }

        if let Ok(mut animator) = q_animators.get_mut(*actor) {
            animator.clip = AnimClip::SwingPick;
        };

        task.progress += time.delta_seconds() * game_speed.speed();
    }
}
