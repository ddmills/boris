use bevy::ecs::{
    event::{Event, EventReader},
    system::{Commands, Query, ResMut},
};

use crate::{Terrain, Tree};

use super::{Job, JobChop, JobLocation, JobType};

#[derive(Event)]
pub struct SpawnJobChopEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_job_chop(
    mut terrain: ResMut<Terrain>,
    mut cmd: Commands,
    mut ev_spawn_job_chop: EventReader<SpawnJobChopEvent>,
    q_trees: Query<&Tree>,
) {
    for ev in ev_spawn_job_chop.read() {
        let [chunk_idx, block_idx] = terrain.get_block_indexes(ev.pos[0], ev.pos[1], ev.pos[2]);
        let trees = terrain.get_trees(chunk_idx, block_idx);

        for tree_entity in trees {
            let Ok(tree) = q_trees.get(tree_entity) else {
                println!("Job Chop for tree that doesn't exist anymore?");
                continue;
            };

            let mut chop = true;

            for part in tree.trunk.iter() {
                let is_changed = terrain.set_flag_chop(part[0], part[1], part[2], true);

                if !is_changed {
                    chop = false;
                    continue;
                }
            }

            if chop {
                cmd.spawn((
                    Job {
                        job_type: JobType::Chop,
                        assignee: None,
                    },
                    JobChop { tree: tree_entity },
                    JobLocation {
                        targets: tree.trunk.clone(),
                        primary_target: ev.pos,
                    },
                ));
            }
        }
    }
}
