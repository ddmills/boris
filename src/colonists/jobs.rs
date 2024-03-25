use std::collections::VecDeque;

use bevy::ecs::{component::Component, entity::Entity, system::Resource};

#[derive(Clone, Copy, PartialEq)]
pub enum JobType {
    Mine([u32; 3]),
}

#[derive(Component, Clone, Copy)]
pub struct Job {
    pub job_type: JobType,
}

#[derive(Component)]
pub struct IsJobAssigned {
    pub assignee: Entity,
}
#[derive(Component)]
pub struct JobAssignment {
    pub job: Entity,
}
