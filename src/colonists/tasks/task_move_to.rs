use bevy::ecs::{
    component::Component,
    query::With,
    system::{Commands, Query, Res},
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        get_block_flags, get_granular_path, get_partition_path, is_reachable, Actor, ActorRef,
        AnimClip, Animator, Blackboard, BlockMove, GranularPathRequest, NavigationFlags,
        NavigationGraph, PartitionPathRequest, Path, TaskBuilder, TaskState,
    },
    Position, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskMoveTo {
    attempts: u8,
    max_retries: u8,
}

impl Default for TaskMoveTo {
    fn default() -> Self {
        Self {
            attempts: 0,
            max_retries: 4,
        }
    }
}

pub fn task_move_to(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    mut q_paths: Query<&mut Path, With<Actor>>,
    q_movers: Query<&BlockMove, With<Actor>>,
    mut q_animators: Query<&mut Animator, With<Actor>>,
    q_positions: Query<&Position, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &Blackboard, &mut TaskState, &mut TaskMoveTo)>,
) {
    for (ActorRef(actor), blackboard, mut state, mut move_to) in q_behavior.iter_mut() {
        let Ok(position) = q_positions.get(*actor) else {
            println!("no position on actor, cannot move to!");
            cmd.entity(*actor).remove::<Path>();
            *state = TaskState::Failed;
            continue;
        };

        if q_movers.contains(*actor) {
            continue;
        }

        let pos = [position.x, position.y, position.z];

        let Ok(mut path) = q_paths.get_mut(*actor) else {
            if blackboard.move_goals.is_empty() {
                println!("no move_goals on blackboard, cannot move to!");
                *state = TaskState::Failed;
                continue;
            }

            let request = PartitionPathRequest {
                start: pos,
                goals: blackboard.move_goals.clone(),
                flags: NavigationFlags::COLONIST,
            };

            let Some(partition_path) = get_partition_path(&request, &terrain, &graph) else {
                if !is_reachable(&request, &terrain, &graph) {
                    *state = TaskState::Failed;
                    continue;
                }

                move_to.attempts += 1;
                println!(
                    "Failed to find partition path, should be reachable! entity={} attempts={}",
                    actor.index(),
                    move_to.attempts,
                );

                if move_to.attempts >= move_to.max_retries {
                    *state = TaskState::Failed;
                }

                continue;
            };

            let path = Path {
                current_partition_idx: partition_path.path.len() - 1,
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
        let Some(partition_id) = position.partition_id else {
            println!("Not standing in a partition, cannot path!");
            cmd.entity(*actor).remove::<Path>();
            *state = TaskState::Failed;
            continue;
        };

        let partition_path_idx = path.partition_path.iter().position(|p| *p == partition_id);

        if let Some(idx) = partition_path_idx {
            path.current_partition_idx = idx;
        };

        // if current block index is zero, it means we've finished the granular path
        if path.current_block_idx == 0 {
            let Some(next_partition_id) = path.next_partition_id() else {
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
                    partition_path: path.partition_path.clone(),
                    flags: NavigationFlags::COLONIST,
                },
            ) else {
                move_to.attempts += 1;
                cmd.entity(*actor).remove::<Path>();

                if move_to.attempts >= move_to.max_retries {
                    println!(
                        "Granular path attempts failed! attempts={}",
                        move_to.attempts
                    );
                    *state = TaskState::Failed;
                }

                continue;
            };

            path.blocks = granular_path.blocks.clone();
            path.current_block_idx = path.blocks.len() - 1;
        }

        path.current_block_idx -= 1;

        let Some(next_block) = path.next_block() else {
            cmd.entity(*actor).remove::<Path>();
            continue;
        };

        let block_flags = get_block_flags(&terrain, next_block[0], next_block[1], next_block[2]);

        if block_flags & path.flags == NavigationFlags::NONE {
            cmd.entity(*actor).remove::<Path>();
            continue;
        }

        if let Ok(mut animator) = q_animators.get_mut(*actor) {
            animator.clip = AnimClip::Run;
        };

        cmd.entity(*actor).insert(BlockMove {
            speed: 4.,
            target: path.blocks[path.current_block_idx],
            look_at: true,
        });
    }
}
