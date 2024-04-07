use std::hash::Hash;

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

impl PartialEq for NavigationGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NavigationGroup {}

impl Hash for NavigationGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
