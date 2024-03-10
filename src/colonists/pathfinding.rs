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
    utils::hashbrown::HashMap,
};
use ordered_float::OrderedFloat;

use crate::{
    colonists::Partition,
    common::{astar, AStarSettings, Distance, PriorityQueue},
    terrain, Terrain,
};

use super::{Colonist, PartitionGraph, PathfindEvent};

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

#[derive(Component)]
pub struct PartitionPath {
    path: Vec<u16>,
    current: usize,
}

pub fn path_follow(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    graph: Res<PartitionGraph>,
    terrain: Res<Terrain>,
    mut pathers: Query<(Entity, &mut PartitionPath, &mut Transform)>,
) {
    for (entity, mut pather, mut transform) in pathers.iter_mut() {
        let next_part = pather.path[pather.current];
        let Some(partition) = graph.get_partition(next_part) else {
            // old path! need recalculating
            println!("old path needs recalc!");
            continue;
        };
        let pos = terrain.get_chunk_offset(partition.chunk_idx);
        let target = Vec3::new(pos[0] as f32, (pos[1] + 16) as f32, pos[2] as f32);

        let direction = (target - transform.translation).normalize();
        let distance = transform.translation.distance(target);
        let move_dist = time.delta_seconds() * 4.;

        if distance < move_dist {
            if pather.current <= 1 {
                commands.entity(entity).remove::<Path>();
            } else {
                pather.current -= 1;
            }
        } else {
            transform.translation += direction * move_dist;
        }

        // for i in 1..pather.path.len() {
        //     let current = pather.path[i - 1];
        //     let next = pather.path[i];
        //     let mid = Vec3::new(0.5, 0.5, 0.5);

        //     gizmos.line(
        //         Vec3::new(current[0] as f32, current[1] as f32, current[2] as f32) + mid,
        //         Vec3::new(next[0] as f32, next[1] as f32, next[2] as f32) + mid,
        //         Color::RED,
        //     );
        // }
    }
}

struct PartitionPathResult {
    pub is_success: bool,
    pub path: Vec<u16>,
    pub cost: f32,
}

pub fn pathfinding(
    terrain: Res<Terrain>,
    graph: Res<PartitionGraph>,
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

        let [start_chunk_idx, start_block_idx] = terrain.get_block_indexes(
            needs_path.start[0],
            needs_path.start[1],
            needs_path.start[2],
        );
        let [goal_chunk_idx, goal_block_idx] =
            terrain.get_block_indexes(needs_path.goal[0], needs_path.goal[1], needs_path.goal[2]);

        let starting_partition = terrain.get_partition_id(start_chunk_idx, start_block_idx);
        let goal_partition = terrain.get_partition_id(goal_chunk_idx, goal_block_idx);

        if starting_partition == Partition::NONE {
            println!("cannot find path, no starting partition!");
            commands.entity(e).remove::<NeedsPath>();
            continue;
        }

        if goal_partition == Partition::NONE {
            println!("cannot find path, no goal partition!");
            commands.entity(e).remove::<NeedsPath>();
            continue;
        }

        if starting_partition == goal_partition {
            commands.entity(e).insert(PartitionPath {
                current: 1,
                path: vec![goal_partition, starting_partition],
            });

            continue;
        }

        let result = astar(AStarSettings {
            start: starting_partition,
            goal: goal_partition,
            max_depth: 2000,
            neighbors: |v| {
                if let Some(p) = graph.get_partition(v) {
                    return p.neighbors.iter().copied().collect();
                }
                vec![]
            },
            heuristic: |a, b| {
                let chunk_idx_a = graph.get_partition(a).unwrap().chunk_idx;
                let chunk_idx_b = graph.get_partition(b).unwrap().chunk_idx;

                let [ax, ay, az] = terrain.get_chunk_offset(chunk_idx_a);
                let [bx, by, bz] = terrain.get_chunk_offset(chunk_idx_b);

                Distance::diagonal(
                    [ax as i32, ay as i32, az as i32],
                    [bx as i32, by as i32, bz as i32],
                )
            },
            cost: |a, b| {
                println!("cost {}->{}", a, b);

                if let Some(p) = graph.get_partition(b) {
                    let len = p.blocks.len() as f32;
                    return len.max(1.);
                }

                println!("partition does not exist {}, neighbor of {}", b, a);
                1000.0
            },
        });

        if result.is_success {
            commands.entity(e).insert(PartitionPath {
                current: result.path.len() - 2, // first one is the starting position
                path: result.path,
            });
        }

        commands.entity(e).remove::<NeedsPath>();
    }
}

// pub fn pathfinding_old(
//     terrain: Res<Terrain>,
//     mut commands: Commands,
//     pathfinders: Query<(Entity, &NeedsPath)>,
// ) {
//     for (e, needs_path) in pathfinders.iter() {
//         println!(
//             "find path {},{},{}->{},{},{}",
//             needs_path.start[0],
//             needs_path.start[1],
//             needs_path.start[2],
//             needs_path.goal[0],
//             needs_path.goal[1],
//             needs_path.goal[2]
//         );

//         let result = astar(AStarSettings {
//             start: [
//                 needs_path.start[0] as i32,
//                 needs_path.start[1] as i32,
//                 needs_path.start[2] as i32,
//             ],
//             goal: [
//                 needs_path.goal[0] as i32,
//                 needs_path.goal[1] as i32,
//                 needs_path.goal[2] as i32,
//             ],
//             allow_diagonals: true,
//             max_depth: 5000,
//             cost: |a, b| {
//                 let block = terrain.get_block_detail_i32(b[0], b[1], b[2]);

//                 if !block.block.is_empty() {
//                     return f32::INFINITY;
//                 }

//                 let below = terrain.get_block_detail_i32(b[0], b[1] - 1, b[2]);

//                 if !below.block.is_filled() {
//                     return f32::INFINITY;
//                 }

//                 Distance::diagonal(a, b)
//             },
//         });

//         println!("result {} -> {}", result.is_success, result.path.len());

//         commands.entity(e).remove::<NeedsPath>();

//         if result.is_success {
//             commands.entity(e).insert(Path {
//                 current: result.path.len() - 2, // first one is the starting position
//                 path: result.path,
//             });
//         }
//     }
// }

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
