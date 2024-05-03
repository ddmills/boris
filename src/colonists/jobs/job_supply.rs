use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    system::{Commands, Query},
};

use crate::{colonists::ItemTag, Position};

use super::{Job, JobLocation, JobType};

#[derive(Event)]
pub struct SpawnJobSupplyEvent {
    pub item: Entity,
    pub target: Entity,
}

#[derive(Component, Clone)]
pub struct JobSupply {
    pub tags: Vec<ItemTag>,
    pub slot_target_idx: usize,
    pub target: Entity,
}

pub fn on_spawn_job_supply(
    mut cmd: Commands,
    mut ev_spawn_job_supply: EventReader<SpawnJobSupplyEvent>,
    q_positions: Query<&Position>,
) {
    // for ev in ev_spawn_job_supply.read() {
    //     let Ok(item_pos) = q_positions.get(ev.item) else {
    //         println!("Cannot supply because item does not have a position");
    //         continue;
    //     };

    //     let Ok(target_pos) = q_positions.get(ev.target) else {
    //         println!("Cannot supply because target does not have a position");
    //         continue;
    //     };

    //     cmd.spawn((
    //         Job {
    //             job_type: JobType::Chop,
    //             assignee: None,
    //         },
    //         JobSupply {
    //             item: ev.item,
    //             target: ev.target,
    //         },
    //         JobLocation {
    //             targets: vec![target_pos.as_vec()],
    //             primary_target: target_pos.as_vec(),
    //             source: Some(item_pos.as_vec()),
    //             last_accessibility_check: 0.,
    //         },
    //     ));
    // }
}
