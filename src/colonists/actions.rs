use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use super::{ActorRef, Blackboard, Fatigue, TaskBuilder, TaskState};

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
pub struct TaskIdle;

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

pub fn task_find_bed(
    mut q_actors: Query<(&ActorRef, &mut Blackboard, &mut TaskState), With<TaskFindBed>>,
) {
    for (ActorRef(entity), mut blackboard, mut state) in q_actors.iter_mut() {
        if *state == TaskState::Executing {
            println!("find a bed for {}", entity.index());
            blackboard.bed = 3;
            *state = TaskState::Success;
        }
    }
}

pub fn task_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_actors: Query<(&ActorRef, &Blackboard, &mut TaskState), With<TaskSleep>>,
) {
    for (ActorRef(entity), blackboard, mut state) in q_actors.iter_mut() {
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
                println!("slept in bed {}", blackboard.bed);
                fatigue.value = 0.;
                *state = TaskState::Success;
            }
        }
    }
}

pub fn task_idle(
    time: Res<Time>,
    mut q_actors: Query<(&mut TaskState, &mut Blackboard), With<TaskIdle>>,
) {
    for (mut state, mut blackboard) in q_actors.iter_mut() {
        if *state == TaskState::Executing {
            if blackboard.idle_time < 100. {
                blackboard.idle_time += time.delta_seconds() * 20.;
                *state = TaskState::Executing;
            } else {
                *state = TaskState::Success;
            }
        }
    }
}
