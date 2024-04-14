use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Query, Res, ResMut},
    },
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{Actor, ActorRef, Blackboard, NavigationGraph, TaskBuilder, TaskState},
    common::Rand,
    Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskPickRandomSpot;

pub fn task_pick_random_spot(
    mut rand: ResMut<Rand>,
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
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
            let neighbor_ids: Vec<u32> = current_partition.neighbor_ids.iter().copied().collect();
            rand.pick(&neighbor_ids)
        };

        let target_partition = graph.get_partition(&target_partition_id).unwrap();
        let blocks = &target_partition.blocks.iter().copied().collect::<Vec<_>>();
        let target_block_idx = rand.pick(blocks);
        let target_chunk_idx = target_partition.chunk_idx;

        let target_pos = terrain.get_block_world_pos(target_chunk_idx, target_block_idx);

        blackboard.move_goals = vec![target_pos];

        *state = TaskState::Success;
    }
}
