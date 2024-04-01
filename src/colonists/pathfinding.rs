use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    math::vec3,
    time::Time,
    transform::components::Transform,
};
use ordered_float::*;

use crate::{
    common::{astar, AStarSettings, Distance},
    Terrain,
};

use super::{get_block_flags, NavigationFlags, NavigationGraph};

#[derive(Component)]
pub struct BlockMove {
    pub speed: f32,
    pub target: [i32; 3],
}

#[derive(Component, Default)]
pub struct Path {
    pub partition_path: Vec<u32>,
    pub goals: Vec<[u32; 3]>,
    pub current_partition_idx: usize,
    pub flags: NavigationFlags,
    pub blocks: Vec<[i32; 3]>,
    pub current_block_idx: usize,
}

#[derive(Clone, Component, Debug)]
pub struct PartitionPathRequest {
    pub start: [u32; 3],
    pub goals: Vec<[u32; 3]>,
    pub flags: NavigationFlags,
}

pub fn block_move_system(
    mut cmd: Commands,
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
            cmd.entity(entity).remove::<BlockMove>();
        } else {
            transform.translation += direction * move_dist;
        }
    }
}

pub struct GranularPathRequest {
    pub start: [u32; 3],
    pub goals: Vec<[u32; 3]>,
    pub goal_partition_id: u32,
    pub flags: NavigationFlags,
}

pub struct GranularPath {
    pub blocks: Vec<[i32; 3]>,
    pub flags: NavigationFlags,
    pub goals: Vec<[u32; 3]>,
    pub goal_partition_id: u32,
}

pub fn get_granular_path(
    graph: &NavigationGraph,
    terrain: &Terrain,
    request: &GranularPathRequest,
) -> Option<GranularPath> {
    let Some(current_partition_id) =
        terrain.get_partition_id_u32(request.start[0], request.start[1], request.start[2])
    else {
        return None;
    };
    let is_last_partition = request.goal_partition_id == *current_partition_id;
    let goal_partition = graph.get_partition(&request.goal_partition_id)?;

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

                let Some(partition_id) = terrain.get_partition_id(chunk_idx, block_idx) else {
                    return false;
                };

                *partition_id == request.goal_partition_id
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
                != NavigationFlags::NONE;
            let r_clear = get_block_flags(terrain, right[0], right[1], right[2]) & request.flags
                != NavigationFlags::NONE;
            let l_clear = get_block_flags(terrain, left[0], left[1], left[2]) & request.flags
                != NavigationFlags::NONE;
            let b_clear = get_block_flags(terrain, back[0], back[1], back[2]) & request.flags
                != NavigationFlags::NONE;

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

                    let Some(partition_id) = terrain.get_partition_id(chunk_idx, block_idx) else {
                        return None;
                    };

                    let Some(partition) = graph.get_partition(partition_id) else {
                        return None;
                    };

                    if partition.flags & request.flags != NavigationFlags::NONE {
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

impl Path {
    pub fn next_partition_id(&self) -> Option<&u32> {
        if self.current_partition_idx > 0 {
            return self.partition_path.get(self.current_partition_idx - 1);
        }

        self.partition_path.first()
    }

    pub fn next_block(&self) -> Option<&[i32; 3]> {
        if self.current_block_idx > 0 {
            return self.blocks.get(self.current_block_idx - 1);
        }

        self.blocks.first()
    }
}

pub struct PartitionPath {
    pub path: Vec<u32>,
    pub goals: Vec<[u32; 3]>,
    pub flags: NavigationFlags,
}

pub fn get_partition_path(
    request: &PartitionPathRequest,
    terrain: &Terrain,
    graph: &NavigationGraph,
) -> Option<PartitionPath> {
    let [start_chunk_idx, start_block_idx] =
        terrain.get_block_indexes(request.start[0], request.start[1], request.start[2]);

    let goals: Vec<([u32; 3], u32)> = request
        .goals
        .iter()
        .map(|g| (*g, terrain.get_block_indexes(g[0], g[1], g[2])))
        .map(|(g, [g_chunk_idx, g_block_idx])| {
            (g, terrain.get_partition_id(g_chunk_idx, g_block_idx))
        })
        .filter_map(|(g, p_id)| {
            let Some(id) = p_id else {
                return None;
            };
            Some((g, *id))
        })
        .collect();

    let mut goal_partition_ids: Vec<u32> = goals.iter().map(|(_, pid)| *pid).collect();
    goal_partition_ids.sort();
    goal_partition_ids.dedup();

    let Some(starting_partition_id) = terrain.get_partition_id(start_chunk_idx, start_block_idx)
    else {
        return None;
    };

    if goals.is_empty() {
        return None;
    }

    if goal_partition_ids.contains(&starting_partition_id) {
        return Some(PartitionPath {
            path: vec![*starting_partition_id],
            goals: request.goals.clone(),
            flags: request.flags,
        });
    }

    let partition_path = astar(AStarSettings {
        start: *starting_partition_id,
        is_goal: |p| goal_partition_ids.contains(&p),
        max_depth: 2000,
        neighbors: |v| {
            if let Some(p) = graph.get_partition(&v) {
                return p
                    .neighbor_ids
                    .iter()
                    .filter(|n| {
                        let Some(n_p) = graph.get_partition(*n) else {
                            return false;
                        };
                        n_p.flags & request.flags != NavigationFlags::NONE
                    })
                    .copied()
                    .collect();
            }
            vec![]
        },
        heuristic: |a| {
            let [ax, ay, az] = graph.get_partition(&a).unwrap().extents.center();

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
            let [ax, ay, az] = graph.get_partition(&a).unwrap().extents.center();
            let [bx, by, bz] = graph.get_partition(&b).unwrap().extents.center();

            Distance::diagonal(
                [ax as i32, ay as i32, az as i32],
                [bx as i32, by as i32, bz as i32],
            )
        },
    });

    if !partition_path.is_success {
        return None;
    }

    Some(PartitionPath {
        path: partition_path.path,
        goals: request.goals.clone(),
        flags: request.flags,
    })
}
