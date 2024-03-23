use std::cmp::Ordering;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    gizmos::gizmos::Gizmos,
    math::{vec3, Vec3},
    render::color::Color,
    time::Time,
    transform::components::Transform,
};
use ordered_float::*;

use crate::{
    colonists::Partition,
    common::{astar, AStarSettings, Distance},
    Terrain,
};

use super::{get_block_flags, PartitionFlags, PartitionGraph};

#[derive(Component)]
pub struct BlockMove {
    pub speed: f32,
    pub target: [i32; 3],
}

#[derive(Component, Default)]
pub struct Path {
    pub partition_path: Vec<u16>,
    pub goals: Vec<[u32; 3]>,
    pub current_partition_idx: usize,
    pub flags: PartitionFlags,
    pub blocks: Vec<[i32; 3]>,
    pub current_block_idx: usize,
}

#[derive(Clone, Component, Debug)]
pub struct PartitionPathRequest {
    pub start: [u32; 3],
    pub goals: Vec<[u32; 3]>,
    pub flags: PartitionFlags,
}

pub fn block_move_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &BlockMove, &mut Transform)>,
) {
    for (entity, block_move, mut transform) in query.iter_mut() {
        let pos = vec3(
            block_move.target[0] as f32,
            block_move.target[1] as f32,
            block_move.target[2] as f32,
        );

        let direction = (pos - transform.translation).normalize();
        let distance = transform.translation.distance(pos);
        let move_dist = time.delta_seconds() * block_move.speed;

        if distance < move_dist {
            transform.translation = pos;
            commands.entity(entity).remove::<BlockMove>();
        } else {
            transform.translation += direction * move_dist;
        }
    }
}

pub fn path_debug(mut gizmos: Gizmos, pathers: Query<&Path>) {
    for path in pathers.iter() {
        for i in 1..path.blocks.len() {
            let current = path.blocks[i - 1];
            let next = path.blocks[i];

            let mid = Vec3::new(0.5, 0.5, 0.5);

            let color = match i.cmp(&path.current_block_idx) {
                Ordering::Less => Color::ORANGE,
                Ordering::Equal => Color::ORANGE_RED,
                Ordering::Greater => Color::GRAY,
            };

            gizmos.line(
                Vec3::new(current[0] as f32, current[1] as f32, current[2] as f32) + mid,
                Vec3::new(next[0] as f32, next[1] as f32, next[2] as f32) + mid,
                color,
            );
        }

        for g in path.goals.iter() {
            let pos = Vec3::new(g[0] as f32, g[1] as f32 + 0.04, g[2] as f32);

            gizmos.line(pos, pos + Vec3::new(1., 0., 0.), Color::CYAN);
            gizmos.line(pos, pos + Vec3::new(0., 0., 1.), Color::CYAN);

            gizmos.line(pos, pos + Vec3::new(1., 0., 0.), Color::CYAN);
            gizmos.line(pos, pos + Vec3::new(0., 0., 1.), Color::CYAN);

            gizmos.line(
                pos + Vec3::new(1., 0., 1.),
                pos + Vec3::new(1., 0., 0.),
                Color::CYAN,
            );
            gizmos.line(
                pos + Vec3::new(1., 0., 1.),
                pos + Vec3::new(0., 0., 1.),
                Color::CYAN,
            );
        }
    }
}

pub struct GranularPathRequest {
    pub start: [u32; 3],
    pub goals: Vec<[u32; 3]>,
    pub goal_partition_id: u16,
    pub flags: PartitionFlags,
}

pub struct GranularPath {
    pub blocks: Vec<[i32; 3]>,
    pub flags: PartitionFlags,
    pub goals: Vec<[u32; 3]>,
    pub goal_partition_id: u16,
}

