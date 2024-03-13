use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    math::{vec3, Vec3},
    render::color::Color,
    time::Time,
    transform::components::Transform,
};

use crate::{
    colonists::Partition,
    common::{astar, AStarSettings, Distance},
    Terrain,
};

use super::{get_block_flags, Colonist, PartitionFlags, PartitionGraph, PathfindEvent};

#[derive(Component)]
pub struct PathfindRequest {
    start: [u32; 3],
    goal: [u32; 3],
    flags: PartitionFlags,
}

#[derive(Component)]
pub struct PathSegment {
    blocks: Vec<[i32; 3]>,
    current: usize,
    flags: PartitionFlags,
    goal: [u32; 3],
}

#[derive(Component)]
pub struct PartitionPath {
    path: Vec<u16>,
    goal: [u32; 3],
    current: usize,
    flags: PartitionFlags,
}

#[derive(Component)]
pub struct BlockMove {
    pub speed: f32,
    pub target: [i32; 3],
}

pub fn path_follow_block(
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

pub fn path_follow_segment(
    terrain: ResMut<Terrain>,
    mut commands: Commands,
    mut pathers: Query<(Entity, &mut PathSegment, &Transform), Without<BlockMove>>,
) {
    for (entity, mut path, transform) in pathers.iter_mut() {
        if path.current == 0 {
            commands.entity(entity).remove::<PathSegment>();
            continue;
        }

        path.current -= 1;

        let next_block = path.blocks[path.current];

        if get_block_flags(&terrain, next_block[0], next_block[1], next_block[2]) & path.flags
            == PartitionFlags::NONE
        {
            commands.entity(entity).remove::<PathSegment>();
            commands.entity(entity).remove::<PartitionPath>();
            commands.entity(entity).insert(PathfindRequest {
                goal: path.goal,
                flags: path.flags,
                start: [
                    transform.translation.x as u32,
                    transform.translation.y as u32,
                    transform.translation.z as u32,
                ],
            });
            return;
        }

        commands.entity(entity).insert(BlockMove {
            target: next_block,
            speed: 8.,
        });
    }
}

pub fn path_follow_segment_debug(mut gizmos: Gizmos, pathers: Query<&PathSegment>) {
    for path in pathers.iter() {
        for i in 1..path.blocks.len() {
            let current = path.blocks[i - 1];
            let next = path.blocks[i];

            let mid = Vec3::new(0.5, 0.5, 0.5);

            let color = if i > path.current {
                Color::GRAY
            } else if i == path.current {
                Color::ORANGE_RED
            } else {
                Color::ORANGE
            };

            gizmos.line(
                Vec3::new(current[0] as f32, current[1] as f32, current[2] as f32) + mid,
                Vec3::new(next[0] as f32, next[1] as f32, next[2] as f32) + mid,
                color,
            );
        }
    }
}

pub fn path_follow_partition(
    mut commands: Commands,
    graph: Res<PartitionGraph>,
    terrain: ResMut<Terrain>,
    mut pathers: Query<(Entity, &mut PartitionPath, &Transform), Without<PathSegment>>,
) {
    for (entity, mut path, transform) in pathers.iter_mut() {
        if path.current == 0 {
            println!("completed path follow!");
            commands.entity(entity).remove::<PartitionPath>();
            continue;
        }

        path.current -= 1;

        let pos = [
            transform.translation.x as i32,
            transform.translation.y as i32,
            transform.translation.z as i32,
        ];

        let is_last_partition = path.current <= 1;

        let goal_pos = if is_last_partition {
            path.current = 0;
            [
                path.goal[0] as i32,
                path.goal[1] as i32,
                path.goal[2] as i32,
            ]
        } else {
            let idx = path.current - 1;
            let goal_partition_id = path.path[idx];
            let c = graph.get_center(goal_partition_id).unwrap();
            [c[0] as i32, c[1] as i32, c[2] as i32]
        };

        let next_partition = match is_last_partition {
            true => None,
            false => {
                let idx = path.current;
                let next_partition_id = path.path[idx];
                graph.get_partition(next_partition_id)
            }
        };

        let result = astar(AStarSettings {
            start: pos,
            is_goal: |p| {
                // assuming u32 here as we are filter oob earlier
                if is_last_partition {
                    p[0] as u32 == path.goal[0]
                        && p[1] as u32 == path.goal[1]
                        && p[2] as u32 == path.goal[2]
                } else {
                    let idx = path.current;
                    let next_partition_id = path.path[idx];
                    let [chunk_idx, block_idx] =
                        terrain.get_block_indexes(p[0] as u32, p[1] as u32, p[2] as u32);
                    let partition_id = terrain.get_partition_id(chunk_idx, block_idx);
                    partition_id == next_partition_id
                }
            },
            cost: |a, b| Distance::diagonal([a[0], a[1], a[2]], [b[0], b[1], b[2]]),
            heuristic: |v| {
                if is_last_partition {
                    Distance::diagonal(v, goal_pos)
                } else {
                    next_partition
                        .unwrap()
                        .extents
                        .distance_to_edge(v[0], v[1], v[2])
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

                let f_clear = get_block_flags(&terrain, forward[0], forward[1], forward[2])
                    & path.flags
                    != PartitionFlags::NONE;
                let r_clear = get_block_flags(&terrain, right[0], right[1], right[2]) & path.flags
                    != PartitionFlags::NONE;
                let l_clear = get_block_flags(&terrain, left[0], left[1], left[2]) & path.flags
                    != PartitionFlags::NONE;
                let b_clear = get_block_flags(&terrain, back[0], back[1], back[2]) & path.flags
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

                        if part_flags & path.flags != PartitionFlags::NONE {
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
            println!("no segment path found!");
            let mut cmds = commands.entity(entity);
            cmds.remove::<PartitionPath>();
            cmds.insert(PathfindRequest {
                goal: path.goal,
                flags: path.flags,
                start: [pos[0] as u32, pos[1] as u32, pos[2] as u32],
            });
            return;
        }

        if !result.is_success {
            println!("no final segment path found!");
            let mut cmds = commands.entity(entity);
            cmds.remove::<PartitionPath>();
            cmds.insert(PathfindRequest {
                goal: path.goal,
                flags: path.flags,
                start: [pos[0] as u32, pos[1] as u32, pos[2] as u32],
            });
            return;
        }

        commands.entity(entity).insert(PathSegment {
            current: result.path.len(),
            blocks: result.path,
            flags: path.flags,
            goal: path.goal,
        });
    }
}

pub fn pathfinding(
    terrain: Res<Terrain>,
    graph: Res<PartitionGraph>,
    mut commands: Commands,
    pathfinders: Query<(Entity, &PathfindRequest)>,
) {
    for (e, request) in pathfinders.iter() {
        println!(
            "find path {},{},{}->{},{},{}",
            request.start[0],
            request.start[1],
            request.start[2],
            request.goal[0],
            request.goal[1],
            request.goal[2]
        );

        commands.entity(e).remove::<PathfindRequest>();

        let [start_chunk_idx, start_block_idx] =
            terrain.get_block_indexes(request.start[0], request.start[1], request.start[2]);
        let [goal_chunk_idx, goal_block_idx] =
            terrain.get_block_indexes(request.goal[0], request.goal[1], request.goal[2]);

        let starting_partition = terrain.get_partition_id(start_chunk_idx, start_block_idx);
        let goal_partition_id = terrain.get_partition_id(goal_chunk_idx, goal_block_idx);

        if starting_partition == Partition::NONE {
            println!("cannot find path, no starting partition!");
            commands.entity(e).remove::<PathfindRequest>();
            continue;
        }

        if goal_partition_id == Partition::NONE {
            println!("cannot find path, no goal partition!");
            commands.entity(e).remove::<PathfindRequest>();
            continue;
        }

        if starting_partition == goal_partition_id {
            commands.entity(e).insert(PartitionPath {
                current: 1,
                path: vec![goal_partition_id],
                goal: request.goal,
                flags: request.flags,
            });

            continue;
        }

        let partition_path = astar(AStarSettings {
            start: starting_partition,
            is_goal: |p| p == goal_partition_id,
            max_depth: 2000,
            neighbors: |v| {
                if let Some(p) = graph.get_partition(v) {
                    return p.neighbors.iter().copied().collect();
                }
                vec![]
            },
            heuristic: |a| {
                let [ax, ay, az] = graph.get_partition(a).unwrap().extents.center();
                let [bx, by, bz] = request.goal;

                Distance::diagonal(
                    [ax as i32, ay as i32, az as i32],
                    [bx as i32, by as i32, bz as i32],
                )
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
            return;
        }

        commands.entity(e).insert(PartitionPath {
            current: partition_path.path.len() - 1, // first one is the starting position
            path: partition_path.path,
            goal: request.goal,
            flags: request.flags,
        });
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
            commands.entity(e).insert(PathfindRequest {
                start,
                goal: ev.pos,
                flags: PartitionFlags::TALL | PartitionFlags::LADDER,
            });
        }
    }
}
