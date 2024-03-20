use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use super::{ActState, ActionBuilder, ActorRef, Fatigue};

#[derive(Component)]
pub struct ActFindBed;
impl ActionBuilder for ActFindBed {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(ActFindBed);
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<ActFindBed>();
    }

    fn label(&self) -> String {
        String::from("ActFindBed")
    }
}

#[derive(Component)]
pub struct ActSleep;
impl ActionBuilder for ActSleep {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(ActSleep);
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<ActSleep>();
    }

    fn label(&self) -> String {
        String::from("ActSleep")
    }
}

#[derive(Component, Default)]
pub struct ActIdle(pub f32);

impl ActionBuilder for ActIdle {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(ActIdle::default());
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<ActIdle>();
    }

    fn label(&self) -> String {
        String::from("ActIdle")
    }
}

pub fn act_find_bed(mut q_actors: Query<(&ActorRef, &mut ActState), With<ActFindBed>>) {
    for (ActorRef(entity), mut state) in q_actors.iter_mut() {
        if *state == ActState::Executing {
            println!("find a bed for {}", entity.index());
            // actor.blackboard.bed = 3;
            *state = ActState::Success;
        }
    }
}

pub fn act_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_actors: Query<(&ActorRef, &mut ActState), With<ActSleep>>,
) {
    for (ActorRef(entity), mut state) in q_actors.iter_mut() {
        let Ok(mut fatigue) = q_fatigues.get_mut(*entity) else {
            println!("Actor entity does not have a fatigue");
            *state = ActState::Failed;
            continue;
        };

        if *state == ActState::Executing {
            if fatigue.value > 0. {
                fatigue.value -= time.delta_seconds() * 40.;
            }

            if fatigue.value <= 0. {
                fatigue.value = 0.;
                *state = ActState::Success;
            }
        }
    }
}

pub fn act_idle(time: Res<Time>, mut q_actors: Query<(&mut ActState, &mut ActIdle)>) {
    for (mut state, mut idle) in q_actors.iter_mut() {
        if *state == ActState::Executing {
            if idle.0 < 100. {
                idle.0 += time.delta_seconds() * 20.;
                *state = ActState::Executing;
            } else {
                *state = ActState::Success;
            }
        }
    }
}