pub fn get_granular_path(
    graph: &PartitionGraph,
    terrain: &Terrain,
    request: &GranularPathRequest,
) -> Option<GranularPath> {
    let current_partition_id =
        terrain.get_partition_id_u32(request.start[0], request.start[1], request.start[2]);
    let is_last_partition = request.goal_partition_id == current_partition_id;
    let goal_partition = graph.get_partition(request.goal_partition_id)?;

    let goal_positions = if is_last_partition {
        request
            .goals
            .iter()
            .map(|g| [g[0] as i32, g[1] as i32, g[2] as i32])
            .collect()
    } else {
        let c = goal_partition.extents.center();
        vec![[c[0] as i32, c[1] as i32, c[2] as i32]]
    };

    let result = astar(AStarSettings {
        start: [
            request.start[0] as i32,
            request.start[1] as i32,
            request.start[2] as i32,
        ],
        is_goal: |p| {
            // assuming u32 here as we are filter oob earlier
            if is_last_partition {
                goal_positions
                    .iter()
                    .any(|g| p[0] == g[0] && p[1] == g[1] && p[2] == g[2])
            } else {
                let [chunk_idx, block_idx] =
                    terrain.get_block_indexes(p[0] as u32, p[1] as u32, p[2] as u32);
                let partition_id = terrain.get_partition_id(chunk_idx, block_idx);
                partition_id == request.goal_partition_id
            }
        },
        cost: |a, b| Distance::diagonal([a[0], a[1], a[2]], [b[0], b[1], b[2]]),
        heuristic: |v| {
            if is_last_partition {
                goal_positions
                    .iter()
                    .map(|g| OrderedFloat(Distance::diagonal(v, *g)))
                    .min()
                    .unwrap()
                    .0
            } else {
                goal_partition.extents.distance_to_edge(v[0], v[1], v[2])
            }
        },
        neighbors: |v| {
            // TODO: extract neighbors to block graph
            let up = [v[0], v[1] + 1, v[2]];
            let down = [v[0], v[1] - 1, v[2]];
            let left = [v[0] - 1, v[1], v[2]];
            let right = [v[0] + 1, v[1], v[2]];
            let forward = [v[0], v[1], v[2] - 1];
            let back = [v[0], v[1], v[2] + 1];

            let forward_left = [v[0] - 1, v[1], v[2] - 1];
            let forward_right = [v[0] + 1, v[1], v[2] - 1];
            let back_left = [v[0] - 1, v[1], v[2] + 1];
            let back_right = [v[0] + 1, v[1], v[2] + 1];

            let mut edges = vec![up, down, left, right, forward, back];

            let f_clear = get_block_flags(terrain, forward[0], forward[1], forward[2])
                & request.flags
                != PartitionFlags::NONE;
            let r_clear = get_block_flags(terrain, right[0], right[1], right[2]) & request.flags
                != PartitionFlags::NONE;
            let l_clear = get_block_flags(terrain, left[0], left[1], left[2]) & request.flags
                != PartitionFlags::NONE;
            let b_clear = get_block_flags(terrain, back[0], back[1], back[2]) & request.flags
                != PartitionFlags::NONE;

            if f_clear && l_clear {
                edges.push(forward_left);
            }
            if f_clear && r_clear {
                edges.push(forward_right);
            }
            if b_clear && l_clear {
                edges.push(back_left);
            }
            if b_clear && r_clear {
                edges.push(back_right);
            }

            edges
                .iter()
                .filter_map(|p| {
                    let [chunk_idx, block_idx] =
                        terrain.get_block_indexes(p[0] as u32, p[1] as u32, p[2] as u32);
                    let partition_id = terrain.get_partition_id(chunk_idx, block_idx);
                    let part_flags = graph.get_flags(partition_id);

                    if part_flags & request.flags != PartitionFlags::NONE {
                        Some(*p)
                    } else {
                        None
                    }
                })
                .collect()
        },
        max_depth: 3000,
    });

    if !result.is_success {
        println!("no granular path found!");
        return None;
    }

    Some(GranularPath {
        blocks: result.path,
        flags: request.flags,
        goals: request.goals.clone(),
        goal_partition_id: request.goal_partition_id,
    })
}

