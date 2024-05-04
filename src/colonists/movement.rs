use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    math::{vec3, Vec3},
    time::Time,
    transform::components::Transform,
};

use crate::ui::GameSpeed;

#[derive(Component)]
pub struct BlockMove {
    pub speed: f32,
    pub target: [i32; 3],
    pub look_at: bool,
}

pub fn block_move_system(
    mut cmd: Commands,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut q_movers: Query<(Entity, &BlockMove, &mut Transform)>,
) {
    for (entity, block_move, mut transform) in q_movers.iter_mut() {
        let target = vec3(
            block_move.target[0] as f32 + 0.5,
            block_move.target[1] as f32,
            block_move.target[2] as f32 + 0.5,
        );

        let direction = (target - transform.translation).normalize();
        let distance = transform.translation.distance(target);
        let move_dist = time.delta_seconds() * block_move.speed * game_speed.speed();

        if distance < move_dist {
            transform.translation = target;
            cmd.entity(entity).remove::<BlockMove>();
        } else {
            transform.translation += direction * move_dist;
            if block_move.look_at {
                let target_rot = transform
                    .looking_at(
                        Vec3::new(target.x, transform.translation.y, target.z),
                        Vec3::Y,
                    )
                    .rotation;

                transform.rotation = transform
                    .rotation
                    .slerp(target_rot, time.delta_seconds() * game_speed.speed() * 20.);
            }
        }
    }
}
