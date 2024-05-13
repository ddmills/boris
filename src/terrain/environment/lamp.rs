use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::Changed,
        removal_detection::RemovedComponents,
        system::{Query, ResMut, Resource},
    },
    utils::HashMap,
};

use crate::{LampDetail, Position, Terrain};

#[derive(Component)]
pub struct Lamp {
    pub value: u8,
    pub offset: [i32; 3],
}

#[derive(Resource, Default)]
pub struct Lamps(pub HashMap<Entity, [u32; 2]>);

fn update_lamp(
    entity: &Entity,
    position: &Position,
    lamp: &Lamp,
    terrain: &mut Terrain,
    lamps: &mut Lamps,
) {
    if let Some([chunk_idx, block_idx]) = lamps.0.remove(entity) {
        terrain.remove_lamp(chunk_idx, block_idx, entity);
    };

    let pos_i32 = [
        position.x as i32 + lamp.offset[0],
        position.y as i32 + lamp.offset[1],
        position.z as i32 + lamp.offset[2],
    ];

    if terrain.is_oob(pos_i32[0], pos_i32[1], pos_i32[2]) {
        println!("Lamp out of bounds");
        return;
    }

    let [chunk_idx, block_idx] =
        terrain.get_block_indexes(pos_i32[0] as u32, pos_i32[1] as u32, pos_i32[2] as u32);

    lamps.0.insert(*entity, [chunk_idx, block_idx]);

    terrain.add_lamp(
        chunk_idx,
        block_idx,
        *entity,
        LampDetail {
            torchlight: lamp.value,
        },
    );
}

pub fn on_moved_lamp(
    q_moved: Query<(Entity, &Position, &Lamp), Changed<Position>>,
    q_changed: Query<(Entity, &Position, &Lamp), Changed<Lamp>>,
    mut lamps: ResMut<Lamps>,
    mut terrain: ResMut<Terrain>,
) {
    for (entity, position, lamp) in q_moved.iter() {
        update_lamp(&entity, position, lamp, &mut terrain, &mut lamps);
    }
    for (entity, position, lamp) in q_changed.iter() {
        update_lamp(&entity, position, lamp, &mut terrain, &mut lamps);
    }
}

pub fn on_removed_lamp(
    mut q_removed: RemovedComponents<Lamp>,
    mut lamps: ResMut<Lamps>,
    mut terrain: ResMut<Terrain>,
) {
    for entity in q_removed.read() {
        let Some([chunk_idx, block_idx]) = lamps.0.remove(&entity) else {
            println!("Expected a lamp to be registered, but it isn't in the lamps resource!");
            continue;
        };

        terrain.remove_lamp(chunk_idx, block_idx, &entity);
    }
}
