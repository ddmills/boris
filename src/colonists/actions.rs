use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use super::{ActorRef, Fatigue, TaskBuilder, TaskState};

#[derive(Component)]
pub struct TaskFindBed;
impl TaskBuilder for TaskFindBed {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(TaskFindBed);
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<TaskFindBed>();
    }

    fn label(&self) -> String {
        String::from("ActFindBed")
    }
}

#[derive(Component)]
pub struct TaskSleep;
impl TaskBuilder for TaskSleep {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(TaskSleep);
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<TaskSleep>();
    }

    fn label(&self) -> String {
        String::from("ActSleep")
    }
}

#[derive(Component, Default)]
pub struct TaskIdle(pub f32);

impl TaskBuilder for TaskIdle {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(TaskIdle::default());
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<TaskIdle>();
    }

    fn label(&self) -> String {
        String::from("ActIdle")
    }
}

pub fn task_find_bed(mut q_actors: Query<(&ActorRef, &mut TaskState), With<TaskFindBed>>) {
    for (ActorRef(entity), mut state) in q_actors.iter_mut() {
        if *state == TaskState::Executing {
            println!("find a bed for {}", entity.index());
            // actor.blackboard.bed = 3;
            *state = TaskState::Success;
        }
    }
}

pub fn task_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_actors: Query<(&ActorRef, &mut TaskState), With<TaskSleep>>,
) {
    for (ActorRef(entity), mut state) in q_actors.iter_mut() {
        let Ok(mut fatigue) = q_fatigues.get_mut(*entity) else {
            println!("Actor entity does not have a fatigue");
            *state = TaskState::Failed;
            continue;
        };

        if *state == TaskState::Executing {
            if fatigue.value > 0. {
                fatigue.value -= time.delta_seconds() * 40.;
            }

            if fatigue.value <= 0. {
                fatigue.value = 0.;
                *state = TaskState::Success;
            }
        }
    }
}

pub fn task_idle(time: Res<Time>, mut q_actors: Query<(&mut TaskState, &mut TaskIdle)>) {
    for (mut state, mut idle) in q_actors.iter_mut() {
        if *state == TaskState::Executing {
            if idle.0 < 100. {
                idle.0 += time.delta_seconds() * 20.;
                *state = TaskState::Executing;
            } else {
                *state = TaskState::Success;
            }
        }
    }
}
