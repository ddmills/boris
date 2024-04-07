use bevy::{
    ecs::system::Resource,
    utils::hashbrown::{HashMap, HashSet},
};

use crate::{common::flood_fill, Terrain};

use super::{partition, region, NavigationFlags, NavigationGroup, Partition, Region};

#[derive(Resource)]
pub struct NavigationGraph {
    partitions: HashMap<u32, Partition>,
    regions: HashMap<u32, Region>,
    groups: HashMap<u32, NavigationGroup>,

    group_types: HashSet<NavigationFlags>,

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
            group_types: HashSet::from([NavigationFlags::COLONIST, NavigationFlags::CAT]),
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
        let partition_id = self.cur_partition_id;
        let partition = Partition::new(partition_id, region_id, chunk_idx, flags);
        self.partitions.insert(partition_id, partition);
        let region = self.get_region_mut(&region_id).unwrap();
        region.partition_ids.insert(partition_id);
        partition_id
    }

    pub fn create_region(&mut self, flags: NavigationFlags) -> u32 {
        self.cur_region_id += 1;
        let region_id = self.cur_region_id;
        let group_ids = self.create_navigation_groups_for_region(flags, &region_id);

        let mut region = Region::new(region_id, flags);
        region.group_ids = group_ids;
        self.regions.insert(region_id, region);

        region_id
    }

    fn create_navigation_groups_for_region(
        &mut self,
        flags: NavigationFlags,
        region_id: &u32,
    ) -> HashSet<u32> {
        let mut group_ids = HashSet::new();

        for group_type in self.group_types.iter() {
            if !flags.intersects(*group_type) {
                continue;
            }

            self.cur_group_id += 1;
            let group_id = self.cur_group_id;

            let mut group = NavigationGroup::new(group_id, *group_type);
            group.region_ids.insert(*region_id);
            self.groups.insert(group_id, group);

            group_ids.insert(group_id);
        }

        group_ids
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

    pub fn get_group_ids_for_partition(&self, partition_id: &u32) -> HashSet<u32> {
        let Some(partition) = self.get_partition(partition_id) else {
            return HashSet::new();
        };

        let Some(region) = self.get_region(&partition.region_id) else {
            return HashSet::new();
        };

        region.group_ids.clone()
    }

    pub fn get_groups_for_partition(&self, partition_id: &u32) -> HashSet<&NavigationGroup> {
        self.get_group_ids_for_partition(partition_id)
            .iter()
            .filter_map(|group_id| self.get_group(group_id))
            .collect::<HashSet<_>>()
    }

    /// Set partitions A and B as neighbors. It also makes the regions neighbors
    /// if applicable, or merges regions if applicable. If the regions are
    /// merged, the new region ID will be returned.
    pub fn set_partition_neighbors(&mut self, a_id: &u32, b_id: &u32) -> Option<u32> {
        let [a_partition, b_partition] = self.partitions.get_many_mut([a_id, b_id]).unwrap();
        a_partition.neighbor_ids.insert(*b_id);
        b_partition.neighbor_ids.insert(*a_id);

        let a_region_id = a_partition.region_id;
        let b_region_id = b_partition.region_id;

        if a_region_id != b_region_id {
            if a_partition.flags == b_partition.flags {
                let region_id = self.merge_regions(&a_region_id, &b_region_id);
                return Some(region_id);
            } else {
                self.set_region_neighbors(&a_region_id, &b_region_id);
            }
        }

        None
    }

    /// Set regions A and B as neighbors. Also merge any navigation groups if
    /// applicable.
    pub fn set_region_neighbors(&mut self, a_id: &u32, b_id: &u32) {
        let [a_region, b_region] = self.regions.get_many_mut([a_id, b_id]).unwrap();

        a_region.neighbor_ids.insert(*b_id);
        b_region.neighbor_ids.insert(*a_id);

        self.merge_navigation_groups_for_regions(a_id, b_id);
    }

    pub fn merge_navigation_groups_for_regions(&mut self, a_region_id: &u32, b_region_id: &u32) {
        let a_region = self.get_region(a_region_id).unwrap();
        let b_region = self.get_region(b_region_id).unwrap();

        let a_groups = a_region
            .group_ids
            .iter()
            .map(|group_id| {
                let group = self.get_group(group_id).unwrap();
                (*group_id, group.flags)
            })
            .collect::<Vec<_>>();
        let b_groups = b_region
            .group_ids
            .iter()
            .map(|group_id| {
                let group = self.get_group(group_id).unwrap();
                (*group_id, group.flags)
            })
            .collect::<Vec<_>>();

        for (a_group_id, a_group_flags) in a_groups.iter() {
            if let Some((matching_group_id, _)) =
                b_groups.iter().find(|(b_group_id, b_group_flags)| {
                    b_group_id != a_group_id && b_group_flags == a_group_flags
                })
            {
                self.merge_groups(a_group_id, matching_group_id);
            }
        }
    }

    pub fn merge_groups(&mut self, a_group_id: &u32, b_group_id: &u32) -> u32 {
        let (small_group_id, big_group_id) = self.compare_groups(a_group_id, b_group_id);

        let [small_group, big_group] = self
            .groups
            .get_many_mut([&small_group_id, &big_group_id])
            .unwrap();

        for region_id in small_group.region_ids.iter() {
            big_group.region_ids.insert(*region_id);
            let group_ids = &mut self.regions.get_mut(region_id).unwrap().group_ids;
            group_ids.insert(big_group_id);
            group_ids.remove(&small_group_id);
        }

        self.delete_group(&small_group_id);

        big_group_id
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

    pub fn delete_group(&mut self, group_id: &u32) {
        self.groups.remove(group_id);
    }

    pub fn delete_region(&mut self, region_id: &u32) {
        let region = self.regions.remove(region_id).unwrap();

        // remove this region from neighbors
        for neighbor_id in region.neighbor_ids.iter() {
            let neighbor = self.get_region_mut(neighbor_id).unwrap();
            neighbor.neighbor_ids.remove(region_id);
        }

        // remove this region from nav groups
        for group_id in region.group_ids.iter() {
            let group = self.get_group_mut(group_id).unwrap();
            group.region_ids.remove(region_id);

            // delete the region if it's empty, otherwise flood it to check
            // if it's still contiguous
            if group.region_ids.is_empty() {
                self.delete_group(group_id);
            } else {
                // todo: flood group?
                // println!("flood group {} len {}", group_id, group.region_ids.len());
            }
        }
    }

    pub fn delete_partition(&mut self, partition_id: &u32) -> Partition {
        let partition = self.partitions.remove(partition_id).unwrap();

        // Remove this partition from neighbors
        for neighbor_id in partition.neighbor_ids.iter() {
            let neighbor = self.get_partition_mut(neighbor_id).unwrap();
            neighbor.neighbor_ids.remove(partition_id);
        }

        // Remove this partition from the region
        let region = self.get_region_mut(&partition.region_id).unwrap();
        let region_id = region.id;

        region.partition_ids.remove(partition_id);

        // TODO: do we need to flood the region, or do we need to flood the
        // nav groups for the region?
        // perhaps flood nav groups afterward?
        self.flood_region(&region_id);

        partition
    }

    /// Flood the partitions in this region, deleting the region if
    /// it has none, or creating new regions for any unique islands.
    fn flood_region(&mut self, region_id: &u32) {
        let region = self.get_region(region_id).unwrap();

        if region.partition_ids.is_empty() {
            self.delete_region(region_id);
            return;
        }

        let mut open_list = region.partition_ids.iter().collect::<Vec<_>>();
        let mut closed_list = vec![];
        let mut islands = vec![];

        while let Some(seed) = open_list.pop() {
            let mut island = HashSet::new();
            let mut neighbors = HashSet::new();

            flood_fill(
                seed,
                |id| {
                    if closed_list.contains(&id) {
                        return false;
                    }

                    let neighbor_partition = self.get_partition(id).unwrap();
                    closed_list.push(id);

                    if neighbor_partition.flags != region.flags {
                        neighbors.insert(neighbor_partition.region_id);
                        return false;
                    }

                    open_list.retain(|i| *i != id);
                    island.insert(*id);
                    true
                },
                |id| {
                    self.get_partition(id)
                        .unwrap()
                        .neighbor_ids
                        .iter()
                        .collect()
                },
            );
            islands.push((island, neighbors));
        }

        self.split_region(region_id, region.flags, islands);
    }

    fn split_region(
        &mut self,
        region_id: &u32,
        flags: NavigationFlags,
        islands: Vec<(HashSet<u32>, HashSet<u32>)>,
    ) {
        // TODO: split up the NAV GROUPs after

        let region_mut = self.get_region_mut(region_id).unwrap();
        let current_neighbors = region_mut.neighbor_ids.clone();
        region_mut.neighbor_ids.clear();

        for neighbor_id in current_neighbors.iter() {
            let neighbor = self.get_region_mut(neighbor_id).unwrap();
            neighbor.neighbor_ids.remove(region_id);
        }

        for (idx, (island, neighbors_ids)) in islands.iter().enumerate() {
            if idx == 0 {
                for neighbor_id in neighbors_ids.iter() {
                    self.set_region_neighbors(region_id, neighbor_id);
                }

                for partition_id in island.iter() {
                    self.get_partition_mut(partition_id).unwrap().region_id = *region_id;
                }

                self.get_region_mut(region_id).unwrap().partition_ids = island.clone();
            } else {
                let new_region_id = self.create_region(flags);
                let new_region = self.get_region_mut(&new_region_id).unwrap();
                new_region.partition_ids = island.clone();

                for neighbor_id in neighbors_ids.iter() {
                    self.set_region_neighbors(&new_region_id, neighbor_id);
                }

                for partition_id in island.iter() {
                    self.get_partition_mut(partition_id).unwrap().region_id = new_region_id;
                }
            }
        }
    }

    fn get_partition_ids_for_chunk(&self, chunk_idx: u32) -> Vec<u32> {
        self.partitions
            .iter()
            .filter_map(|(partition_id, partition)| {
                if partition.chunk_idx == chunk_idx {
                    Some(*partition_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn delete_partitions_for_chunk(&mut self, chunk_idx: u32) -> Vec<Partition> {
        let partition_ids = self.get_partition_ids_for_chunk(chunk_idx);

        partition_ids
            .iter()
            .map(|partition_id| self.delete_partition(partition_id))
            .collect::<Vec<_>>()
    }

    /// merge partition B into partition A. Returns the resulting partition id and region id
    pub fn merge_partitions(
        &mut self,
        a_id: &u32,
        b_id: &u32,
        terrain: &mut Terrain,
    ) -> (u32, u32) {
        let b_partition = self.partitions.remove(b_id).unwrap();
        let b_region_id = b_partition.region_id;
        let b_neighbor_ids = b_partition.neighbor_ids;
        let block_idxs = b_partition.blocks.clone();
        let a_partition = self.get_partition_mut(a_id).unwrap();
        let a_region_id = a_partition.region_id;

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
            self.delete_region(&b_region_id);
        } else if b_region_id != a_region_id {
            println!("merge regions? {} {}", a_region_id, b_region_id);
        }

        (*a_id, a_region_id)
    }

    /// merge the smaller region (in terms of partition_ids) into the bigger region.
    /// This also updates the partition ids within the regions. Returns the bigger
    /// region id
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

        self.delete_region(&small_id);

        big_id
    }

    /// Compares the number of partitions in the given groups, and returns (smaller_id, bigger_id)
    fn compare_groups(&self, a_id: &u32, b_id: &u32) -> (u32, u32) {
        let a_group = self.get_group(a_id).unwrap();
        let b_group = self.get_group(b_id).unwrap();

        let (smaller_group, bigger_group) = {
            if a_group.region_ids.len() > b_group.region_ids.len() {
                (b_group, a_group)
            } else {
                (a_group, b_group)
            }
        };

        (smaller_group.id, bigger_group.id)
    }

    /// Compares the number of partitions in the given regions, and returns (smaller_id, bigger_id)
    fn compare_regions(&self, a_id: &u32, b_id: &u32) -> (u32, u32) {
        let a_region = self.get_region(a_id).unwrap();
        let b_region = self.get_region(b_id).unwrap();

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
