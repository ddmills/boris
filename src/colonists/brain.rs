use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query},
};

pub trait ActionBuilder: Send + Sync {
    fn insert(&self, cmd: &mut Commands, entity: Entity);
    fn remove(&self, cmd: &mut Commands, entity: Entity);
    fn label(&self) -> String;
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum ActState {
    Executing,
    Success,
    Failed,
}

#[derive(Component, Clone)]
pub struct Brain {
    pub behavior: Entity,
    pub actions: Vec<Arc<dyn ActionBuilder>>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Actor(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
pub struct Behavior {
    pub idx: usize,
}

#[derive(Component)]
pub struct HasAct;

fn set_action<T: ActionBuilder>(builder: &T, cmd: &mut Commands, entity: Entity) {
    builder.insert(cmd, entity);
    cmd.entity(entity).insert(HasAct);
}

pub fn brain_pick_act(
    mut commands: Commands,
    mut q_behaviors: Query<(&Actor, &mut Behavior, &mut ActState)>,
    mut q_brains: Query<&Brain>,
) {
    for brain in q_brains.iter_mut() {
        let Ok((Actor(actor), mut behavior, mut state)) = q_behaviors.get_mut(brain.behavior)
        else {
            println!("Brain without behavior detected?");
            continue;
        };

        if behavior.idx >= brain.actions.len() {
            continue;
        }

        if *state == ActState::Executing {
            continue;
        }

        if behavior.idx > 0 {
            let cur_act = brain.actions.get(behavior.idx - 1).unwrap();
            println!(
                "removing action from actor {}->{}",
                behavior.idx - 1,
                cur_act.label()
            );
            cur_act.remove(&mut commands, brain.behavior);
        }

        let next_act = brain.actions.get(behavior.idx).unwrap();
        println!(
            "inserting action onto actor {}->{}",
            behavior.idx,
            next_act.label()
        );

        next_act.insert(&mut commands, brain.behavior);
        // commands.entity(*actor).insert(HasAct);
        *state = ActState::Executing;
        behavior.idx += 1;
    }
}

pub fn brain_system(
    mut commands: Commands,
    mut q_behaviors: Query<(&Actor, &mut Behavior, &mut ActState)>,
    q_brains: Query<&Brain, With<HasAct>>,
) {
    // for brain in q_brains.iter() {
    //     let Ok((Actor(actor), behavior, state)) = q_behaviors.get_mut(brain.behavior) else {
    //         println!("Brain without behavior detected?");
    //         continue;
    //     };

    //     if *state == ActState::Success || *state == ActState::Failed {
    //         let act_builder = brain.actions.get(behavior.idx - 1).unwrap();
    //         println!(
    //             "removing action from actor {}->{}",
    //             behavior.idx - 1,
    //             act_builder.label()
    //         );
    //         act_builder.remove(&mut commands, *actor);
    //         commands.entity(*actor).remove::<HasAct>();
    //     }
    // }
}
