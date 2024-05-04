use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        system::{Query, Res},
    },
    time::Time,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, AnimClip, Animator, TaskBuilder, TaskState},
    structures::BuildStructureEvent,
    ui::GameSpeed,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskBuild {
    pub structure: Entity,
    pub progress: f32,
}

pub fn task_build(
    mut q_animators: Query<&mut Animator, With<Actor>>,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &mut TaskBuild)>,
    mut ev_build_structure: EventWriter<BuildStructureEvent>,
) {
    for (ActorRef(actor), mut state, mut task) in q_behavior.iter_mut() {
        if task.progress >= 6. {
            ev_build_structure.send(BuildStructureEvent {
                entity: task.structure,
            });
            *state = TaskState::Success;
            continue;
        }

        if let Ok(mut animator) = q_animators.get_mut(*actor) {
            animator.clip = AnimClip::SwingHammer;
        };

        task.progress += time.delta_seconds() * game_speed.speed();
    }
}
