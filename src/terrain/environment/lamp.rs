use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    system::{Commands, ResMut},
};

use crate::{LampDetail, Terrain};

#[derive(Event)]
pub struct SpawnLampEvent {
    pub value: u8,
    pub pos: [u32; 3],
    pub entity: Entity,
}

#[derive(Component)]
pub struct Lamp;

pub fn on_spawn_lamp(
    mut cmd: Commands,
    mut ev_spawn_lamp: EventReader<SpawnLampEvent>,
    mut terrain: ResMut<Terrain>,
) {
    for ev in ev_spawn_lamp.read() {
        let mut ecmd = cmd.entity(ev.entity);

        println!("Spawn lamp");

        ecmd.insert(Lamp);

        let [chunk_idx, block_idx] = terrain.get_block_indexes(ev.pos[0], ev.pos[1], ev.pos[2]);

        terrain.add_lamp(
            chunk_idx,
            block_idx,
            ev.entity,
            LampDetail {
                torchlight: ev.value,
            },
        );
    }
}
