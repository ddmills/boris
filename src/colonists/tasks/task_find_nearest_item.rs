use std::collections::VecDeque;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Query, Res},
    },
    transform::components::Transform,
    utils::hashbrown::HashSet,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        test_item_tags, Actor, ActorRef, Blackboard, Item, ItemTag, Partition, PartitionGraph,
        TaskBuilder, TaskState,
    },
    Terrain,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskFindNearestItem(pub Vec<ItemTag>);

pub fn task_find_nearest_item(
    terrain: Res<Terrain>,
    graph: Res<PartitionGraph>,
    q_items: Query<(&Transform, &Item)>,
    q_actors: Query<&Transform, With<Actor>>,
    mut q_behavior: Query<(
        &ActorRef,
        &mut TaskState,
        &mut Blackboard,
        &TaskFindNearestItem,
    )>,
) {
    for (ActorRef(actor), mut state, mut blackboard, task) in q_behavior.iter_mut() {
        blackboard.item = None;

        let Ok(transform) = q_actors.get(*actor) else {
            *state = TaskState::Failed;
            continue;
        };

        let actor_x = transform.translation.x as u32;
        let actor_y = transform.translation.y as u32;
        let actor_z = transform.translation.z as u32;

        let start_id = terrain.get_partition_id_u32(actor_x, actor_y, actor_z);

        if start_id == Partition::NONE {
            println!("Item cannot be found because seeker is not in a partition!");
            *state = TaskState::Failed;
            continue;
        };

        let Some(items) = find_nearest(start_id, task.0.clone(), &graph, &q_items) else {
            println!("No nearby item with matching tags");
            *state = TaskState::Failed;
            continue;
        };

        println!("found {} matching items!", items.len());

        let item = items.first().unwrap();

        let Ok((item_tansform, _)) = q_items.get(*item) else {
            println!("Item without transform? Or stale item data");
            *state = TaskState::Failed;
            continue;
        };

        let item_pos = [
            item_tansform.translation.x as u32,
            item_tansform.translation.y as u32,
            item_tansform.translation.z as u32,
        ];

        blackboard.item = Some(*item);
        blackboard.move_goals = vec![item_pos];
        *state = TaskState::Success;
    }
}

pub fn find_nearest(
    start_id: u16,
    tags: Vec<ItemTag>,
    graph: &PartitionGraph,
    q_items: &Query<(&Transform, &Item)>,
) -> Option<Vec<Entity>> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start_id);

    while let Some(partition_id) = queue.pop_front() {
        visited.insert(partition_id);

        let Some(partition) = graph.get_partition(partition_id) else {
            continue;
        };

        let matching_items: Vec<Entity> = partition
            .items
            .iter()
            .filter(|i| {
                let Ok((_, item)) = q_items.get(**i) else {
                    return false;
                };

                test_item_tags(&item.tags, &tags)
            })
            .cloned()
            .collect();

        if !matching_items.is_empty() {
            return Some(matching_items);
        }

        for neighbor_id in partition.neighbors.iter() {
            if !visited.contains(neighbor_id) {
                queue.push_back(*neighbor_id)
            }
        }
    }

    None
}
