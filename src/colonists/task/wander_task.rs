use bevy::ecs::component::Component;
use bevy::prelude::*;
use big_brain::prelude::*;

use crate::colonists::{Agent, PartitionFlags, PartitionGraph, PartitionPathRequest, Path};
use crate::common::Rand;
use crate::Terrain;

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct PickWanderSpotAct;

pub fn wander_action_system(
    mut commands: Commands,
    q_agents: Query<&Transform, With<Agent>>,
    mut query: Query<(&Actor, &mut ActionState), With<PickWanderSpotAct>>,
    graph: Res<PartitionGraph>,
    terrain: Res<Terrain>,
    mut rand: ResMut<Rand>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        let Ok(transform) = q_agents.get(*actor) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                println!("wander requested");
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                println!("wander executing");
                let pos = [
                    transform.translation.x as u32,
                    transform.translation.y as u32,
                    transform.translation.z as u32,
                ];
                let current_partition_id = terrain.get_partition_id_u32(pos[0], pos[1], pos[2]);
                let Some(current_partition) = graph.get_partition(current_partition_id) else {
                    *state = ActionState::Failure;
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

                println!("wander requesting path");
                commands.entity(*actor).insert(PartitionPathRequest {
                    start: pos,
                    goals: vec![target_pos],
                    flags: PartitionFlags::TALL | PartitionFlags::LADDER,
                });
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                println!("wander cancelled");
                *state = ActionState::Failure;
                commands.entity(*actor).remove::<PartitionPathRequest>();
            }
            ActionState::Failure => {
                println!("wander failed");
            }
            ActionState::Success => {
                println!("wander succeeded");
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct WanderScorer;

pub fn wander_scorer_system(mut q_ai: Query<&mut Score, With<WanderScorer>>) {
    for mut score in &mut q_ai {
        score.set(0.1);
    }
}
