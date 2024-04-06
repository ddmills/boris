use bevy::{ecs::entity::Entity, utils::hashbrown::HashSet};

use super::{NavigationFlags, PartitionExtents};

pub struct Partition {
    pub id: u32,
    pub region_id: u32,
    pub chunk_idx: u32,
    pub flags: NavigationFlags,
    pub is_computed: bool,
    pub neighbor_ids: HashSet<u32>,
    pub blocks: HashSet<u32>,
    pub extents: PartitionExtents,
    pub items: HashSet<Entity>,
}

impl Partition {
    pub fn new(id: u32, region_id: u32, chunk_idx: u32, flags: NavigationFlags) -> Self {
        Self {
            id,
            region_id,
            chunk_idx,
            flags,
            is_computed: false,
            neighbor_ids: HashSet::new(),
            blocks: HashSet::new(),
            extents: PartitionExtents::default(),
            items: HashSet::new(),
        }
    }

    pub fn assign_block(&mut self, block_idx: u32, block_pos: [u32; 3]) {
        self.blocks.insert(block_idx);
        self.extents.extend(block_pos);
    }
}
