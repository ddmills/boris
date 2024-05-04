use std::time::Duration;

use bevy::{
    animation::{AnimationClip, AnimationPlayer},
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Query, Res, Resource},
    },
};

use crate::ui::GameSpeed;

#[derive(Resource)]
pub struct ColonistAnimations {
    pub base: Handle<AnimationClip>,
    pub run: Handle<AnimationClip>,
    pub idle: Handle<AnimationClip>,
    pub pick_up: Handle<AnimationClip>,
    pub swing_pick: Handle<AnimationClip>,
    pub swing_hammer: Handle<AnimationClip>,
    pub swing_axe: Handle<AnimationClip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimClip {
    None,
    Run,
    Idle,
    PickUp,
    SwingPick,
    SwingHammer,
    SwingAxe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimState {
    Playing,
    Completed,
}

#[derive(Component)]
pub struct Animator {
    pub clip: AnimClip,
    pub armature: Entity,
    pub prev_clip: AnimClip,
    pub state: AnimState,
}

pub fn colonist_animations(
    animations: Res<ColonistAnimations>,
    game_speed: Res<GameSpeed>,
    mut q_animators: Query<&mut Animator>,
    mut q_players: Query<&mut AnimationPlayer>,
) {
    for mut animator in q_animators.iter_mut() {
        let Ok(mut player) = q_players.get_mut(animator.armature) else {
            println!("Armature destroyed?");
            continue;
        };

        if player.is_finished() {
            animator.state = AnimState::Completed;
        } else {
            animator.state = AnimState::Playing;
        }

        if animator.clip != animator.prev_clip {
            let clip = match animator.clip {
                AnimClip::None => animations.base.clone_weak(),
                AnimClip::Run => animations.run.clone_weak(),
                AnimClip::Idle => animations.idle.clone_weak(),
                AnimClip::PickUp => animations.pick_up.clone_weak(),
                AnimClip::SwingPick => animations.swing_pick.clone_weak(),
                AnimClip::SwingHammer => animations.swing_hammer.clone_weak(),
                AnimClip::SwingAxe => animations.swing_axe.clone_weak(),
            };

            let one_shot = match animator.clip {
                AnimClip::None => false,
                AnimClip::Run => false,
                AnimClip::Idle => false,
                AnimClip::SwingPick => false,
                AnimClip::PickUp => true,
                AnimClip::SwingHammer => false,
                AnimClip::SwingAxe => false,
            };

            animator.prev_clip = animator.clip;
            if game_speed.speed() > 0. {
                player.play_with_transition(
                    clip,
                    Duration::from_millis((160. / game_speed.speed()) as u64),
                );
            }
            player.set_speed(game_speed.speed());

            if !one_shot {
                player.repeat();
            }
        }
    }
}
