use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        get_block_flags, get_granular_path, get_partition_path, Actor, ActorRef, Blackboard,
        BlockMove, GranularPathRequest, NavigationFlags, NavigationGraph, PartitionPathRequest,
        Path, TaskBuilder, TaskState,
    },
    Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMoveTo;

pub fn task_move_to(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    mut q_paths: Query<&mut Path, With<Actor>>,
    q_movers: Query<&BlockMove, With<Actor>>,
    q_transforms: Query<&Transform, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &Blackboard, &mut TaskState), With<TaskMoveTo>>,
) {
    for (ActorRef(actor), blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(transform) = q_transforms.get(*actor) else {
            println!("no transform on actor, cannot move to!");
            cmd.entity(*actor).remove::<Path>();
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

        let Ok(mut path) = q_paths.get_mut(*actor) else {
            if blackboard.move_goals.is_empty() {
                println!("no move_goals on blackboard, cannot move to!");
                *state = TaskState::Failed;
                continue;
            }

            let request = PartitionPathRequest {
                start: pos,
                goals: blackboard.move_goals.clone(),
                flags: NavigationFlags::TALL | NavigationFlags::LADDER,
            };

            // generate path
            let Some(partition_path) = get_partition_path(&request, &terrain, &graph) else {
                println!("path could not be generated, cannot move to!");
                *state = TaskState::Failed;
                continue;
            };

            let path = Path {
                current_partition_idx: partition_path.goals.len() - 1,
                goals: partition_path.goals,
                partition_path: partition_path.path,
                flags: request.flags,
                blocks: vec![],
                current_block_idx: 0,
            };

            cmd.entity(*actor).insert(path);
            continue;
        };

        let at_goal = path
            .goals
            .iter()
            .any(|g| g[0] == pos[0] && g[1] == pos[1] && g[2] == pos[2]);

        if at_goal {
            cmd.entity(*actor).remove::<Path>();
            *state = TaskState::Success;
            continue;
        }

        // what partition are we standing in? if it's not part of the predetermined path, we stay course.
        // if it is part of the path, we set our current index to be the path idx
        let Some(partition_id) = terrain.get_partition_id_u32(pos[0], pos[1], pos[2]) else {
            println!("Not standing in a partition, cannot path!");
            cmd.entity(*actor).remove::<Path>();
            *state = TaskState::Failed;
            continue;
        };

        let partition_path_idx = path.partition_path.iter().position(|p| p == partition_id);

        if let Some(idx) = partition_path_idx {
            path.current_partition_idx = idx;
        };

        // if current block index is zero, it means we've finished the granular path
        if path.current_block_idx == 0 {
            let Some(next_partition_id) = path.next_partition_id() else {
                println!("Path has changed? Retrying pathfinding.");
                cmd.entity(*actor).remove::<Path>();
                continue;
            };

            let Some(granular_path) = get_granular_path(
                &graph,
                &terrain,
                &GranularPathRequest {
                    start: pos,
                    goals: path.goals.clone(),
                    goal_partition_id: *next_partition_id,
                    flags: path.flags,
                },
            ) else {
                println!("Could not get granular path, retrying!");
                cmd.entity(*actor).remove::<Path>();
                continue;
            };

            path.blocks = granular_path.blocks.clone();
            path.current_block_idx = path.blocks.len() - 1;
        }

        path.current_block_idx -= 1;

        let Some(next_block) = path.next_block() else {
            println!("Path has changed? Retrying pathfinding.");
            cmd.entity(*actor).remove::<Path>();
            continue;
        };

        let block_flags = get_block_flags(&terrain, next_block[0], next_block[1], next_block[2]);

        if block_flags & path.flags == NavigationFlags::NONE {
            println!("Path block flags have changed? Retrying pathfinding.");
            cmd.entity(*actor).remove::<Path>();
            continue;
        }

        cmd.entity(*actor).insert(BlockMove {
            speed: 8.,
            target: path.blocks[path.current_block_idx],
        });
    }
}
