use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    time::Time,
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::{common::Rand, Terrain};

use super::{
    get_block_flags, get_granular_path, get_partition_path, Actor, ActorRef, Blackboard, BlockMove,
    Fatigue, GranularPathRequest, PartitionFlags, PartitionGraph, PartitionPathRequest, Path,
    TaskBuilder, TaskState,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskFindBed;

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskSleep;

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskIdle;

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskPickRandomSpot;

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMoveTo;

pub fn task_pick_random_spot(
    mut rand: ResMut<Rand>,
    terrain: Res<Terrain>,
    graph: Res<PartitionGraph>,
    q_transforms: Query<&Transform, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut Blackboard, &mut TaskState), With<TaskPickRandomSpot>>,
) {
    for (ActorRef(actor), mut blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(transform) = q_transforms.get(*actor) else {
            println!("no transform on actor, cannot pick random spot!");
            *state = TaskState::Failed;
            continue;
        };

        let pos = [
            transform.translation.x as u32,
            transform.translation.y as u32,
            transform.translation.z as u32,
        ];

        let current_partition_id = terrain.get_partition_id_u32(pos[0], pos[1], pos[2]);
        let Some(current_partition) = graph.get_partition(current_partition_id) else {
            *state = TaskState::Failed;
            return;
        };

        let target_partition_id = if current_partition.neighbors.is_empty() {
            current_partition_id
        } else {
            let neighbors: Vec<u16> = current_partition.neighbors.iter().copied().collect();
            rand.pick(&neighbors)
        };

        let target_partition = graph.get_partition(target_partition_id).unwrap();
        let target_block_idx = rand.pick(&target_partition.blocks);
        let target_chunk_idx = target_partition.chunk_idx;

        let target_pos = terrain.get_block_world_pos(target_chunk_idx, target_block_idx);

        println!(
            "wander to {},{},{}",
            target_pos[0], target_pos[1], target_pos[2]
        );

        blackboard.move_goals = vec![target_pos];

        *state = TaskState::Success;
    }
}

pub fn task_move_to(
    mut commands: Commands,
    terrain: Res<Terrain>,
    graph: Res<PartitionGraph>,
    q_movers: Query<&BlockMove, With<Actor>>,
    q_transforms: Query<&Transform, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut Blackboard, &mut TaskState), With<TaskMoveTo>>,
) {
    for (ActorRef(actor), mut blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(transform) = q_transforms.get(*actor) else {
            println!("no transform on actor, cannot move to!");
            *state = TaskState::Failed;
            continue;
        };

        if q_movers.contains(*actor) {
            continue;
        }

        let pos = [
            transform.translation.x as u32,
            transform.translation.y as u32,
            transform.translation.z as u32,
        ];

        if blackboard.path.is_none() {
            if blackboard.move_goals.is_empty() {
                println!("no move_goals on blackboard, cannot move to!");
                *state = TaskState::Failed;
                continue;
            }

            let request = PartitionPathRequest {
                start: pos,
                goals: blackboard.move_goals.clone(),
                flags: PartitionFlags::TALL | PartitionFlags::LADDER,
            };

            // generate path
            let Some(partition_path) = get_partition_path(&request, &terrain, &graph) else {
                println!("path could not be generated, cannot move to!");
                *state = TaskState::Failed;
                continue;
            };

            println!("path generated!");
            blackboard.path = Some(Path {
                current_partition_idx: partition_path.goals.len() - 1,
                goals: partition_path.goals,
                partition_path: partition_path.path,
                flags: request.flags,
                blocks: vec![],
                current_block_idx: 0,
            });
        }

        let path = blackboard.path.as_mut().unwrap();

        // if we're at the goal, success
        let at_goal = path
            .goals
            .iter()
            .any(|g| g[0] == pos[0] && g[1] == pos[1] && g[2] == pos[2]);

        if at_goal {
            println!("goal reached!");
            blackboard.path = None;
            *state = TaskState::Success;
            continue;
        }

        // what partition are we standing in? if it's not part of the predetermined path, we stay course.
        // if it is part of the path, we set our current index to be the path idx
        let partition_id = terrain.get_partition_id_u32(pos[0], pos[1], pos[2]);
        let partition_path_idx = path.partition_path.iter().position(|p| *p == partition_id);

        if let Some(idx) = partition_path_idx {
            path.current_partition_idx = idx;
        };

        println!("pathing?");

        // if current block index is zero, it means we've finished the granular path
        if path.current_block_idx == 0 {
            let Some(granular_path) = get_granular_path(
                &graph,
                &terrain,
                &GranularPathRequest {
                    start: pos,
                    goals: path.goals.clone(),
                    goal_partition_id: path.next_partition_id(),
                    flags: path.flags,
                },
            ) else {
                println!("Could not get granular path, cannot move to!");
                *state = TaskState::Failed;
                continue;
            };

            path.blocks = granular_path.blocks.clone();
            path.current_block_idx = path.blocks.len() - 1;
        }

        path.current_block_idx -= 1;

        let next_block = path.next_block();
        let block_flags = get_block_flags(&terrain, next_block[0], next_block[1], next_block[2]);

        if block_flags & path.flags == PartitionFlags::NONE {
            println!("path has changed! it's now blocked!");
            *state = TaskState::Failed;
            continue;
        }

        let [x, y, z] = path.blocks[path.current_block_idx];
        println!("inserting block move! {},{},{}", x, y, z);
        commands.entity(*actor).insert(BlockMove {
            speed: 8.,
            target: path.blocks[path.current_block_idx],
        });
    }
}

pub fn task_find_bed(
    mut q_behavior: Query<(&ActorRef, &mut Blackboard, &mut TaskState), With<TaskFindBed>>,
) {
    for (ActorRef(entity), mut blackboard, mut state) in q_behavior.iter_mut() {
        if *state == TaskState::Executing {
            println!("find a bed for {}", entity.index());
            blackboard.bed = 3;
            *state = TaskState::Success;
        }
    }
}

pub fn task_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_behavior: Query<(&ActorRef, &Blackboard, &mut TaskState), With<TaskSleep>>,
) {
    for (ActorRef(entity), blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(mut fatigue) = q_fatigues.get_mut(*entity) else {
            println!("Actor entity does not have a fatigue");
            *state = TaskState::Failed;
            continue;
        };

        if *state == TaskState::Executing {
            if fatigue.value > 0. {
                fatigue.value -= time.delta_seconds() * 40.;
            }

            if fatigue.value <= 0. {
                println!("slept in bed {}", blackboard.bed);
                fatigue.value = 0.;
                *state = TaskState::Success;
            }
        }
    }
}

pub fn task_idle(
    time: Res<Time>,
    mut q_behavior: Query<(&mut TaskState, &mut Blackboard), With<TaskIdle>>,
) {
    for (mut state, mut blackboard) in q_behavior.iter_mut() {
        if *state == TaskState::Executing {
            if blackboard.idle_time < 100. {
                blackboard.idle_time += time.delta_seconds() * 20.;
                *state = TaskState::Executing;
            } else {
                *state = TaskState::Success;
            }
        }
    }
}
