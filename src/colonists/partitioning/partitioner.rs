use bevy::ecs::{event::EventReader, system::ResMut};
use ndshape::AbstractShape;

use crate::{
    colonists::{get_block_flags, PartitionEvent},
    common::flood_fill,
    Terrain,
};

use super::NavigationGraph;

pub fn partition(
    mut partition_ev: EventReader<PartitionEvent>,
    mut graph: ResMut<NavigationGraph>,
    mut terrain: ResMut<Terrain>,
) {
    for ev in partition_ev.read() {
        let chunk_idx = ev.chunk_idx;

        println!("partition chunk {}", chunk_idx);

        for block_idx in 0..terrain.chunk_shape.size() {
            let [x, y, z] = terrain.get_block_world_pos(chunk_idx, block_idx);
            let block_flags = get_block_flags(&terrain, x as i32, y as i32, z as i32);

            // ignore empty blocks
            if block_flags.is_empty() {
                continue;
            }

            // get the partition for this block. if it does not exist, create one
            let mut partition_id = terrain
                .get_partition_id(chunk_idx, block_idx)
                .copied()
                .unwrap_or_else(|| {
                    let new_region_id = graph.create_region(block_flags);
                    let new_partition_id =
                        graph.create_partition(new_region_id, chunk_idx, block_flags);
                    let new_partition = graph.get_partition_mut(&new_partition_id).unwrap();

                    terrain.set_partition_id(chunk_idx, block_idx, new_partition_id);
                    new_partition.assign_block(block_idx, [x, y, z]);

                    new_partition_id
                });

            {
                let partition = graph.get_partition_mut(&partition_id).unwrap();

                // if the partition is already computed, we can safely skip this
                if partition.is_computed {
                    continue;
                }
            }

            let mut region_id = graph.get_partition(&partition_id).unwrap().region_id;

            flood_fill([x as i32, y as i32, z as i32], |[nx, ny, nz]| {
                if terrain.is_oob(nx, ny, nz) {
                    return false;
                }

                let [nchunk_idx, nblock_idx] =
                    terrain.get_block_indexes(nx as u32, ny as u32, nz as u32);

                // this is the seed block
                if nblock_idx == block_idx && nchunk_idx == chunk_idx {
                    return true;
                }

                if let Some(npartition_id_ref) = terrain.get_partition_id(nchunk_idx, nblock_idx) {
                    let npartition_id = *npartition_id_ref;

                    // already assigned to this partition
                    if npartition_id == partition_id {
                        return false;
                    }

                    let nblock_flags = get_block_flags(&terrain, nx, ny, nz);

                    if nblock_flags.is_empty() {
                        return false;
                    }

                    let flag_diff = nblock_flags != block_flags;
                    let chunk_diff = nchunk_idx != chunk_idx;

                    if flag_diff || chunk_diff {
                        // add neighbor
                        graph.set_partition_neighbors(&partition_id, &npartition_id);

                        if !flag_diff {
                            // todo: merge regions
                        }

                        return false;
                    }

                    println!("merge partitions {} {}", partition_id, npartition_id);
                    partition_id =
                        graph.merge_partitions(&partition_id, &npartition_id, &mut terrain);
                    region_id = graph.get_partition(&partition_id).unwrap().region_id;

                    return true;
                }

                let nblock_flags = get_block_flags(&terrain, nx, ny, nz);

                if nblock_flags.is_empty() {
                    return false;
                }

                let flag_diff = nblock_flags != block_flags;
                let chunk_diff = nchunk_idx != chunk_idx;

                if flag_diff || chunk_diff {
                    // if flags are the same, we add to existing region, otherwise we make
                    // a new region and add it as a neighbor.
                    let nregion_id = if flag_diff {
                        let new_region_id = graph.create_region(nblock_flags);
                        graph.set_region_neighbors(&region_id, &new_region_id);
                        new_region_id
                    } else {
                        region_id
                    };

                    let npartition_id =
                        graph.create_partition(nregion_id, nchunk_idx, nblock_flags);

                    terrain.set_partition_id(nchunk_idx, nblock_idx, npartition_id);
                    graph.assign_block(
                        &npartition_id,
                        nblock_idx,
                        [nx as u32, ny as u32, nz as u32],
                        &mut terrain,
                    );

                    return false;
                }

                terrain.set_partition_id(nchunk_idx, nblock_idx, partition_id);
                graph.assign_block(
                    &partition_id,
                    nblock_idx,
                    [nx as u32, ny as u32, nz as u32],
                    &mut terrain,
                );

                true
            });

            let partition = graph.get_partition_mut(&partition_id).unwrap();
            partition.is_computed = true;
        }
    }
}
