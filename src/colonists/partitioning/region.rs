use bevy::utils::hashbrown::HashSet;

pub struct Region {
    pub id: u32,
    pub partition_ids: HashSet<u32>,
    pub neighbor_ids: HashSet<u32>,
    pub group_ids: HashSet<u32>,
}

impl Region {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            partition_ids: HashSet::new(),
            neighbor_ids: HashSet::new(),
            group_ids: HashSet::new(),
        }
    }
}
