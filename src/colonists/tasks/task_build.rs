use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    time::Time,
};
use itertools::Itertools;
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, AnimClip, Animator, TaskBuilder, TaskState},
    furniture::{
        blueprint::{self, BlueprintMode},
        Blueprint,
    },
    ui::GameSpeed,
    Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskBuild {
    pub blueprint: Entity,
    pub progress: f32,
}

pub fn task_build(
    mut cmd: Commands,
    mut terrain: ResMut<Terrain>,
    mut q_animators: Query<&mut Animator, With<Actor>>,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut q_blueprints: Query<&mut Blueprint>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut TaskBuild)>,
) {
    for (ActorRef(actor), mut state, mut task) in q_behavior.iter_mut() {
        let Ok(mut blueprint) = q_blueprints.get_mut(task.blueprint) else {
            println!("entity does not have a blueprint! Cannot build.");
            *state = TaskState::Failed;
            continue;
        };

        if !blueprint.is_valid {
            println!("blueprint no longer valid! Cannot build.");
            *state = TaskState::Failed;
            continue;
        }

        if task.progress >= 6. {
            blueprint.mode = BlueprintMode::Built;
            blueprint.is_dirty = true;
            *state = TaskState::Success;

            for tile in blueprint.tiles.iter() {
                let [x, y, z] = tile.position;
                let [chunk_idx, _] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
                terrain.set_chunk_nav_dirty(chunk_idx, true);
            }

            continue;
        }

        if let Ok(mut animator) = q_animators.get_mut(*actor) {
            animator.clip = AnimClip::SwingHammer;
        };

        task.progress += time.delta_seconds() * game_speed.speed();
    }
}
