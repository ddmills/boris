use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    system::{Commands, Query},
};

use crate::furniture::Blueprint;

use super::{Job, JobLocation};

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
    q_blueprints: Query<&Blueprint>,
) {
    for ev in ev_spawn_job_build.read() {
        let Ok(blueprint) = q_blueprints.get(ev.blueprint) else {
            println!("blueprint doesn't exist? Cannot build");
            continue;
        };

        let targets = blueprint
            .tiles
            .iter()
            .map(|t| [t[0] as u32, t[1] as u32, t[2] as u32])
            .collect::<Vec<_>>();

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
            },
        ));
    }
}
