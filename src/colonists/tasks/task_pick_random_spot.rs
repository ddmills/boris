use bevy::ecs::{
    component::Component,
    query::With,
    system::{Query, Res, ResMut},
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        Actor, ActorRef, Blackboard, NavigationFlags, NavigationGraph, TaskBuilder, TaskState,
    },
    common::Rand,
    Position, Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskPickRandomSpot;

pub fn task_pick_random_spot(
    mut rand: ResMut<Rand>,
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    q_positions: Query<&Position, With<Actor>>,
    mut q_behavior: Query<(&ActorRef, &mut Blackboard, &mut TaskState), With<TaskPickRandomSpot>>,
) {
    for (ActorRef(actor), mut blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(position) = q_positions.get(*actor) else {
            println!("no transform on actor, cannot pick random spot!");
            *state = TaskState::Failed;
            continue;
        };

        let pos = [position.x, position.y, position.z];

        let Some(current_partition_id) = terrain.get_partition_id_u32(pos[0], pos[1], pos[2])
        else {
            *state = TaskState::Failed;
            return;
        };

        let Some(current_partition) = graph.get_partition(&current_partition_id) else {
            *state = TaskState::Failed;
            return;
        };

        let target_partition_id = if current_partition.neighbor_ids.is_empty() {
            current_partition_id
        } else {
            let neighbor_ids: Vec<u32> = current_partition
                .neighbor_ids
                .iter()
                .filter_map(|n| {
                    let p = graph.get_partition(n)?;

                    if !p
                        .flags
                        .intersects(NavigationFlags::SHORT | NavigationFlags::TALL)
                    {
                        return None;
                    }

                    Some(*n)
                })
                .collect();

            if neighbor_ids.is_empty() {
                current_partition_id
            } else {
                rand.pick(&neighbor_ids)
            }
        };

        let target_partition = graph.get_partition(&target_partition_id).unwrap();
        let blocks = &target_partition.blocks.iter().collect::<Vec<_>>();

        if blocks.is_empty() {
            *state = TaskState::Failed;
            return;
        }

        let target_block_idx = rand.pick(blocks);
        let target_chunk_idx = target_partition.chunk_idx;

        let target_pos = terrain.get_block_world_pos(target_chunk_idx, *target_block_idx);

        blackboard.move_goals = vec![target_pos];
        blackboard.primary_goal = Some(target_pos);

        *state = TaskState::Success;
    }
}
