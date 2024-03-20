use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query},
};

use crate::colonists::TaskIdle;

use super::{Fatigue, TaskFindBed, TaskSleep};

pub trait TaskBuilder: Send + Sync {
    fn insert(&self, cmd: &mut Commands, entity: Entity);
    fn remove(&self, cmd: &mut Commands, entity: Entity);
    fn label(&self) -> String;
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum TaskState {
    Executing,
    Success,
    Failed,
}

#[derive(Component, Clone)]
pub struct Actor;

#[derive(Component, Clone)]
pub struct HasBehavior {
    pub behavior_entity: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ActorRef(pub Entity);

#[derive(Component, Clone)]
pub struct Behavior {
    pub label: String,
    pub idx: usize,
    pub tasks: Vec<Arc<dyn TaskBuilder>>,
}

#[derive(Component, Clone, Default)]
pub struct Blackboard {
    pub bed: u8,
    pub idle_time: f32,
}

pub fn behavior_system(
    mut commands: Commands,
    mut q_behaviors: Query<(Entity, &ActorRef, &mut Behavior, &mut TaskState)>,
    q_has_behavior: Query<&HasBehavior>,
) {
    for (entity, ActorRef(actor), mut behavior, mut state) in q_behaviors.iter_mut() {
        let Ok(has_behavior) = q_has_behavior.get(*actor) else {
            println!("Detached behavior detected?");
            continue;
        };

        if *state == TaskState::Executing {
            continue;
        }

        if behavior.idx >= behavior.tasks.len() {
            commands.entity(*actor).remove::<HasBehavior>();
            commands.entity(entity).despawn();
            continue;
        }

        if behavior.idx > 0 {
            let cur_task = behavior.tasks.get(behavior.idx - 1).unwrap();
            cur_task.remove(&mut commands, has_behavior.behavior_entity);
        }

        let next_task = behavior.tasks.get(behavior.idx).unwrap();

        println!("{}->{}", behavior.label, next_task.label());

        next_task.insert(&mut commands, has_behavior.behavior_entity);
        *state = TaskState::Executing;
        behavior.idx += 1;
    }
}

pub fn assign_behavior_system(
    mut commands: Commands,
    q_actors: Query<(Entity, &Fatigue), (With<Actor>, Without<HasBehavior>)>,
) {
    for (actor, fatigue) in q_actors.iter() {
        let behavior = if fatigue.value >= 75. {
            commands
                .spawn((
                    Behavior {
                        label: String::from("Sleep"),
                        idx: 0,
                        tasks: vec![Arc::new(TaskFindBed), Arc::new(TaskSleep)],
                    },
                    Blackboard::default(),
                    TaskState::Success,
                    ActorRef(actor),
                ))
                .id()
        } else {
            commands
                .spawn((
                    Behavior {
                        label: String::from("Idle"),
                        idx: 0,
                        tasks: vec![Arc::new(TaskIdle)],
                    },
                    Blackboard::default(),
                    TaskState::Success,
                    ActorRef(actor),
                ))
                .id()
        };

        commands.entity(actor).insert(HasBehavior {
            behavior_entity: behavior,
        });
    }
}
