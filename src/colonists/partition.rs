use bevy::{
    ecs::{
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        system::{Query, Res, ResMut, Resource},
    },
    gizmos::gizmos::Gizmos,
    math::Vec3,
    render::color::Color,
    transform::components::Transform,
    utils::{hashbrown::HashSet, HashMap},
};
use ndshape::AbstractShape;

use crate::{
    common::{flood_fill, flood_fill_i32, max_3, Distance},
    Block, Terrain,
};

use super::{Item, NavigationFlags};

#[derive(Default)]
pub struct PartitionExtents {
    is_init: bool,
    pub min_x: u32,
    pub min_y: u32,
    pub min_z: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub max_z: u32,
    pub traversal_distance: f32,
}

impl PartitionExtents {
    pub fn center(&self) -> [u32; 3] {
        [
            self.min_x + (self.max_x - self.min_x) / 2,
            self.min_y + (self.max_y - self.min_y) / 2,
            self.min_z + (self.max_z - self.min_z) / 2,
        ]
    }

    pub fn extend(&mut self, pos: [u32; 3]) {
        if !self.is_init {
            self.min_x = pos[0];
            self.min_y = pos[1];
            self.min_z = pos[2];
            self.max_x = pos[0];
            self.max_y = pos[1];
            self.max_z = pos[2];
            self.is_init = true;
            return;
        };

        self.min_x = pos[0].min(self.min_x);
        self.min_y = pos[1].min(self.min_y);
        self.min_z = pos[2].min(self.min_z);
        self.max_x = pos[0].max(self.max_x);
        self.max_y = pos[1].max(self.max_y);
        self.max_z = pos[2].max(self.max_z);
    }

    pub fn distance_to_edge(&self, x: i32, _y: i32, z: i32) -> f32 {
        // TODO: this only works in 2D space
        let dx = max_3(self.min_x as i32 - x, 0, x - self.max_x as i32).abs();
        let dz = max_3(self.min_z as i32 - z, 0, z - self.max_z as i32).abs();
        // let dz = max_3(self.min_z as i32 - z, 0, z - self.max_z as i32).abs();

        (dx + dz) as f32 - (0.59 * dx.min(dz) as f32)
    }

    pub fn update_traversal_distance(&mut self) {
        self.traversal_distance = Distance::diagonal(
            [self.min_x as i32, self.min_y as i32, self.min_z as i32],
            [self.max_x as i32, self.max_y as i32, self.max_z as i32],
        );
    }
}

pub struct Region {
    pub id: u16,
    pub flags: NavigationFlags,
    pub partition_ids: HashSet<u16>,
    pub neighbor_ids: HashSet<u16>,
}

pub struct Partition {
    pub id: u16,
    pub neighbor_ids: HashSet<u16>,
    pub is_computed: bool,
    pub chunk_idx: u32,
    pub blocks: Vec<u32>,
    pub flags: NavigationFlags,
    pub extents: PartitionExtents,
    pub items: Vec<Entity>,
    pub region_id: u16,
}

impl Partition {
    pub const NONE: u16 = 0;

    pub fn assign_block(&mut self, block_idx: u32, block_pos: [u32; 3]) {
        self.blocks.push(block_idx);
        self.extents.extend(block_pos);
    }

    pub fn remove_neighbor(&mut self, neighbor_id: &u16) {
        self.neighbor_ids.remove(neighbor_id);
    }
}

#[derive(Resource, Default)]
pub struct PartitionDebug {
    pub id: u16,
    pub show: bool,
}

pub fn partition_debug(
    terrain: Res<Terrain>,
    graph: Res<PartitionGraph>,
    mut debug: ResMut<PartitionDebug>,
    mut gizmos: Gizmos,
) {
    if !debug.show || debug.id == Partition::NONE {
        return;
    }

    let Some(partition) = graph.partitions.get(&debug.id) else {
        println!("Partition ID does not exist!");
        debug.id = Partition::NONE;
        debug.show = false;
        return;
    };

    debug_partition(
        partition,
        &terrain,
        &mut gizmos,
        Color::OLIVE,
        Color::ORANGE,
    );

    let region = graph.get_region(partition.region_id).unwrap();

    for partition_id in region.partition_ids.iter() {
        if *partition_id == debug.id {
            continue;
        }

        let part = graph.get_partition(*partition_id).unwrap();

        debug_partition(part, &terrain, &mut gizmos, Color::GRAY, Color::GRAY);
    }

    for neighbor_reg in region.neighbor_ids.iter() {
        let neighbor = graph.get_region(*neighbor_reg).unwrap();
        for partition_id in neighbor.partition_ids.iter() {
            let part = graph.get_partition(*partition_id).unwrap();
            debug_partition(part, &terrain, &mut gizmos, Color::BLUE, Color::BLUE);
        }
    }
}

