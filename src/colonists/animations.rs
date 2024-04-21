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

#[derive(Resource)]
pub struct ColonistAnimations {
    pub base: Handle<AnimationClip>,
    pub run: Handle<AnimationClip>,
    pub idle: Handle<AnimationClip>,
    pub pick_up: Handle<AnimationClip>,
    pub swing_pick: Handle<AnimationClip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimClip {
    None,
    Run,
    Idle,
    PickUp,
    SwingPick,
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
            };

            let one_shot = match animator.clip {
                AnimClip::None => false,
                AnimClip::Run => false,
                AnimClip::Idle => false,
                AnimClip::SwingPick => false,
                AnimClip::PickUp => true,
            };

            animator.prev_clip = animator.clip;
            player.play_with_transition(clip, Duration::from_millis(160));

            if !one_shot {
                player.repeat();
            }
        }
    }
}
