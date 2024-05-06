use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    query::Without,
    system::{Commands, Query},
};

use crate::{colonists::ItemTag, rendering::SlotIndex, structures::PartSlots};

use super::{IsJobCancelled, Job, JobCancelEvent, JobLocation};

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

pub fn check_job_supply_valid(
    q_jobs: Query<(Entity, &JobSupply), Without<IsJobCancelled>>,
    q_targets: Query<&PartSlots>,
    mut ev_job_cancel: EventWriter<JobCancelEvent>,
) {
    for (entity, job_supply) in q_jobs.iter() {
        let Ok(_) = q_targets.get(job_supply.target) else {
            ev_job_cancel.send(JobCancelEvent(entity));
            continue;
        };
    }
}