fn debug_partition(
    partition: &Partition,
    terrain: &Res<Terrain>,
    gizmos: &mut Gizmos,
    color: Color,
    color_extents: Color,
) {
    for block_idx in partition.blocks.iter() {
        let [x, y, z] = terrain.get_block_world_pos(partition.chunk_idx, *block_idx);
        let pos = Vec3::new(x as f32, y as f32 + 0.02, z as f32);

        gizmos.line(pos, pos + Vec3::new(1., 0., 0.), color);
        gizmos.line(pos, pos + Vec3::new(0., 0., 1.), color);

        gizmos.line(pos, pos + Vec3::new(1., 0., 0.), color);
        gizmos.line(pos, pos + Vec3::new(0., 0., 1.), color);

        gizmos.line(
            pos + Vec3::new(1., 0., 1.),
            pos + Vec3::new(1., 0., 0.),
            color,
        );
        gizmos.line(
            pos + Vec3::new(1., 0., 1.),
            pos + Vec3::new(0., 0., 1.),
            color,
        );

        let extents = &partition.extents;

        let ex_min = Vec3::new(
            extents.min_x as f32,
            extents.min_y as f32,
            extents.min_z as f32,
        );
        let ex_max = Vec3::new(
            extents.max_x as f32 + 1.,
            extents.max_y as f32 + 1.,
            extents.max_z as f32 + 1.,
        );

        gizmos.line(
            ex_min,
            Vec3::new(ex_max.x, ex_min.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            ex_min,
            Vec3::new(ex_min.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            ex_min,
            Vec3::new(ex_min.x, ex_min.y, ex_max.z),
            color_extents,
        );

        gizmos.line(
            ex_max,
            Vec3::new(ex_min.x, ex_max.y, ex_max.z),
            color_extents,
        );
        gizmos.line(
            ex_max,
            Vec3::new(ex_max.x, ex_min.y, ex_max.z),
            color_extents,
        );
        gizmos.line(
            ex_max,
            Vec3::new(ex_max.x, ex_max.y, ex_min.z),
            color_extents,
        );

        gizmos.line(
            Vec3::new(ex_max.x, ex_min.y, ex_min.z),
            Vec3::new(ex_max.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            Vec3::new(ex_min.x, ex_max.y, ex_max.z),
            Vec3::new(ex_min.x, ex_min.y, ex_max.z),
            color_extents,
        );

        gizmos.line(
            Vec3::new(ex_min.x, ex_max.y, ex_min.z),
            Vec3::new(ex_max.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            Vec3::new(ex_min.x, ex_min.y, ex_max.z),
            Vec3::new(ex_max.x, ex_min.y, ex_max.z),
            color_extents,
        );

        gizmos.line(
            Vec3::new(ex_min.x, ex_max.y, ex_max.z),
            Vec3::new(ex_min.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            Vec3::new(ex_max.x, ex_min.y, ex_min.z),
            Vec3::new(ex_max.x, ex_min.y, ex_max.z),
            color_extents,
        );
    }
}

#[derive(Resource, Default)]
pub struct PartitionGraph {
    pub regions: HashMap<u16, Region>,
    pub partitions: HashMap<u16, Partition>,
    cur_region_id: u16,
    cur_partition_id: u16,
    // pub returned_partition_ids: Vec<u16>,
    // pub returned_region_ids: Vec<u16>,
}

impl PartitionGraph {
    pub fn create_partition(&mut self, chunk_idx: u32, flags: NavigationFlags) -> u16 {
        // let id = self.returned_partition_ids.pop().unwrap_or_else(|| {
        //     self.cur_partition_id += 1;
        //     self.cur_partition_id
        // });
        self.cur_partition_id += 1;
        let id = self.cur_partition_id;

        let mut p_ids = HashSet::new();
        p_ids.insert(id);
        let region_id = self.create_region(flags, p_ids);

        self.partitions.insert(
            id,
            Partition {
                id,
                chunk_idx,
                neighbor_ids: HashSet::new(),
                is_computed: false,
                blocks: vec![],
                flags,
                extents: PartitionExtents::default(),
                items: vec![],
                region_id,
            },
        );

        id
    }

    pub fn create_region(&mut self, flags: NavigationFlags, partition_ids: HashSet<u16>) -> u16 {
        // let id = self.returned_region_ids.pop().unwrap_or_else(|| {
        //     self.cur_region_id += 1;
        //     self.cur_region_id
        // });
        self.cur_region_id += 1;
        let id = self.cur_region_id;

        self.regions.insert(
            id,
            Region {
                id,
                flags,
                partition_ids,
                neighbor_ids: HashSet::new(),
            },
        );

        id
    }

    pub fn get_partition_mut(&mut self, partition_id: u16) -> Option<&mut Partition> {
        self.partitions.get_mut(&partition_id)
    }

    pub fn get_partition(&self, partition_id: u16) -> Option<&Partition> {
        self.partitions.get(&partition_id)
    }

    pub fn get_region_mut(&mut self, region_id: u16) -> Option<&mut Region> {
        self.regions.get_mut(&region_id)
    }

    pub fn get_region(&self, region_id: u16) -> Option<&Region> {
        self.regions.get(&region_id)
    }

    pub fn get_region_for_partition(&self, partition_id: u16) -> Option<&Region> {
        let partition = self.get_partition(partition_id)?;

        self.get_region(partition.region_id)
    }

    pub fn delete_region(&mut self, region_id: u16) -> Option<Region> {
        let region = self.regions.remove(&region_id);

        if let Some(r) = region {
            for neighbor_id in r.neighbor_ids.iter() {
                self.get_region_mut(*neighbor_id)
                    .unwrap()
                    .neighbor_ids
                    .remove(&region_id);
            }

            return Some(r);
        };

        None
    }

    fn get_partition_ids_for_chunk(&self, chunk_idx: u32) -> Vec<u16> {
        self.partitions
            .iter()
            .filter_map(|(id, p)| {
                if p.chunk_idx == chunk_idx {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn delete_partitions_for_chunk(&mut self, chunk_idx: u32) -> Vec<Partition> {
        let partition_ids = self.get_partition_ids_for_chunk(chunk_idx);
        let mut cleanups: Vec<[u16; 2]> = vec![];
        let mut regions_to_flood = HashSet::new();

        for partition_id in partition_ids.iter() {
            let partition = self.get_partition(*partition_id).unwrap();

            for neighbor_id in partition.neighbor_ids.iter() {
                cleanups.push([*neighbor_id, *partition_id]);
            }
        }

        let mut partitions = vec![];
        let mut removed = vec![];

        for [neighbor_id, remove_id] in cleanups {
            self.get_partition_mut(neighbor_id)
                .unwrap()
                .remove_neighbor(&remove_id);
            removed.push(remove_id);
        }

        println!(
            "deleting {} partition ids for chunk {}",
            partition_ids.len(),
            chunk_idx
        );

        for id in partition_ids.iter() {
            let p = self.partitions.remove(id).unwrap();

            if let Some(region) = self.get_region_mut(p.region_id) {
                region.partition_ids.remove(id);
                regions_to_flood.insert(region.id);
            }

            partitions.push(p);
        }

        println!("flooding {} regions", regions_to_flood.len());
        for region_to_flood in regions_to_flood.iter() {
            self.flood_region(*region_to_flood);
        }

        partitions
    }

    /// flood the partitions in this region, deleting the region if
    /// it has none, or creating new regions for any unique islands.
    fn flood_region(&mut self, region_id: u16) {
        let region = self.get_region(region_id).unwrap();
        println!("flooding region {}, {}", region_id, region.flags);
        // delete if empty
        if region.partition_ids.is_empty() {
            println!("region is empty, deleting it.");
            self.delete_region(region_id).unwrap();
            return;
        }

        let mut open_list = region.partition_ids.iter().collect::<Vec<_>>();
        let mut closed_list = vec![];
        let mut groups = vec![];

        println!("flooding filling through {} partition ids", open_list.len());
        while let Some(seed) = open_list.pop() {
            let mut group = HashSet::new();
            let mut neighbors = HashSet::new();

            flood_fill(
                *seed,
                |id| {
                    let neighbor_partition = self.get_partition(id).unwrap();

                    if neighbor_partition.flags != region.flags {
                        closed_list.push(id);
                        neighbors.insert(neighbor_partition.region_id);
                        return false;
                    }

                    if closed_list.contains(&id) {
                        return false;
                    }

                    open_list.retain(|i| **i != id);
                    closed_list.push(id);
                    group.insert(id);
                    true
                },
                |id| {
                    self.get_partition(id)
                        .unwrap()
                        .neighbor_ids
                        .iter()
                        .copied()
                        .collect()
                },
            );
            groups.push((group, neighbors));
        }

        self.split_region(region_id, region.flags, groups);
    }

    fn split_region(
        &mut self,
        region_id: u16,
        flags: NavigationFlags,
        groups: Vec<(HashSet<u16>, HashSet<u16>)>,
    ) {
        println!("region {} split into {} groups", region_id, groups.len());
        let region = self.get_region(region_id).unwrap();

        for neighbor_id in region.neighbor_ids.clone().iter() {
            self.remove_region_neighbors(region_id, *neighbor_id);
        }

        for (idx, (group, neighbors_ids)) in groups.iter().enumerate() {
            if idx == 0 {
                for neighbor_id in neighbors_ids.iter() {
                    self.set_region_neighbors(region_id, *neighbor_id);
                }

                for partition_id in group.iter() {
                    self.get_partition_mut(*partition_id).unwrap().region_id = region_id;
                }

                self.get_region_mut(region_id).unwrap().partition_ids = group.clone();
            } else {
                let new_region_id = self.create_region(flags, group.clone());

                for neighbor_id in neighbors_ids.iter() {
                    self.set_region_neighbors(new_region_id, *neighbor_id);
                }

                for partition_id in group.iter() {
                    self.get_partition_mut(*partition_id).unwrap().region_id = new_region_id;
                }
            }
        }

        println!("done splitting region {}", region_id);
    }

    pub fn is_partition_computed(&self, id: u16) -> bool {
        if let Some(p) = self.get_partition(id) {
            return p.is_computed;
        }
        false
    }

    pub fn get_partition_flags(&self, id: u16) -> NavigationFlags {
        if let Some(p) = self.get_partition(id) {
            return p.flags;
        }
        NavigationFlags::NONE
    }

    pub fn set_partition_computed(&mut self, id: u16, value: bool) {
        if let Some(p) = self.get_partition_mut(id) {
            p.is_computed = value;

            if value {
                p.extents.update_traversal_distance();
            }
        }
    }

    pub fn assign_block(&mut self, partition_id: u16, block_idx: u32, block_pos: [u32; 3]) {
        if let Some(p) = self.get_partition_mut(partition_id) {
            p.assign_block(block_idx, block_pos);
        }
    }

    pub fn set_partition_neighbors(&mut self, a_id: u16, b_id: u16) {
        let a = self.partitions.get_mut(&a_id).unwrap();
        a.neighbor_ids.insert(b_id);

        let b = self.partitions.get_mut(&b_id).unwrap();
        b.neighbor_ids.insert(a_id);
    }

    pub fn set_region_neighbors(&mut self, a_id: u16, b_id: u16) {
        let a = self.regions.get_mut(&a_id).unwrap();
        a.neighbor_ids.insert(b_id);

        let b = self.regions.get_mut(&b_id).unwrap();
        b.neighbor_ids.insert(a_id);
    }

    pub fn remove_region_neighbors(&mut self, a_id: u16, b_id: u16) {
        let a = self.regions.get_mut(&a_id).unwrap();
        a.neighbor_ids.remove(&b_id);

        let b = self.regions.get_mut(&b_id).unwrap();
        b.neighbor_ids.remove(&a_id);
    }
}

#[derive(Event)]
pub struct PartitionEvent {
    pub chunk_idx: u32,
    pub refresh: bool,
}

pub fn busy_work(
    graph: &mut ResMut<PartitionGraph>,
    a_region_id: u16,
    b_region_id: u16,
) -> (u16, u16, Vec<u16>) {
    let [a_region, b_region] = graph
        .regions
        .get_many_mut([&a_region_id, &b_region_id])
        .unwrap();

    let (smaller_region, bigger_region) = {
        if a_region.partition_ids.len() > b_region.partition_ids.len() {
            (b_region, a_region)
        } else {
            (a_region, b_region)
        }
    };

    let partition_ids = smaller_region
        .partition_ids
        .iter()
        .map(|partition_id| {
            bigger_region.partition_ids.insert(*partition_id);
            *partition_id
        })
        .collect::<Vec<_>>();

    println!(
        "merging {} partitions from region {} into {}",
        partition_ids.len(),
        smaller_region.id,
        bigger_region.id,
    );

    (bigger_region.id, smaller_region.id, partition_ids)
}

pub fn merge_regions(
    graph: &mut ResMut<PartitionGraph>,
    a_region_id: u16,
    b_region_id: u16,
) -> u16 {
    if a_region_id == b_region_id {
        return a_region_id;
    }

    let (big_region_id, small_region_id, partition_ids) =
        busy_work(graph, a_region_id, b_region_id);

    graph.delete_region(small_region_id);

    for p_id in partition_ids {
        graph.partitions.get_mut(&p_id).unwrap().region_id = big_region_id;
    }

    big_region_id
}

pub fn merge_partitions(
    graph: &mut ResMut<PartitionGraph>,
    terrain: &mut ResMut<Terrain>,
    a_id: u16,
    b_id: u16,
) -> u16 {
    // merge B into A
    let b_partition = graph.get_partition(b_id).unwrap();
    let block_idxs: Vec<u32> = b_partition.blocks.to_vec();
    let neighbors_ids: Vec<u16> = b_partition.neighbor_ids.iter().copied().collect();
    let b_computed = b_partition.is_computed;
    let a_partition = graph.get_partition(a_id).unwrap();

    merge_regions(graph, a_partition.region_id, b_partition.region_id);

    let a_partition_mut = graph.get_partition_mut(a_id).unwrap();
    a_partition_mut.is_computed = a_partition_mut.is_computed && b_computed;

    for block_idx in block_idxs {
        let block_pos = terrain.get_block_world_pos(a_partition_mut.chunk_idx, block_idx);
        a_partition_mut.assign_block(block_idx, block_pos);
        terrain.set_partition_id(a_partition_mut.chunk_idx, block_idx, a_id);
    }

    a_partition_mut.extents.update_traversal_distance();

    for neighor_id in neighbors_ids {
        if neighor_id == a_id {
            continue;
        }

        if let Some(neighbor) = graph.get_partition_mut(neighor_id) {
            neighbor.remove_neighbor(&b_id);
            graph.set_partition_neighbors(a_id, neighor_id);
            // todo: fix regions here?
        }
    }
    graph.partitions.remove(&b_id);

    a_id
}

pub fn partition(
    mut terrain: ResMut<Terrain>,
    mut graph: ResMut<PartitionGraph>,
    mut partition_ev: EventReader<PartitionEvent>,
    q_items: Query<&Transform, With<Item>>,
) {
    for ev in partition_ev.read() {
        let chunk_idx = ev.chunk_idx;
        let mut items: Vec<Entity> = vec![];

        if ev.refresh {
            let cleanups = graph.delete_partitions_for_chunk(chunk_idx);
            println!("done deleting partitions for chunk {}", chunk_idx);

            for mut p in cleanups {
                for b in p.blocks.iter() {
                    terrain.set_partition_id(p.chunk_idx, *b, Partition::NONE);
                    items.append(&mut p.items);
                }
            }

            println!("done cleaning up partitions for chunk {}", chunk_idx);
        }

        println!("partitioning chunk {}", chunk_idx);
        for block_idx in 0..terrain.chunk_shape.size() {
            let [x, y, z] = terrain.get_block_world_pos(chunk_idx, block_idx);

            let seed_flags = get_block_flags(&terrain, x as i32, y as i32, z as i32);

            // don't partition empty space
            if seed_flags == NavigationFlags::NONE {
                continue;
            }

            let mut partition_id = terrain.get_partition_id(chunk_idx, block_idx);

            if partition_id == Partition::NONE {
                // if we are here, that means the block is navigable,
                // and it is not assigned to a partition yet. We must
                // create a new partition and assign it
                partition_id = graph.create_partition(chunk_idx, seed_flags);
            }

            // if the block is already in a computed partition, it has
            // already been claimed and we can skip it.
            if graph.is_partition_computed(partition_id) {
                continue;
            }

            let mut region_id = graph.get_region_for_partition(partition_id).unwrap().id;

            // next, flood fill from the block, looking for other
            // navigable blocks to add the current partition
            flood_fill_i32([x as i32, y as i32, z as i32], |[nx, ny, nz]| {
                if terrain.is_oob(nx, ny, nz) {
                    return false;
                }

                let [nchunk_idx, nblock_idx] =
                    terrain.get_block_indexes(nx as u32, ny as u32, nz as u32);

                let mut npartition_id = terrain.get_partition_id(nchunk_idx, nblock_idx);

                // if this block is already assigned to our partition,
                // it means we have already visited it, and we should
                // not flood from it again.
                if npartition_id == partition_id && block_idx != nblock_idx {
                    return false;
                }

                let nblock_flags = get_block_flags(&terrain, nx, ny, nz);

                // this block is not navigable and won't fit in any partition
                if nblock_flags == NavigationFlags::NONE {
                    return false;
                }

                let npartition_flags = graph.get_partition_flags(partition_id);

                let flag_diff = nblock_flags != npartition_flags;
                let chunk_diff = nchunk_idx != chunk_idx;

                // if we are in a different chunk, or if the flags do not match,
                // we must determine which partition this block belongs to, and
                // assign it as a neighbor
                if flag_diff || chunk_diff {
                    if npartition_id != Partition::NONE {
                        // a partition already exists, add it as a neighbor
                        graph.set_partition_neighbors(partition_id, npartition_id);
                    } else {
                        // a partition does not exist, create a new one, assign the
                        // block to it, and add it as a neighbor
                        npartition_id = graph.create_partition(nchunk_idx, nblock_flags);
                        graph.set_partition_neighbors(partition_id, npartition_id);

                        terrain.set_partition_id(nchunk_idx, nblock_idx, npartition_id);
                        graph.assign_block(
                            npartition_id,
                            nblock_idx,
                            [nx as u32, ny as u32, nz as u32],
                        );
                    }

                    let nregion_id = graph.get_region_for_partition(npartition_id).unwrap().id;

                    if !flag_diff {
                        region_id = merge_regions(&mut graph, region_id, nregion_id);
                    } else {
                        // set regions as neighbors
                        graph.set_region_neighbors(region_id, nregion_id);
                    }

                    // we are done flooding here, we will process this neighbor
                    // partition later.
                    return false;
                }

                if npartition_id != Partition::NONE && npartition_id != partition_id {
                    println!(
                        "these should be merged? {}, {}",
                        npartition_id, partition_id
                    );
                    // merge_partitions(&mut graph, &mut terrain, partition_id, npartition_id);
                }

                // this block is navigable, it is in the same chunk, and it has
                // matching flags, so we can assign it to the partition and
                // continue flooding.
                terrain.set_partition_id(nchunk_idx, nblock_idx, partition_id);
                graph.assign_block(partition_id, nblock_idx, [nx as u32, ny as u32, nz as u32]);

                true
            });

            // we have flooded the partition, we mark it as computed
            graph.set_partition_computed(partition_id, true);
        }

        for item in items {
            let Ok(transform) = q_items.get(item) else {
                println!("Item was supposed to be in this chunk.");
                continue;
            };

            let x = transform.translation.x as u32;
            let y = transform.translation.y as u32;
            let z = transform.translation.z as u32;

            let item_partition_id = terrain.get_partition_id_u32(x, y, z);

            if item_partition_id == Partition::NONE {
                println!("Item is not in a valid partition! Teleport it?");
                continue;
            }

            let Some(partition) = graph.get_partition_mut(item_partition_id) else {
                println!("Missing partition?");
                continue;
            };

            println!("updated item to be in new partition!");
            partition.items.push(item);
        }
    }
}

pub fn get_block_flags(terrain: &Terrain, x: i32, y: i32, z: i32) -> NavigationFlags {
    let block = terrain.get_block_i32(x, y, z);

    let mut flags = NavigationFlags::NONE;

    if block == Block::LADDER {
        return NavigationFlags::LADDER;
    }

    if !block.is_empty() {
        return NavigationFlags::NONE;
    }

    let nblock_below = terrain.get_block_i32(x, y - 1, z);

    if nblock_below == Block::LADDER {
        return NavigationFlags::LADDER;
    }

    if nblock_below.is_walkable() {
        flags |= NavigationFlags::SOLID_GROUND;

        let nblock_above = terrain.get_block_i32(x, y + 1, z);

        if nblock_above.is_empty() {
            flags |= NavigationFlags::TALL;
        }
    }

    flags
}

pub fn partition_setup(terrain: Res<Terrain>, mut partition_chunk_ev: EventWriter<PartitionEvent>) {
    println!("partitioning world..");

    // for chunk_idx in 0..terrain.chunk_count {
    //     partition_chunk_ev.send(PartitionEvent {
    //         chunk_idx,
    //         refresh: false,
    //     });
    // }
    println!("..done partitioning world");
}
