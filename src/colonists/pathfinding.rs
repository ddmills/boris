use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::{With, Without},
        system::{Commands, Query, Res},
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

use super::{Colonist, PartitionGraph, PathfindEvent};

#[derive(Component)]
pub struct NeedsPath {
    start: [u32; 3],
    goal: [u32; 3],
}

#[derive(Component)]
pub struct PathSegment {
    blocks: Vec<[u32; 3]>,
    current: usize,
}

#[derive(Component)]
pub struct PathComplete {}

#[derive(Component)]
pub struct PartitionPath {
    path: Vec<u16>,
    goal: [u32; 3],
    current: usize,
}

#[derive(Component)]
pub struct BlockMove {
    pub speed: f32,
    pub target: [u32; 3],
}

pub fn path_follow_block(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BlockMove, &mut Transform)>,
) {
    for (entity, mut block_move, mut transform) in query.iter_mut() {
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
    mut commands: Commands,
    mut pathers: Query<(Entity, &mut PathSegment), Without<BlockMove>>,
) {
    for (entity, mut path) in pathers.iter_mut() {
        if path.current == 0 {
            commands.entity(entity).remove::<PathSegment>();
            continue;
        }

        path.current -= 1;

        let next_block = path.blocks[path.current];

        commands.entity(entity).insert(BlockMove {
            target: next_block,
            speed: 4.,
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
    terrain: Res<Terrain>,
    mut pathers: Query<(Entity, &mut PartitionPath, &Transform), Without<PathSegment>>,
) {
    for (entity, mut path, transform) in pathers.iter_mut() {
        if path.current == 0 {
            println!("completed path follow!");
            commands.entity(entity).remove::<PartitionPath>();
            continue;
        }
        path.current -= 1;

        if path.current != 0 {
            let current_partition_id = path.path[path.current];
            let next_partition_id = path.path[path.current - 1];

            let Some(current_partition) = graph.get_partition(current_partition_id) else {
                // old path! need recalculating
                println!("old path needs recalc! (missing current)");
                commands.entity(entity).remove::<PartitionPath>();
                continue;
            };

            let Some(next_partition) = graph.get_partition(next_partition_id) else {
                // old path! need recalculating
                println!("old path needs recalc! (missing next)");
                commands.entity(entity).remove::<PartitionPath>();
                continue;
            };

            let pos = [
                transform.translation.x as i32,
                transform.translation.y as i32,
                transform.translation.z as i32,
            ];

            let next_centroid = next_partition.extents.center();

            let result = astar(AStarSettings {
                start: pos,
                is_goal: |p| {
                    // assuming u32 here as we are filter oob earlier
                    let [chunk_idx, block_idx] =
                        terrain.get_block_indexes(p[0] as u32, p[1] as u32, p[2] as u32);
                    let partition_id = terrain.get_partition_id(chunk_idx, block_idx);

                    partition_id == next_partition_id
                },
                cost: |a, b| Distance::diagonal([a[0], a[1], a[2]], [b[0], b[1], b[2]]),
                heuristic: |v| {
                    Distance::diagonal(
                        v,
                        [
                            next_centroid[0] as i32,
                            next_centroid[1] as i32,
                            next_centroid[2] as i32,
                        ],
                    )
                },
                neighbors: |v| {
                    [
                        [v[0], v[1] + 1, v[2]], // UP
                        [v[0], v[1] - 1, v[2]], // DOWN
                        [v[0] - 1, v[1], v[2]], // LEFT
                        [v[0] + 1, v[1], v[2]], // RIGHT
                        [v[0], v[1], v[2] - 1], // FORWARD
                        [v[0], v[1], v[2] + 1], // BACK
                    ]
                    .iter()
                    .filter_map(|p| {
                        // let block = terrain.get_block_i32(p[0], p[1], p[2]);
                        // todo, use block flags, add diagonals when possible

                        let [chunk_idx, block_idx] =
                            terrain.get_block_indexes(p[0] as u32, p[1] as u32, p[2] as u32);
                        let partition_id = terrain.get_partition_id(chunk_idx, block_idx);

                        if partition_id == current_partition_id || partition_id == next_partition_id
                        {
                            Some(*p)
                        } else {
                            None
                        }
                    })
                    .collect()
                },
                max_depth: 300,
            });

            if !result.is_success {
                println!("no segment path found!");
                commands.entity(entity).remove::<PartitionPath>();
                return;
            }

            let blocks = result
                .path
                .iter()
                .map(|p| [p[0] as u32, p[1] as u32, p[2] as u32])
                .collect();

            commands.entity(entity).insert(PathSegment {
                blocks,
                current: result.path.len(),
            });
        } else {
            // we are in last one, we need special astar logic
            let partition_id = path.path[path.current];

            let Some(partition) = graph.get_partition(partition_id) else {
                // old path! need recalculating
                println!("old path needs recalc! (missing current)");
                commands.entity(entity).remove::<PartitionPath>();
                continue;
            };

            let pos = [
                transform.translation.x as i32,
                transform.translation.y as i32,
                transform.translation.z as i32,
            ];

            let goal_i32 = [
                path.goal[0] as i32,
                path.goal[1] as i32,
                path.goal[2] as i32,
            ];

            let result = astar(AStarSettings {
                start: pos,
                is_goal: |p| {
                    // assuming u32 here as we are filter oob earlier
                    p[0] == goal_i32[0] && p[1] == goal_i32[1] && p[2] == goal_i32[2]
                },
                cost: |a, b| Distance::diagonal([a[0], a[1], a[2]], [b[0], b[1], b[2]]),
                heuristic: |v| Distance::diagonal(v, goal_i32),
                neighbors: |v| {
                    [
                        [v[0], v[1] + 1, v[2]], // UP
                        [v[0], v[1] - 1, v[2]], // DOWN
                        [v[0] - 1, v[1], v[2]], // LEFT
                        [v[0] + 1, v[1], v[2]], // RIGHT
                        [v[0], v[1], v[2] - 1], // FORWARD
                        [v[0], v[1], v[2] + 1], // BACK
                    ]
                    .iter()
                    .filter_map(|p| {
                        // let block = terrain.get_block_i32(p[0], p[1], p[2]);
                        // todo, use block flags, add diagonals when possible

                        let [chunk_idx, block_idx] =
                            terrain.get_block_indexes(p[0] as u32, p[1] as u32, p[2] as u32);
                        let p_id = terrain.get_partition_id(chunk_idx, block_idx);

                        if p_id == partition_id {
                            Some(*p)
                        } else {
                            None
                        }
                    })
                    .collect()
                },
                max_depth: 300,
            });

            if !result.is_success {
                println!("no final segment path found!");
                commands.entity(entity).remove::<PartitionPath>();
                return;
            }

            let blocks = result
                .path
                .iter()
                .map(|p| [p[0] as u32, p[1] as u32, p[2] as u32])
                .collect();

            commands.entity(entity).insert(PathSegment {
                blocks,
                current: result.path.len(),
            });
        }
    }
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
        let goal_partition_id = terrain.get_partition_id(goal_chunk_idx, goal_block_idx);

        if starting_partition == Partition::NONE {
            println!("cannot find path, no starting partition!");
            commands.entity(e).remove::<NeedsPath>();
            continue;
        }

        if goal_partition_id == Partition::NONE {
            println!("cannot find path, no goal partition!");
            commands.entity(e).remove::<NeedsPath>();
            continue;
        }

        if starting_partition == goal_partition_id {
            commands.entity(e).insert(PartitionPath {
                current: 2,
                path: vec![goal_partition_id, starting_partition],
                goal: needs_path.goal,
            });

            continue;
        }

        let goal_partition = graph.get_partition(goal_partition_id).unwrap();
        let destination = goal_partition.extents.center();

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
                let [bx, by, bz] = destination;

                Distance::diagonal(
                    [ax as i32, ay as i32, az as i32],
                    [bx as i32, by as i32, bz as i32],
                )
            },
            cost: |a, b| {
                let [ax, ay, az] = graph.get_partition(a).unwrap().extents.center();
                let [bx, by, bz] = graph.get_partition(b).unwrap().extents.center();

                println!("cost {}->{}", a, b);
                Distance::diagonal(
                    [ax as i32, ay as i32, az as i32],
                    [bx as i32, by as i32, bz as i32],
                )
            },
        });

        commands.entity(e).remove::<NeedsPath>();

        if !partition_path.is_success {
            println!("could not find path");
            return;
        }

        commands.entity(e).insert(PartitionPath {
            current: partition_path.path.len(), // first one is the starting position
            path: partition_path.path,
            goal: needs_path.goal,
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
            commands.entity(e).insert(NeedsPath {
                start,
                goal: ev.pos,
            });
        }
    }
}
