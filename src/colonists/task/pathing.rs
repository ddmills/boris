use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    transform::components::Transform,
};
use big_brain::prelude::*;

use crate::{
    colonists::{
        get_block_flags, get_granular_path, get_partition_path, Agent, BlockMove,
        GranularPathRequest, PartitionFlags, PartitionGraph, Path,
    },
    Terrain,
};

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct GeneratePathAct;

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct FollowPathAct;

#[derive(Clone, Component, Debug)]
pub struct PartitionPathRequest {
    pub start: [u32; 3],
    pub goals: Vec<[u32; 3]>,
    pub flags: PartitionFlags,
}

pub fn generate_path_action_system(
    mut commands: Commands,
    q_agents: Query<&PartitionPathRequest, With<Agent>>,
    mut q_ai: Query<(&Actor, &mut ActionState), With<GeneratePathAct>>,
    graph: Res<PartitionGraph>,
    terrain: Res<Terrain>,
) {
    for (Actor(actor), mut state) in q_ai.iter_mut() {
        let Ok(request) = q_agents.get(*actor) else {
            let str = match *state {
                ActionState::Init => "Init",
                ActionState::Requested => "Requested",
                ActionState::Executing => "Executing",
                ActionState::Cancelled => "Cancelled",
                ActionState::Success => "Success",
                ActionState::Failure => "Failure",
            };
            println!("no part path req??? wtf {}", str);
            // commands.entity(*actor).remove::<PartitionPathRequest>();
            // commands.entity(*actor).remove::<Path>();
            // *state = ActionState::Failure;

            continue;
        };

        match *state {
            ActionState::Requested => {
                println!(
                    "generate path requested [{},{},{}]",
                    request.start[0], request.start[1], request.start[2]
                );
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                println!("executing generate path");
                let Some(path) = get_partition_path(request, terrain.as_ref(), graph.as_ref())
                else {
                    *state = ActionState::Cancelled;
                    continue;
                };

                println!(
                    "partition path generated, len={}, goals={}, flags={}",
                    path.path.len(),
                    path.goals.len(),
                    request.flags
                );
                let p = Path {
                    current_partition_idx: path.goals.len() - 1,
                    goals: path.goals,
                    partition_path: path.path,
                    flags: request.flags,
                    blocks: vec![],
                    current_block_idx: 0,
                };

                println!("inserting path component");
                commands.entity(*actor).insert(p);

                println!("partition path state is success");
                commands.entity(*actor).remove::<PartitionPathRequest>();
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                println!("generate path action cancelled");
                commands.entity(*actor).remove::<PartitionPathRequest>();
                commands.entity(*actor).remove::<Path>();
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

pub fn follow_path_action_system(
    mut commands: Commands,
    q_movers: Query<&BlockMove, With<Agent>>,
    mut q_agents: Query<(&mut Path, &Transform), With<Agent>>,
    mut q_ai: Query<(&Actor, &mut ActionState), With<FollowPathAct>>,
    graph: Res<PartitionGraph>,
    terrain: Res<Terrain>,
) {
    for (Actor(actor), mut state) in q_ai.iter_mut() {
        let Ok((mut path, transform)) = q_agents.get_mut(*actor) else {
            println!("no path, no transform???");
            continue;
        };

        match *state {
            ActionState::Requested => {
                println!("follow path requested! {}", path.flags);
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                // if we have a block_move, wait until it's completed
                if q_movers.contains(*actor) {
                    continue;
                }

                let pos = [
                    transform.translation.x as u32,
                    transform.translation.y as u32,
                    transform.translation.z as u32,
                ];

                // if we're at the goal, success
                let at_goal = path
                    .goals
                    .iter()
                    .any(|g| g[0] == pos[0] && g[1] == pos[1] && g[2] == pos[2]);

                if at_goal {
                    commands.entity(*actor).remove::<PartitionPathRequest>();
                    commands.entity(*actor).remove::<Path>();
                    *state = ActionState::Success;
                    continue;
                }

                // what partition are we standing in? if it's not part of the predetermined path, we stay course.
                // if it is part of the path, we set our current index to be the path idx
                let partition_id = terrain.get_partition_id_u32(pos[0], pos[1], pos[2]);
                let partition_path_idx =
                    path.partition_path.iter().position(|p| *p == partition_id);

                if let Some(idx) = partition_path_idx {
                    path.current_partition_idx = idx;
                };

                // if current block index is zero, it means we've finished the granulra path
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
                        *state = ActionState::Cancelled;
                        continue;
                    };

                    path.blocks = granular_path.blocks.clone();
                    path.current_block_idx = path.blocks.len() - 1;
                }

                path.current_block_idx -= 1;

                let next_block = path.next_block();
                let block_flags =
                    get_block_flags(&terrain, next_block[0], next_block[1], next_block[2]);

                if block_flags & path.flags == PartitionFlags::NONE {
                    println!("path has changed! it's now blocked!");
                    *state = ActionState::Cancelled;
                    continue;
                }

                commands.entity(*actor).insert(BlockMove {
                    speed: 8.,
                    target: path.blocks[path.current_block_idx],
                });
            }
            ActionState::Cancelled => {
                println!("follow path action cancelled");
                commands.entity(*actor).remove::<PartitionPathRequest>();
                commands.entity(*actor).remove::<Path>();
                *state = ActionState::Failure;
            }
            ActionState::Failure => {
                println!("follow path action failed");
            }
            ActionState::Success => {
                println!("follow path action success");
            }
            _ => {}
        }
    }
}
