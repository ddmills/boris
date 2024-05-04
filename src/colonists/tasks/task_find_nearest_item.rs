use std::collections::VecDeque;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Query, Res},
    },
    utils::hashbrown::HashSet,
};
use task_derive::TaskBuilder;

use crate::{
    colonists::{
        test_item_tags, Actor, ActorRef, Blackboard, InInventory, InSlot, Item, ItemTag,
        NavigationGraph, TaskBuilder, TaskState,
    },
    Position,
};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskFindNearestItem(pub Vec<ItemTag>);

pub fn task_find_nearest_item(
    graph: Res<NavigationGraph>,
    mut q_items: Query<(&Position, &mut Item), (Without<InInventory>, Without<InSlot>)>,
    q_actors: Query<&Position, With<Actor>>,
    mut q_behavior: Query<(
        &ActorRef,
        &mut TaskState,
        &mut Blackboard,
        &TaskFindNearestItem,
    )>,
) {
    for (ActorRef(actor), mut state, mut blackboard, task) in q_behavior.iter_mut() {
        blackboard.item = None;

        let Ok(position) = q_actors.get(*actor) else {
            *state = TaskState::Failed;
            continue;
        };

        let Some(start_id) = position.partition_id else {
            println!("Item cannot be found because seeker is not in a partition!");
            *state = TaskState::Failed;
            continue;
        };

        let Some(items) = find_nearest(start_id, task.0.clone(), &graph, &q_items) else {
            println!("No nearby item with matching tags");
            for tag in task.0.clone() {
                println!("- tag {}", tag);
            }
            *state = TaskState::Failed;
            continue;
        };

        let item_entity = items.first().unwrap();

        let Ok((item_position, mut item)) = q_items.get_mut(*item_entity) else {
            println!("Item without transform? Or stale item data");
            *state = TaskState::Failed;
            continue;
        };

        item.reserved = Some(*actor);
        blackboard.item = Some(*item_entity);

        let target_pos = item_position.as_array();
        blackboard.move_goals = vec![target_pos];
        blackboard.primary_goal = Some(target_pos);
        *state = TaskState::Success;
    }
}

fn find_nearest(
    start_id: u32,
    tags: Vec<ItemTag>,
    graph: &NavigationGraph,
    q_items: &Query<(&Position, &mut Item), (Without<InInventory>, Without<InSlot>)>,
) -> Option<Vec<Entity>> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start_id);

    let max_depth = 10000;

    while let Some(partition_id) = queue.pop_front() {
        if visited.len() >= max_depth {
            println!(
                "max item search depth exceeded. Searching for {} in {}",
                tags.first().unwrap(),
                partition_id
            );
            return None;
        }

        visited.insert(partition_id);

        let Some(partition) = graph.get_partition(&partition_id) else {
            continue;
        };

        let matching_items: Vec<Entity> = partition
            .items
            .iter()
            .filter(|i| {
                let Ok((_, item)) = q_items.get(**i) else {
                    return false;
                };

                if item.reserved.is_some() {
                    return false;
                }

                test_item_tags(&item.tags, &tags)
            })
            .cloned()
            .collect();

        if !matching_items.is_empty() {
            return Some(matching_items);
        }

        for neighbor_id in partition.neighbor_ids.iter() {
            if !visited.contains(neighbor_id) {
                queue.push_back(*neighbor_id)
            }
        }
    }

    None
}
