use bevy::ecs::{
    event::{Event, EventReader},
    system::{Commands, ResMut},
};

use crate::Terrain;

use super::{Job, JobLocation, JobMine, JobType};

#[derive(Event)]
pub struct SpawnJobMineEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_job_mine(
    mut terrain: ResMut<Terrain>,
    mut cmd: Commands,
    mut ev_spawn_job_mine: EventReader<SpawnJobMineEvent>,
) {
    for ev in ev_spawn_job_mine.read() {
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
            JobLocation { pos: ev.pos },
        ));
    }
}
