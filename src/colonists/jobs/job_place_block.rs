use bevy::ecs::{
    component::Component,
    event::{Event, EventReader},
    system::{Commands, ResMut},
};

use crate::{BlockType, Terrain};

use super::{Job, JobLocation, JobType};

#[derive(Event)]
pub struct SpawnJobPlaceBlockEvent {
    pub pos: [u32; 3],
    pub block_type: BlockType,
}

#[derive(Component, Clone, Copy)]
pub struct JobPlaceBlock;

pub fn on_spawn_job_place_block(
    mut cmd: Commands,
    mut terrain: ResMut<Terrain>,
    mut ev_spawn_place_block_job: EventReader<SpawnJobPlaceBlockEvent>,
) {
    for ev in ev_spawn_place_block_job.read() {
        let flagged = terrain.set_flag_blueprint(ev.pos[0], ev.pos[1], ev.pos[2], true);

        if !flagged {
            println!("already building?");
            continue;
        }

        cmd.spawn((
            Job {
                job_type: JobType::PlaceBlock(ev.block_type),
                assignee: None,
            },
            JobPlaceBlock,
            JobLocation {
                targets: vec![ev.pos],
                primary_target: ev.pos,
                last_accessibility_check: 0.,
            },
        ));
    }
}
