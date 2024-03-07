use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res},
    },
    gizmos::gizmos::Gizmos,
    math::Vec3,
    render::color::Color,
    time::Time,
    transform::components::Transform,
};

use crate::{
    common::{astar, AStarSettings, Distance},
    Block, Terrain,
};

use super::{Colonist, PathfindEvent};

#[derive(Component)]
pub struct NeedsPath {
    start: [u32; 3],
    goal: [u32; 3],
}

#[derive(Component)]
pub struct Path {
    path: Vec<[i32; 3]>,
    current: usize,
}

pub fn path_follow(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut pathers: Query<(Entity, &mut Path, &mut Transform)>,
) {
    for (entity, mut pather, mut transform) in pathers.iter_mut() {
        let next_pos = pather.path[pather.current];
        let target = Vec3::new(next_pos[0] as f32, next_pos[1] as f32, next_pos[2] as f32);

        let direction = (target - transform.translation).normalize();

        if transform.translation.distance(target) < 0.01 {
            if pather.current <= 1 {
                commands.entity(entity).remove::<Path>();
            } else {
                pather.current -= 1;
            }
        } else {
            transform.translation += direction * time.delta_seconds() * 3.;
        }

        for i in 1..pather.path.len() {
            let current = pather.path[i - 1];
            let next = pather.path[i];
            let mid = Vec3::new(0.5, 0.5, 0.5);

            gizmos.line(
                Vec3::new(current[0] as f32, current[1] as f32, current[2] as f32) + mid,
                Vec3::new(next[0] as f32, next[1] as f32, next[2] as f32) + mid,
                Color::RED,
            );
        }
    }
}

pub fn pathfinding(
    terrain: Res<Terrain>,
    mut commands: Commands,
    pathfinders: Query<(Entity, &NeedsPath)>,
) {
    for (e, needs_path) in pathfinders.iter() {
        println!(
            "find path {},{},{}->{},{},{}",
            needs_path.start[0],
            needs_path.start[1],
            needs_path.start[2],
            needs_path.goal[0],
            needs_path.goal[1],
            needs_path.goal[2]
        );

        let result = astar(AStarSettings {
            start: [
                needs_path.start[0] as i32,
                needs_path.start[1] as i32,
                needs_path.start[2] as i32,
            ],
            goal: [
                needs_path.goal[0] as i32,
                needs_path.goal[1] as i32,
                needs_path.goal[2] as i32,
            ],
            allow_diagonals: true,
            max_depth: 5000,
            cost: |a, b| {
                let block = terrain.get_block_detail_i32(b[0], b[1], b[2]);

                if block.block != Block::EMPTY {
                    return f32::INFINITY;
                }

                let below = terrain.get_block_detail_i32(b[0], b[1] - 1, b[2]);

                if !below.block.is_filled() {
                    return f32::INFINITY;
                }

                Distance::diagonal(a, b)
            },
        });

        println!("result {} -> {}", result.is_success, result.path.len());

        commands.entity(e).remove::<NeedsPath>();

        if result.is_success {
            commands.entity(e).insert(Path {
                current: result.path.len() - 2, // first one is the starting position
                path: result.path,
            });
        }
    }
}

pub fn on_pathfind(
    mut commands: Commands,
    mut ev_pathfind: EventReader<PathfindEvent>,
    colonists: Query<(Entity, &Transform), With<Colonist>>,
) {
    for ev in ev_pathfind.read() {
        for (e, t) in colonists.iter() {
            let start = [
                t.translation.x.floor().abs() as u32,
                t.translation.y.floor().abs() as u32,
                t.translation.z.floor().abs() as u32,
            ];
            commands.entity(e).insert(NeedsPath {
                start,
                goal: ev.pos,
            });
        }
    }
}
