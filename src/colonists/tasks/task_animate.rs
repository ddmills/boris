use bevy::ecs::{component::Component, query::With, system::Query};
use task_derive::TaskBuilder;

use crate::colonists::{Actor, ActorRef, AnimClip, AnimState, Animator, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskAnimate(pub AnimClip);

pub fn task_animate(
    mut q_animators: Query<&mut Animator, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &TaskAnimate)>,
) {
    for (ActorRef(actor), mut state, task) in q_behavior.iter_mut() {
        let Ok(mut animator) = q_animators.get_mut(*actor) else {
            println!("Cannot play animation, actor does not have Animator");
            *state = TaskState::Failed;
            continue;
        };

        if animator.clip != task.0 {
            animator.clip = task.0;
            *state = TaskState::Executing;
            continue;
        }

        if animator.state == AnimState::Completed {
            *state = TaskState::Success;
            continue;
        }
    }
}
