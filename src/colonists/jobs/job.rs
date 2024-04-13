use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
};

use crate::Terrain;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum JobType {
    Mine,
    BuildWall,
}

#[derive(Component, Clone, Copy)]
pub struct JobMine;

#[derive(Component, Clone, Copy)]
pub struct JobBuild;

#[derive(Component, Clone, Copy)]
pub struct Job {
    pub job_type: JobType,
    pub assignee: Option<Entity>,
}

#[derive(Component)]
pub struct JobLocation {
    pub pos: [u32; 3],
}

#[derive(Component)]
pub struct IsJobAccessible;

#[derive(Component)]
pub struct IsJobCancelled;

#[derive(Component)]
pub struct IsJobCompleted;

#[derive(Component)]
pub struct JobAssignment {
    pub job: Entity,
}

pub fn job_accessibility(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    q_jobs: Query<(Entity, &Job, &JobLocation), (Without<IsJobCancelled>, Without<IsJobCompleted>)>,
) {
    for (entity, job, job_location) in q_jobs.iter() {
        if job.assignee.is_some() {
            continue;
        }

        let goals = job_access_points(job_location.pos, job.job_type);

        let is_accessible = goals
            .iter()
            .any(|g| terrain.get_partition_id_u32(g[0], g[1], g[2]).is_some());

        let is_filled = terrain
            .get_block(
                job_location.pos[0],
                job_location.pos[1],
                job_location.pos[2],
            )
            .is_filled();

        let is_cancelled = match job.job_type {
            JobType::Mine => {
                if !is_filled {
                    cmd.entity(entity).try_insert(IsJobCancelled);
                    true
                } else {
                    false
                }
            }
            JobType::BuildWall => {
                if is_filled {
                    cmd.entity(entity).try_insert(IsJobCancelled);
                    true
                } else {
                    false
                }
            }
        };

        if !is_cancelled && is_accessible {
            cmd.entity(entity).insert(IsJobAccessible);
        } else {
            cmd.entity(entity).remove::<IsJobAccessible>();
        }
    }
}

pub fn job_despawn_complete(mut cmd: Commands, q_jobs: Query<Entity, With<IsJobCompleted>>) {
    for e in q_jobs.iter() {
        cmd.entity(e).despawn_recursive();
    }
}

pub fn job_despawn_cancelled(mut cmd: Commands, q_jobs: Query<Entity, With<IsJobCancelled>>) {
    for e in q_jobs.iter() {
        cmd.entity(e).despawn_recursive();
    }
}

pub fn job_access_points(pos: [u32; 3], job: JobType) -> Vec<[u32; 3]> {
    let [x, y, z] = pos;

    match job {
        JobType::Mine => {
            let mut goals = vec![
                [x + 1, y, z],
                [x, y, z + 1],
                [x + 1, y + 1, z],
                [x, y + 1, z + 1],
                [x + 1, y, z + 1],
            ];

            if x > 0 {
                goals.push([x - 1, y, z]);
                goals.push([x - 1, y + 1, z]);
                goals.push([x - 1, y, z + 1]);

                if y > 0 {
                    goals.push([x - 1, y - 1, z]);
                }

                if z > 0 {
                    goals.push([x - 1, y, z - 1]);
                }
            }

            if y > 0 {
                goals.push([x + 1, y - 1, z]);
                goals.push([x, y - 1, z + 1]);

                if z > 0 {
                    goals.push([x, y - 1, z - 1]);

                    if y > 1 {
                        goals.push([x, y - 2, z - 1]);
                    }
                }

                if y > 1 {
                    goals.push([x, y - 2, z + 1]);
                }
            }

            if z > 0 {
                goals.push([x, y, z - 1]);
                goals.push([x, y + 1, z - 1]);
                goals.push([x + 1, y, z - 1]);
            }

            goals
        }
        JobType::BuildWall => {
            let mut goals = vec![
                [x + 1, y, z],
                [x + 1, y + 1, z],
                [x, y, z + 1],
                [x, y + 1, z + 1],
            ];

            if x > 0 {
                goals.push([x - 1, y, z]);
                goals.push([x - 1, y + 1, z]);

                if y > 0 {
                    goals.push([x - 1, y - 1, z]);
                }
            }

            if z > 0 {
                goals.push([x, y, z - 1]);
                goals.push([x, y + 1, z - 1]);

                if y > 0 {
                    goals.push([x, y - 1, z - 1]);
                }
            }

            goals
        }
    }
}
