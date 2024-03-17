use bevy::prelude::*;
use bevy::{ecs::component::Component, reflect::Reflect};
use big_brain::prelude::*;

#[derive(Component, Debug, Reflect)]
pub struct Fatigue {
    pub is_sleeping: bool,
    pub per_second: f32,
    pub level: f32,
}

pub fn fatigue_system(time: Res<Time>, mut query: Query<&mut Fatigue>) {
    for mut fatigue in query.iter_mut() {
        if fatigue.is_sleeping {
            continue;
        }

        fatigue.level += fatigue.per_second * time.delta_seconds();
        if fatigue.level >= 100. {
            fatigue.level = 100.
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct SleepAct;

pub fn sleep_action_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q_agents: Query<(&mut Fatigue, &Handle<StandardMaterial>)>,
    mut q_ai: Query<(&Actor, &mut ActionState, &SleepAct)>,
) {
    for (Actor(actor), mut state, sleep) in q_ai.iter_mut() {
        let Ok((mut fatigue, material)) = q_agents.get_mut(*actor) else {
            println!("fatigue???");
            continue;
        };

        match *state {
            ActionState::Requested => {
                println!("sleep requested");
                fatigue.is_sleeping = true;
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                fatigue.level -= 10. * time.delta_seconds();
                fatigue.level = fatigue.level.clamp(0., 100.);
                materials.get_mut(material).unwrap().base_color = Color::GOLD;

                if fatigue.level <= 5. {
                    println!("done sleeping");
                    materials.get_mut(material).unwrap().base_color = Color::RED;
                    fatigue.is_sleeping = false;
                    *state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                println!("sleep cancelled");
                materials.get_mut(material).unwrap().base_color = Color::RED;
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FatigueScorer;

pub fn fatigue_scorer_system(
    q_agents: Query<&Fatigue>,
    mut q_ai: Query<(&Actor, &mut Score), With<FatigueScorer>>,
) {
    for (Actor(actor), mut score) in &mut q_ai {
        let Ok(fatigue) = q_agents.get(*actor) else {
            continue;
        };

        if !fatigue.is_sleeping {
            let new_score = fatigue.level / 100.;
            score.set(new_score.clamp(0., 1.));
        }
    }
}
