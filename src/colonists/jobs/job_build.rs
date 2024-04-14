use bevy::ecs::{
    event::{Event, EventReader},
    system::{Commands, ResMut},
};

use crate::{BlockType, Terrain};

use super::{Job, JobBuild, JobLocation, JobType};

#[derive(Event)]
pub struct SpawnJobBuildEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_job_build(
    mut cmd: Commands,
    mut terrain: ResMut<Terrain>,
    mut ev_spawn_job_mine: EventReader<SpawnJobBuildEvent>,
) {
    for ev in ev_spawn_job_mine.read() {
        terrain.set_block_type(ev.pos[0], ev.pos[1], ev.pos[2], BlockType::BLUEPRINT);

        cmd.spawn((
            Job {
                job_type: JobType::BuildWall,
                assignee: None,
            },
            JobBuild,
            JobLocation { pos: ev.pos },
        ));
    }
}
