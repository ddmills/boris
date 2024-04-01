use bevy::{ecs::system::Resource, utils::hashbrown::HashMap};

use super::{NavigationGroup, Partition, Region};

#[derive(Resource)]
pub struct NavigationGraph {
    partitions: HashMap<u32, Partition>,
    regions: HashMap<u32, Region>,
    groups: HashMap<u32, NavigationGroup>,

    cur_partition_id: u32,
    cur_region_id: u32,
    cur_group_id: u32,
}

impl Default for NavigationGraph {
    fn default() -> Self {
        Self {
            partitions: HashMap::new(),
            regions: HashMap::new(),
            groups: HashMap::new(),
            cur_partition_id: 0,
            cur_region_id: 0,
            cur_group_id: 0,
        }
    }
}

impl NavigationGraph {
    pub fn get_partition(&self, id: &u32) -> Option<&Partition> {
        self.partitions.get(id)
    }

    pub fn get_partition_mut(&mut self, id: &u32) -> Option<&mut Partition> {
        self.partitions.get_mut(id)
    }

    pub fn get_region(&self, id: &u32) -> Option<&Region> {
        self.regions.get(id)
    }

    pub fn get_region_mut(&mut self, id: &u32) -> Option<&mut Region> {
        self.regions.get_mut(id)
    }

    pub fn get_group(&self, id: &u32) -> Option<&NavigationGroup> {
        self.groups.get(id)
    }

    pub fn get_group_mut(&mut self, id: &u32) -> Option<&mut NavigationGroup> {
        self.groups.get_mut(id)
    }

    pub fn get_partition_flags() {}
}
