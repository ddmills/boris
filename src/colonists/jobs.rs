use bevy::ecs::{component::Component, entity::Entity};

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum JobType {
    Mine,
}

#[derive(Component, Clone, Copy)]
pub struct JobMine;

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
