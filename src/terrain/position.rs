use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::Changed,
        system::{Query, ResMut},
    },
    transform::components::GlobalTransform,
};

use crate::{
    colonists::{InInventory, NavigationGraph},
    Terrain,
};

#[derive(Component, Default)]
pub struct Position {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub chunk_idx: u32,
    pub block_idx: u32,
    pub partition_id: Option<u32>,
}

pub fn update_positions(
    mut graph: ResMut<NavigationGraph>,
    mut terrain: ResMut<Terrain>,
    mut q_changed: Query<(Entity, &mut Position, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (entity, mut position, transform) in q_changed.iter_mut() {
        let translation = transform.translation();
        let x = translation.x as u32;
        let y = translation.y as u32;
        let z = translation.z as u32;

        if x != position.x || y != position.y || z != position.z {
            position.x = x;
            position.y = y;
            position.z = z;

            let [chunk_idx, block_idx] = terrain.get_block_indexes(x, y, z);

            if chunk_idx != position.chunk_idx || block_idx != position.block_idx {
                terrain.remove_item(position.chunk_idx, position.block_idx, &entity);
                terrain.add_item(chunk_idx, block_idx, entity);
            }

            position.chunk_idx = chunk_idx;
            position.block_idx = block_idx;

            let partition_id = terrain.get_partition_id(chunk_idx, block_idx);

            if partition_id != position.partition_id {
                if let Some(current_partition_id) = position.partition_id {
                    graph.remove_item_from_partition(&current_partition_id, &entity);
                }

                if let Some(new_partition_id) = partition_id {
                    graph.add_item_to_partition(&new_partition_id, entity);
                }

                position.partition_id = partition_id;
            }

            println!("position changed. {}, {}, {}", x, y, z);
        }
    }
}
