use bevy::ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
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
pub struct JobAssignment {
    pub job: Entity,
}

pub fn job_accessibility(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    q_jobs: Query<(Entity, &Job, &JobLocation)>,
) {
    for (entity, job, job_location) in q_jobs.iter() {
        if job.assignee.is_some() {
            continue;
        }

        let goals = job_access_points(job_location.pos, job.job_type);

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
