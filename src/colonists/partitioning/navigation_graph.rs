use bevy::{ecs::system::Resource, utils::hashbrown::HashMap};

use crate::Terrain;

use super::{NavigationFlags, NavigationGroup, Partition, Region};

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
    pub fn create_partition(
        &mut self,
        region_id: u32,
        chunk_idx: u32,
        flags: NavigationFlags,
    ) -> u32 {
        self.cur_partition_id += 1;
        let id = self.cur_partition_id;
        let partition = Partition::new(id, region_id, chunk_idx, flags);
        self.partitions.insert(id, partition);
        let region = self.get_region_mut(&region_id).unwrap();
        region.partition_ids.insert(id);
        id
    }

    pub fn create_region(&mut self, flags: NavigationFlags) -> u32 {
        self.cur_region_id += 1;
        let id = self.cur_region_id;
        let region = Region::new(id, flags);
        self.regions.insert(id, region);
        id
    }

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

    pub fn get_region_id_for_partition(&self, partition_id: &u32) -> Option<&u32> {
        let partition = self.get_partition(partition_id)?;

        Some(&partition.region_id)
    }

    pub fn set_partition_neighbors(&mut self, a_id: &u32, b_id: &u32) {
        let a = self.get_partition_mut(a_id).unwrap();
        a.neighbor_ids.insert(*b_id);

        let b = self.get_partition_mut(b_id).unwrap();
        b.neighbor_ids.insert(*a_id);
    }

    pub fn set_region_neighbors(&mut self, a_id: &u32, b_id: &u32) {
        let a = self.get_region_mut(a_id).unwrap();
        a.neighbor_ids.insert(*b_id);

        let b = self.get_region_mut(b_id).unwrap();
        b.neighbor_ids.insert(*a_id);
    }

    pub fn assign_block(
        &mut self,
        partition_id: &u32,
        block_idx: u32,
        block_pos: [u32; 3],
        terrain: &mut Terrain,
    ) {
        let partition = self.get_partition_mut(partition_id).unwrap();
        partition.assign_block(block_idx, block_pos);
        terrain.set_partition_id(partition.chunk_idx, block_idx, *partition_id);
    }

    pub fn delete_region(&mut self, region_id: &u32) {
        self.regions.remove(region_id);
    }

    pub fn merge_partitions(&mut self, a_id: &u32, b_id: &u32, terrain: &mut Terrain) -> u32 {
        let b_partition = self.partitions.remove(b_id).unwrap();
        let b_region_id = b_partition.region_id;
        let b_neighbor_ids = b_partition.neighbor_ids;
        let block_idxs = b_partition.blocks.clone();
        let a_partition = self.get_partition_mut(a_id).unwrap();

        a_partition.is_computed = a_partition.is_computed && b_partition.is_computed;

        for block_idx in block_idxs {
            let block_pos = terrain.get_block_world_pos(a_partition.chunk_idx, block_idx);
            a_partition.assign_block(block_idx, block_pos);
            terrain.set_partition_id(a_partition.chunk_idx, block_idx, *a_id);
        }

        if a_partition.is_computed {
            a_partition.extents.update_traversal_distance();
        }

        for neighor_id in b_neighbor_ids.iter() {
            if neighor_id == a_id {
                continue;
            }

            if let Some(neighbor) = self.get_partition_mut(neighor_id) {
                neighbor.neighbor_ids.remove(b_id);
                self.set_partition_neighbors(a_id, neighor_id);
            }
        }

        let b_region = self.get_region_mut(&b_region_id).unwrap();
        b_region.partition_ids.remove(b_id);

        if b_region.partition_ids.is_empty() {
            println!("deleting region {}", b_region_id);
            self.delete_region(&b_region_id);
        }

        *a_id
    }

    /// merge the two given regions into one, and returns the new region id.
    /// The smaller region (in terms of partition_ids) will be merged into
    /// the bigger region. This also updates the partition ids within the
    /// regions.
    pub fn merge_regions(&mut self, a_id: &u32, b_id: &u32) -> u32 {
        if a_id == b_id {
            return *a_id;
        }

        let (small_id, big_id) = self.compare_regions(a_id, b_id);

        let [small_region, big_region] = self.regions.get_many_mut([&small_id, &big_id]).unwrap();

        for partition_id in small_region.partition_ids.iter() {
            self.partitions.get_mut(partition_id).unwrap().region_id = big_id;
            big_region.partition_ids.insert(*partition_id);
        }

        *a_id
    }

    /// Compares the number of partitions in the given regions, and returns (smaller_id, bigger_id)
    fn compare_regions(&mut self, a_id: &u32, b_id: &u32) -> (u32, u32) {
        let [a_region, b_region] = self.regions.get_many_mut([a_id, b_id]).unwrap();

        let (smaller_region, bigger_region) = {
            if a_region.partition_ids.len() > b_region.partition_ids.len() {
                (b_region, a_region)
            } else {
                (a_region, b_region)
            }
        };

        (smaller_region.id, bigger_region.id)
    }
}
