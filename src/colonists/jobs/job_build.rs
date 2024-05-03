use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    system::{Commands, Query},
};

use crate::furniture::{Blueprint, BlueprintSlot, BlueprintSlots};

use super::{Job, JobLocation, JobSupply};

#[derive(Event)]
pub struct SpawnJobBuildEvent {
    pub blueprint: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct JobBuild {
    pub blueprint: Entity,
}

pub fn on_spawn_job_build(
    mut cmd: Commands,
    mut ev_spawn_job_build: EventReader<SpawnJobBuildEvent>,
    q_blueprints: Query<(&Blueprint, &BlueprintSlots)>,
) {
    for ev in ev_spawn_job_build.read() {
        let Ok((blueprint, blueprint_slots)) = q_blueprints.get(ev.blueprint) else {
            println!("blueprint doesn't exist? Cannot build");
            continue;
        };

        let targets = blueprint
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

        for (idx, slot) in blueprint_slots.slots.iter().enumerate() {
            println!(
                "spawning job to set slot{} {}",
                idx,
                slot.tags.first().unwrap()
            );
            cmd.spawn((
                Job {
                    job_type: super::JobType::Supply,
                    assignee: None,
                },
                JobSupply {
                    tags: slot.tags.clone(),
                    slot_target_idx: idx,
                    target: ev.blueprint,
                },
                JobLocation {
                    targets: targets.clone(),
                    primary_target: blueprint.position,
                    last_accessibility_check: 0.,
                    source: None,
                },
            ));
        }

        cmd.spawn((
            Job {
                job_type: super::JobType::Build,
                assignee: None,
            },
            JobBuild {
                blueprint: ev.blueprint,
            },
            JobLocation {
                targets,
                primary_target: blueprint.position,
                last_accessibility_check: 0.,
                source: None,
            },
        ));
    }
}
