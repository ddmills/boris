use bevy::utils::hashbrown::HashSet;

use super::NavigationFlags;

pub struct Region {
    pub id: u32,
    pub flags: NavigationFlags,
    pub partition_ids: HashSet<u32>,
    pub neighbor_ids: HashSet<u32>,
    pub group_ids: HashSet<u32>,
}

impl Region {
    pub fn new(id: u32, flags: NavigationFlags) -> Self {
        Self {
            id,
            flags,
            partition_ids: HashSet::new(),
            neighbor_ids: HashSet::new(),
            group_ids: HashSet::new(),
        }
    }
}
