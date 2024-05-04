use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    system::{Commands, Query},
};

use crate::structures::{PartSlots, Structure};

use super::{Job, JobLocation, SpawnJobSupplyEvent};

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
            .map(|t| {
                [
                    t.position[0] as u32,
                    t.position[1] as u32,
                    t.position[2] as u32,
                ]
            })
            .collect::<Vec<_>>();

        for (idx, slot) in part_slots.slots.iter().enumerate() {
            ev_spawn_job_supply.send(SpawnJobSupplyEvent {
                flags: slot.flags.clone(),
                slot_taget_idx: idx,
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
