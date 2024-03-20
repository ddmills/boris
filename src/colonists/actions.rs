use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use super::{ActState, ActionBuilder, Actor, Fatigue};

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

#[derive(Component)]
pub struct ActNone;
impl ActionBuilder for ActNone {
    fn insert(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).insert(ActNone);
    }

    fn remove(&self, cmd: &mut Commands, entity: Entity) {
        cmd.entity(entity).remove::<ActNone>();
    }

    fn label(&self) -> String {
        String::from("ActNone")
    }
}

pub fn act_find_bed(mut q_actors: Query<(&Actor, &mut ActState), With<ActFindBed>>) {
    for (Actor(entity), mut state) in q_actors.iter_mut() {
        if *state == ActState::Executing {
            println!("find a bed for {}", entity.index());
            // brain.blackboard.bed = 3;
            *state = ActState::Success;
        }
    }
}

pub fn act_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_actors: Query<(&Actor, &mut ActState), With<ActSleep>>,
) {
    for (Actor(entity), mut state) in q_actors.iter_mut() {
        let Ok(mut fatigue) = q_fatigues.get_mut(*entity) else {
            println!("Actor entity does not have a fatigue");
            *state = ActState::Failed;
            continue;
        };

        if *state == ActState::Executing {
            // println!("sleeping in bed {}", brain.blackboard.bed);

            if fatigue.value > 0. {
                fatigue.value -= time.delta_seconds() * 40.;
            }

            if fatigue.value <= 0. {
                fatigue.value = 0.;
                println!("sleeping successful");
                *state = ActState::Success;
            }
        }
    }
}
