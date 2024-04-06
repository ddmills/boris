use bevy::utils::hashbrown::HashSet;

use super::NavigationFlags;

pub struct NavigationGroup {
    pub id: u32,
    pub flags: NavigationFlags,
    pub region_ids: HashSet<u32>,
}

impl NavigationGroup {
    pub fn new(id: u32, flags: NavigationFlags) -> Self {
        Self {
            id,
            flags,
            region_ids: HashSet::new(),
        }
    }
}
