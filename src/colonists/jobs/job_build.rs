use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    query::Without,
    system::{Commands, Query},
};

use crate::{
    rendering::SlotIndex,
    structures::{PartSlots, Structure, TileRequirement},
};

use super::{IsJobCancelled, Job, JobCancelEvent, JobLocation, SpawnJobSupplyEvent};

#[derive(Event)]
pub struct SpawnJobBuildEvent {
    pub structure: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct JobBuild {
    pub structure: Entity,
}

pub fn on_spawn_job_build(
    mut cmd: Commands,
    mut ev_spawn_job_build: EventReader<SpawnJobBuildEvent>,
    mut ev_spawn_job_supply: EventWriter<SpawnJobSupplyEvent>,
    q_structures: Query<(&Structure, &PartSlots)>,
) {
    for ev in ev_spawn_job_build.read() {
        let Ok((structure, part_slots)) = q_structures.get(ev.structure) else {
            println!("structure doesn't exist? Cannot build");
            continue;
        };

        let targets = structure
            .tiles
            .iter()
            .filter_map(|t| {
                if t.is_blocker || t.requirements.contains(TileRequirement::IS_ATTACHABLE) {
                    Some([
                        t.position[0] as u32,
                        t.position[1] as u32,
                        t.position[2] as u32,
                    ])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if let Some(slot) = part_slots.slot_0.as_ref() {
            ev_spawn_job_supply.send(SpawnJobSupplyEvent {
                flags: slot.flags.clone(),
                slot_taget_idx: SlotIndex::Slot0,
                target: ev.structure,
                targets: targets.clone(),
                primary_target: structure.position,
            });
        }

        if let Some(slot) = part_slots.slot_1.as_ref() {
            ev_spawn_job_supply.send(SpawnJobSupplyEvent {
                flags: slot.flags.clone(),
                slot_taget_idx: SlotIndex::Slot1,
                target: ev.structure,
                targets: targets.clone(),
                primary_target: structure.position,
            });
        }

        if let Some(slot) = part_slots.slot_2.as_ref() {
            ev_spawn_job_supply.send(SpawnJobSupplyEvent {
                flags: slot.flags.clone(),
                slot_taget_idx: SlotIndex::Slot2,
                target: ev.structure,
                targets: targets.clone(),
                primary_target: structure.position,
            });
        }

        cmd.spawn((
            Job {
                job_type: super::JobType::Build,
                assignee: None,
            },
            JobBuild {
                structure: ev.structure,
            },
            JobLocation {
                targets,
                primary_target: structure.position,
                last_accessibility_check: 0.,
                source: None,
            },
        ));
    }
}

pub fn check_job_build_valid(
    q_jobs: Query<(Entity, &JobBuild), Without<IsJobCancelled>>,
    q_targets: Query<&Structure>,
    mut ev_job_cancel: EventWriter<JobCancelEvent>,
) {
    for (entity, job_build) in q_jobs.iter() {
        let Ok(_) = q_targets.get(job_build.structure) else {
            ev_job_cancel.send(JobCancelEvent(entity));
            continue;
        };
    }
}
