use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    time::Time,
};

use crate::{BlockType, Terrain};

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum JobType {
    Mine,
    Chop,
    PlaceBlock(BlockType),
    Build,
    Supply,
}

#[derive(Component, Clone, Copy)]
pub struct Job {
    pub job_type: JobType,
    pub assignee: Option<Entity>,
}

#[derive(Component)]
pub struct JobLocation {
    pub targets: Vec<[u32; 3]>,
    pub primary_target: [u32; 3],
    pub source: Option<[u32; 3]>,
    pub last_accessibility_check: f32,
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
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut q_jobs: Query<
        (Entity, &Job, &mut JobLocation),
        (Without<IsJobCancelled>, Without<IsJobCompleted>),
    >,
) {
    let now = time.elapsed_seconds();

    for (entity, job, mut job_location) in q_jobs.iter_mut() {
        let time_since_last_check = now - job_location.last_accessibility_check;

        if time_since_last_check < 2.0 {
            continue;
        }

        job_location.last_accessibility_check = now;

        let goals = job_access_points_many(&job_location.targets, job.job_type);

        let is_accessible = goals
            .iter()
            .any(|g| terrain.get_partition_id_u32(g[0], g[1], g[2]).is_some());

        if is_accessible {
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

#[derive(Event)]
pub struct JobCancelEvent(pub Entity);

pub fn on_cancel_job(
    mut cmd: Commands,
    job_holders: Query<Entity>,
    mut ev_job_cancel: EventReader<JobCancelEvent>,
    mut q_jobs: Query<&mut Job>,
) {
    for JobCancelEvent(entity) in ev_job_cancel.read() {
        println!("cancel job {}", entity.index());

        let Ok(mut job) = q_jobs.get_mut(*entity) else {
            println!("ERR: job does not exist!? {}", entity.index());
            continue;
        };

        if let Some(job_assignee) = job.assignee {
            if let Ok(holder) = job_holders.get(job_assignee) {
                cmd.entity(holder).remove::<JobAssignment>();
            } else {
                println!("ERR: no holder for job!?");
            };
        }

        cmd.entity(*entity).insert(IsJobCancelled);
        job.assignee = None;
    }
}

pub fn job_access_points_many(targets: &[[u32; 3]], job: JobType) -> Vec<[u32; 3]> {
    let points = targets.iter().flat_map(|t| job_access_points(*t, job));

    if matches!(
        job,
        JobType::Build | JobType::PlaceBlock(_) | JobType::Supply
    ) {
        return points
            .filter(|p| {
                !targets
                    .iter()
                    .any(|t| t[0] == p[0] && t[1] == p[1] && t[2] == p[2])
            })
            .collect::<Vec<_>>();
    }

    points.collect::<Vec<_>>()
}

pub fn job_access_points(pos: [u32; 3], job: JobType) -> Vec<[u32; 3]> {
    let [x, y, z] = pos;

    match job {
        JobType::Chop => {
            let mut goals = vec![[x + 1, y, z], [x, y, z + 1], [x + 1, y, z + 1]];

            if x > 0 {
                goals.push([x - 1, y, z]);
                goals.push([x - 1, y, z + 1]);

                if z > 0 {
                    goals.push([x - 1, y, z - 1]);
                    goals.push([x + 1, y, z - 1]);
                    goals.push([x, y, z - 1]);
                }
            }

            goals
        }
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
        JobType::PlaceBlock(_) | JobType::Build | JobType::Supply => {
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

            if y > 0 {
                goals.push([x, y - 1, z]);
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
