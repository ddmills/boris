use std::collections::VecDeque;

use bevy::ecs::system::Resource;

#[derive(Clone, Copy)]
pub enum Job {
    Mine([u32; 3]),
}

#[derive(Resource)]
pub struct JobList {
    pub jobs: VecDeque<Job>,
}

impl JobList {
    pub fn queue(&mut self, job: Job) {
        self.jobs.push_back(job);
    }

    pub fn pop(&mut self) -> Option<Job> {
        self.jobs.pop_front()
    }
}
