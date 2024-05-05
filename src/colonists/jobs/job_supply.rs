use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    system::Commands,
};

use crate::{colonists::ItemTag, rendering::SlotIndex};

use super::{Job, JobLocation};

#[derive(Event)]
pub struct SpawnJobSupplyEvent {
    pub flags: Vec<ItemTag>,
    pub slot_taget_idx: SlotIndex,
    pub target: Entity,
    pub targets: Vec<[u32; 3]>,
    pub primary_target: [u32; 3],
}

#[derive(Component, Clone)]
pub struct JobSupply {
    pub flags: Vec<ItemTag>,
    pub slot_target_idx: SlotIndex,
    pub target: Entity,
}

pub fn on_spawn_job_supply(
    mut cmd: Commands,
    mut ev_spawn_job_supply: EventReader<SpawnJobSupplyEvent>,
) {
    for ev in ev_spawn_job_supply.read() {
        cmd.spawn((
            Job {
                job_type: super::JobType::Supply,
                assignee: None,
            },
            JobSupply {
                flags: ev.flags.clone(),
                slot_target_idx: ev.slot_taget_idx,
                target: ev.target,
            },
            JobLocation {
                targets: ev.targets.clone(),
                primary_target: ev.primary_target,
                last_accessibility_check: 0.,
                source: None,
            },
        ));
    }
}
