use bevy::{
    ecs::{
        event::{Event, EventReader, EventWriter},
        system::{Res, ResMut, Resource},
    },
    gizmos::gizmos::Gizmos,
    math::Vec3,
    render::color::Color,
    utils::{HashMap, HashSet},
};
use ndshape::AbstractShape;

use crate::{
    common::{flood_fill, max_3, Distance},
    Block, Terrain,
};

use super::PartitionFlags;

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

pub struct Partition {
    id: u16,
    pub neighbors: HashSet<u16>,
    pub is_computed: bool,
    pub chunk_idx: u32,
    pub blocks: Vec<u32>,
    pub flags: PartitionFlags,
    pub extents: PartitionExtents,
}

impl Partition {
    pub const NONE: u16 = 0;

    pub fn add_block(&mut self, block_idx: u32, block_pos: [u32; 3]) {
        self.blocks.push(block_idx);
        self.extents.extend(block_pos);
    }

    pub fn remove_neighbor(&mut self, neighbor_id: u16) {
        self.neighbors.remove(&neighbor_id);
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
    debug: Res<PartitionDebug>,
    mut gizmos: Gizmos,
) {
    if !debug.show {
        return;
    }

    if let Some(partition) = graph.partitions.get(&debug.id) {
        debug_partition(
            partition,
            &terrain,
            &mut gizmos,
            Color::OLIVE,
            Color::ORANGE,
        );
    }

    for neighbor in graph.get_neighbors(debug.id) {
        debug_partition(neighbor, &terrain, &mut gizmos, Color::GRAY, Color::GRAY);
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
    pub partitions: HashMap<u16, Partition>,
    cur_id: u16,
    pub returned_ids: Vec<u16>,
}

impl PartitionGraph {
    pub fn create_partition(&mut self, chunk_idx: u32) -> u16 {
        let id = self.returned_ids.pop().unwrap_or_else(|| {
            self.cur_id += 1;
            self.cur_id
        });
        let p = Partition {
            id,
            chunk_idx,
            neighbors: HashSet::new(),
            is_computed: false,
            blocks: vec![],
            flags: PartitionFlags::NONE,
            extents: PartitionExtents::default(),
        };

        self.partitions.insert(p.id, p);

        id
    }

    pub fn get_center(&self, partition_id: u16) -> Option<[u32; 3]> {
        if let Some(p) = self.partitions.get(&partition_id) {
            return Some(p.extents.center());
        }

        None
    }

    pub fn get_partition_mut(&mut self, partition_id: u16) -> Option<&mut Partition> {
        self.partitions.get_mut(&partition_id)
    }

    pub fn get_partition(&self, partition_id: u16) -> Option<&Partition> {
        self.partitions.get(&partition_id)
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

    pub fn get_neighbors(&self, partition_id: u16) -> Vec<&Partition> {
        let Some(partition) = self.get_partition(partition_id) else {
            return vec![];
        };

        partition
            .neighbors
            .iter()
            .map(|n| self.get_partition(*n).unwrap())
            .collect()
    }

    pub fn delete_partitions_for_chunk(&mut self, chunk_idx: u32) -> Vec<Partition> {
        let partition_ids = self.get_partition_ids_for_chunk(chunk_idx);
        let mut cleanups: Vec<[u16; 2]> = vec![];

        for partition_id in partition_ids.clone() {
            let partition = self.get_partition(partition_id).unwrap();

            for neighbor_id in partition.neighbors.iter() {
                cleanups.push([*neighbor_id, partition_id]);
            }
        }

        let mut partitions = vec![];
        let mut removed = vec![];

        for [neighbor_id, remove_id] in cleanups {
            self.get_partition_mut(neighbor_id)
                .unwrap()
                .remove_neighbor(remove_id);
            removed.push(remove_id);
        }

        for id in partition_ids {
            partitions.push(self.partitions.remove(&id).unwrap());
            self.returned_ids.push(id);
        }

        partitions
    }

    pub fn is_partition_computed(&self, id: u16) -> bool {
        if let Some(p) = self.get_partition(id) {
            return p.is_computed;
        }
        false
    }

    pub fn get_flags(&self, id: u16) -> PartitionFlags {
        if let Some(p) = self.get_partition(id) {
            return p.flags;
        }
        PartitionFlags::NONE
    }

    pub fn set_flags(&mut self, id: u16, flags: PartitionFlags) {
        if let Some(p) = self.get_partition_mut(id) {
            p.flags = flags;
        }
    }

    pub fn set_partition_computed(&mut self, id: u16, value: bool) {
        if let Some(p) = self.get_partition_mut(id) {
            p.is_computed = value;

            if value {
                p.extents.update_traversal_distance();
            }
        }
    }

    pub fn set_block(&mut self, partition_id: u16, block_idx: u32, block_pos: [u32; 3]) {
        if let Some(p) = self.get_partition_mut(partition_id) {
            p.add_block(block_idx, block_pos);
        }
    }

    pub fn set_neighbors(&mut self, a_id: u16, b_id: u16) {
        let a = self.partitions.get_mut(&a_id).unwrap();
        a.neighbors.insert(b_id);

        let b = self.partitions.get_mut(&b_id).unwrap();
        b.neighbors.insert(a_id);
    }
}

#[derive(Event)]
pub struct PartitionEvent {
    pub chunk_idx: u32,
    pub refresh: bool,
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
    let neighbors_ids: Vec<u16> = b_partition.neighbors.iter().copied().collect();
    let b_computed = b_partition.is_computed;
    let a_partition = graph.get_partition_mut(a_id).unwrap();
    a_partition.is_computed = a_partition.is_computed && b_computed;

    for block_idx in block_idxs {
        let block_pos = terrain.get_block_world_pos(a_partition.chunk_idx, block_idx);
        a_partition.add_block(block_idx, block_pos);
        terrain.set_partition_id(a_partition.chunk_idx, block_idx, a_id);
    }

    a_partition.extents.update_traversal_distance();

    for neighor_id in neighbors_ids {
        if neighor_id == a_id {
            continue;
        }

        if let Some(neighbor) = graph.get_partition_mut(neighor_id) {
            neighbor.remove_neighbor(b_id);
            graph.set_neighbors(a_id, neighor_id);
        }
    }

    graph.partitions.remove(&b_id);

    a_id
}

pub fn partition(
    mut terrain: ResMut<Terrain>,
    mut graph: ResMut<PartitionGraph>,
    mut partition_ev: EventReader<PartitionEvent>,
) {
    for ev in partition_ev.read() {
        let chunk_idx = ev.chunk_idx;

        if ev.refresh {
            let cleanups = graph.delete_partitions_for_chunk(chunk_idx);
            for p in cleanups {
                for b in p.blocks.iter() {
                    terrain.set_partition_id(p.chunk_idx, *b, Partition::NONE);
                }
            }
        }

        println!("partitioning chunk {}", chunk_idx);
        for block_idx in 0..terrain.chunk_shape.size() {
            let [x, y, z] = terrain.get_block_world_pos(chunk_idx, block_idx);

            let seed_flags = get_block_flags(&terrain, x as i32, y as i32, z as i32);

            // don't partition empty space
            if seed_flags == PartitionFlags::NONE {
                continue;
            }

            let mut partition_id = terrain.get_partition_id(chunk_idx, block_idx);

            if partition_id == Partition::NONE {
                // if we are here, that means the block is navigable,
                // and it is not assigned to a partition yet. We must
                // create a new partition and assign it
                partition_id = graph.create_partition(chunk_idx);
                graph.set_flags(partition_id, seed_flags);
            }

            // if the block is already in a computed partition, it has
            // already been claimed and we can skip it.
            if graph.is_partition_computed(partition_id) {
                continue;
            }

            // next, flood fill from the block, looking for other
            // navigable blocks to add the current partition
            flood_fill([x as i32, y as i32, z as i32], |[nx, ny, nz]| {
                if terrain.is_oob(nx, ny, nz) {
                    return false;
                }

                let [nchunk_idx, nblock_idx] =
                    terrain.get_block_indexes(nx as u32, ny as u32, nz as u32);

                let npartition_id = terrain.get_partition_id(nchunk_idx, nblock_idx);

                // if this block is already assigned to our partition,
                // it means we have already visited it, and we should
                // not flood from it again.
                if npartition_id == partition_id && block_idx != nblock_idx {
                    return false;
                }

                let nblock_flags = get_block_flags(&terrain, nx, ny, nz);

                // this block is not navigable and won't fit in any partition
                if nblock_flags == PartitionFlags::NONE {
                    return false;
                }

                let npartition_flags = graph.get_flags(partition_id);

                // if we are in a different chunk, or if the flags do not match,
                // we must determine which partition this block belongs to, and
                // aassign it as a neighbor
                if nblock_flags != npartition_flags || nchunk_idx != chunk_idx {
                    if npartition_id != Partition::NONE {
                        // a partition already exists, add it as a neighbor
                        graph.set_neighbors(partition_id, npartition_id);
                    } else {
                        // a partition does not exist, create a new one, assign the
                        // block to it, and add it as a neighbor
                        let npartition_id = graph.create_partition(nchunk_idx);
                        graph.set_neighbors(partition_id, npartition_id);

                        terrain.set_partition_id(nchunk_idx, nblock_idx, npartition_id);
                        graph.set_block(
                            npartition_id,
                            nblock_idx,
                            [nx as u32, ny as u32, nz as u32],
                        );
                        graph.set_flags(npartition_id, nblock_flags);
                    }

                    // we are done flooding here, as we will process this neighbor
                    // partition later.
                    return false;
                }

                if npartition_id != Partition::NONE && npartition_id != partition_id {
                    merge_partitions(&mut graph, &mut terrain, partition_id, npartition_id);
                }

                // this block is navigable, it is in the same chunk, and it has
                // matching flags, so we can assign it to the partition and
                // continue flooding.
                terrain.set_partition_id(nchunk_idx, nblock_idx, partition_id);
                graph.set_block(partition_id, nblock_idx, [nx as u32, ny as u32, nz as u32]);

                true
            });

            // we have flooded the partition, we mark it as computed
            graph.set_partition_computed(partition_id, true);
        }
    }
}

pub fn get_block_flags(terrain: &Terrain, x: i32, y: i32, z: i32) -> PartitionFlags {
    let block = terrain.get_block_i32(x, y, z);

    let mut flags = PartitionFlags::NONE;

    if block == Block::LADDER {
        return PartitionFlags::LADDER;
    }

    if !block.is_empty() {
        return PartitionFlags::NONE;
    }

    let nblock_below = terrain.get_block_i32(x, y - 1, z);

    if nblock_below == Block::LADDER {
        return PartitionFlags::LADDER;
    }

    if nblock_below.is_walkable() {
        flags |= PartitionFlags::SOLID_GROUND;

        let nblock_above = terrain.get_block_i32(x, y + 1, z);

        if nblock_above.is_empty() {
            flags |= PartitionFlags::TALL;
        }
    }

    flags
}

pub fn partition_setup(terrain: Res<Terrain>, mut partition_chunk_ev: EventWriter<PartitionEvent>) {
    println!("partitioning world..");

    for chunk_idx in 0..terrain.chunk_count {
        partition_chunk_ev.send(PartitionEvent {
            chunk_idx,
            refresh: false,
        });
    }
    println!("..done partitioning world");
}