pub fn is_reachable(
    start_id: u16,
    goal_ids: Vec<u16>,
    graph: PartitionGraph,
    flags: PartitionFlags,
) -> bool {
    if goal_ids.contains(&start_id) {
        return true;
    }

    let partition_path = astar(AStarSettings {
        start: start_id,
        is_goal: |p| goal_ids.contains(&p),
        max_depth: 2000,
        neighbors: |v| {
            if let Some(p) = graph.get_partition(v) {
                return p
                    .neighbors
                    .iter()
                    .filter(|n| graph.get_flags(**n) & flags != PartitionFlags::NONE)
                    .copied()
                    .collect();
            }
            vec![]
        },
        heuristic: |a| {
            let [ax, ay, az] = graph.get_partition(a).unwrap().extents.center();

            goal_ids
                .iter()
                .filter_map(|g_id| {
                    if let Some(g) = graph.get_center(*g_id) {
                        return Some(OrderedFloat(Distance::diagonal(
                            [ax as i32, ay as i32, az as i32],
                            [g[0] as i32, g[1] as i32, g[2] as i32],
                        )));
                    }
                    None
                })
                .min()
                .unwrap()
                .0
        },
        cost: |a, b| {
            let [ax, ay, az] = graph.get_partition(a).unwrap().extents.center();
            let [bx, by, bz] = graph.get_partition(b).unwrap().extents.center();

            Distance::diagonal(
                [ax as i32, ay as i32, az as i32],
                [bx as i32, by as i32, bz as i32],
            )
        },
    });

    partition_path.is_success
}

impl Path {
    pub fn next_partition_id(&self) -> u16 {
        if self.current_partition_idx > 0 {
            return self.partition_path[self.current_partition_idx - 1];
        }

        self.partition_path[0]
    }

    pub fn next_block(&self) -> [i32; 3] {
        if self.current_block_idx > 0 {
            return self.blocks[self.current_block_idx - 1];
        }

        self.blocks[0]
    }
}

pub struct PartitionPath {
    pub path: Vec<u16>,
    pub goals: Vec<[u32; 3]>,
    pub flags: PartitionFlags,
}

pub fn get_partition_path(
    request: &PartitionPathRequest,
    terrain: &Terrain,
    graph: &PartitionGraph,
) -> Option<PartitionPath> {
    let [start_chunk_idx, start_block_idx] =
        terrain.get_block_indexes(request.start[0], request.start[1], request.start[2]);

    let goals: Vec<([u32; 3], u16)> = request
        .goals
        .iter()
        .map(|g| (*g, terrain.get_block_indexes(g[0], g[1], g[2])))
        .map(|(g, [g_chunk_idx, g_block_idx])| {
            (g, terrain.get_partition_id(g_chunk_idx, g_block_idx))
        })
        .filter(|(_, p_id)| *p_id != Partition::NONE)
        .collect();

    let mut goal_partition_ids: Vec<u16> = goals.iter().map(|(_, pid)| *pid).collect();
    goal_partition_ids.sort();
    goal_partition_ids.dedup();

    let starting_partition_id = terrain.get_partition_id(start_chunk_idx, start_block_idx);

    if starting_partition_id == Partition::NONE {
        println!("cannot find path, no starting partition!");
        return None;
    }

    if goals.is_empty() {
        println!("cannot find path, no goal partition!");
        return None;
    }

    if goal_partition_ids.contains(&starting_partition_id) {
        return Some(PartitionPath {
            path: vec![starting_partition_id],
            goals: request.goals.clone(),
            flags: request.flags,
        });
    }

    let partition_path = astar(AStarSettings {
        start: starting_partition_id,
        is_goal: |p| goal_partition_ids.contains(&p),
        max_depth: 2000,
        neighbors: |v| {
            if let Some(p) = graph.get_partition(v) {
                return p
                    .neighbors
                    .iter()
                    .filter(|n| graph.get_flags(**n) & request.flags != PartitionFlags::NONE)
                    .copied()
                    .collect();
            }
            vec![]
        },
        heuristic: |a| {
            let [ax, ay, az] = graph.get_partition(a).unwrap().extents.center();

            goals
                .iter()
                .map(|(g, _pid)| {
                    OrderedFloat(Distance::diagonal(
                        [ax as i32, ay as i32, az as i32],
                        [g[0] as i32, g[1] as i32, g[2] as i32],
                    ))
                })
                .min()
                .unwrap()
                .0
        },
        cost: |a, b| {
            let [ax, ay, az] = graph.get_partition(a).unwrap().extents.center();
            let [bx, by, bz] = graph.get_partition(b).unwrap().extents.center();

            Distance::diagonal(
                [ax as i32, ay as i32, az as i32],
                [bx as i32, by as i32, bz as i32],
            )
        },
    });

    if !partition_path.is_success {
        println!("could not find path");
        return None;
    }

    Some(PartitionPath {
        path: partition_path.path,
        goals: request.goals.clone(),
        flags: request.flags,
    })
}
