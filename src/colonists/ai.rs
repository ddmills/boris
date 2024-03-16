use bevy::prelude::*;
use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    reflect::Reflect,
};
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
pub struct Sleep {
    pub duration: f32,
    pub per_second: f32,
}

pub fn sleep_action_system(
    time: Res<Time>,
    mut fatigues: Query<(&mut Fatigue, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Actor, &mut ActionState, &Sleep, &ActionSpan)>,
) {
    for (Actor(actor), mut state, sleep, span) in query.iter_mut() {
        let _guard = span.span().enter();

        let Ok((mut fatigue, material)) = fatigues.get_mut(*actor) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                println!("sleep requested");
                fatigue.is_sleeping = true;
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                fatigue.level -= sleep.per_second * time.delta_seconds();
                materials.get_mut(material).unwrap().base_color = Color::GOLD;

                if fatigue.level <= sleep.duration {
                    println!("done sleeping");
                    materials.get_mut(material).unwrap().base_color = Color::RED;
                    fatigue.is_sleeping = false;
                    *state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                println!("sleep cancelled");
                materials.get_mut(material).unwrap().base_color = Color::RED;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FatigueScorer;

pub fn fatigue_scorer_system(
    fatigues: Query<&Fatigue>,
    mut query: Query<(&Actor, &mut Score), With<FatigueScorer>>,
) {
    for (Actor(actor), mut score) in &mut query {
        let Ok(fatigue) = fatigues.get(*actor) else {
            continue;
        };

        if !fatigue.is_sleeping {
            let new_score = fatigue.level / 100.;
            score.set(new_score);

            if fatigue.level >= 80. {
                println!("Fatigue above 80")
            }
        }
    }
}
