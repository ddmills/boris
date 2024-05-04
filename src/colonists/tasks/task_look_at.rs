use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Query, Res},
    },
    math::{vec3, Vec3},
    time::Time,
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{ActorRef, Blackboard, TaskBuilder, TaskState},
    ui::GameSpeed,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskLookAt;

pub fn task_look_at(
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut q_transforms: Query<&mut Transform>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &Blackboard), With<TaskLookAt>>,
) {
    for (ActorRef(actor), mut state, blackboard) in q_behavior.iter_mut() {
        let Some(primary_goal) = blackboard.primary_goal else {
            *state = TaskState::Failed;
            continue;
        };

        let Ok(mut transform) = q_transforms.get_mut(*actor) else {
            *state = TaskState::Failed;
            continue;
        };

        let target = vec3(
            primary_goal[0] as f32 + 0.5,
            primary_goal[1] as f32,
            primary_goal[2] as f32 + 0.5,
        );

        let target_rot = transform
            .looking_at(
                Vec3::new(target.x, transform.translation.y, target.z),
                Vec3::Y,
            )
            .rotation;

        transform.rotation = transform
            .rotation
            .slerp(target_rot, time.delta_seconds() * game_speed.speed() * 20.);

        if transform.rotation.angle_between(target_rot) < 0.1 {
            transform.rotation = target_rot;
            *state = TaskState::Success;
            continue;
        }
    }
}
