use bevy::ecs::{
    component::Component,
    event::{Event, EventReader},
    system::{Commands, ResMut},
};

use crate::Terrain;

use super::{Job, JobLocation, JobType};

#[derive(Event)]
pub struct SpawnJobMineEvent {
    pub pos: [u32; 3],
}

#[derive(Component, Clone, Copy)]
pub struct JobMine;

pub fn on_spawn_job_mine(
    mut terrain: ResMut<Terrain>,
    mut cmd: Commands,
    mut ev_spawn_job_mine: EventReader<SpawnJobMineEvent>,
) {
    for ev in ev_spawn_job_mine.read() {
        let block = terrain.get_block(ev.pos[0], ev.pos[1], ev.pos[2]);

        if !block.is_mineable() {
            continue;
        }

        let is_changed = terrain.set_flag_mine(ev.pos[0], ev.pos[1], ev.pos[2], true);

        if !is_changed {
            continue;
        }

        cmd.spawn((
            Job {
                job_type: JobType::Mine,
                assignee: None,
            },
            JobMine,
            JobLocation {
                targets: vec![ev.pos],
                primary_target: ev.pos,
                last_accessibility_check: 0.,
            },
        ));
    }
}
