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
        let flagged = terrain.set_flag_blueprint(ev.pos[0], ev.pos[1], ev.pos[2], true);

        if !flagged {
            println!("already building?");
            continue;
        }

        terrain.set_block_type(ev.pos[0], ev.pos[1], ev.pos[2], BlockType::STONE);

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
