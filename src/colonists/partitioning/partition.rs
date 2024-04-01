use bevy::{ecs::entity::Entity, utils::hashbrown::HashSet};

use super::{NavigationFlags, PartitionExtents};

pub struct Partition {
    pub id: u32,
    pub region_id: u32,
    pub neighbor_ids: HashSet<u32>,
    pub chunk_idx: u32,
    pub blocks: HashSet<u32>,
    pub flags: NavigationFlags,
    pub extents: PartitionExtents,
    pub items: HashSet<Entity>,
}

impl Partition {
    pub fn new(id: u32, region_id: u32, chunk_idx: u32) -> Self {
        Self {
            id,
            region_id,
            neighbor_ids: HashSet::new(),
            chunk_idx,
            blocks: HashSet::new(),
            flags: NavigationFlags::NONE,
            extents: PartitionExtents::default(),
            items: HashSet::new(),
        }
    }
}
