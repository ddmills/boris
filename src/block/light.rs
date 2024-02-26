use bevy::render::render_resource::encase::rts_array::Length;

use super::world::{block::Block, terrain::Terrain};

struct LightNode {
    chunk_idx: u32,
    block_idx: u32,
}

pub fn update_light(terrain: &mut Terrain, chunk_idx: u32, block_idx: u32) {
    let mut queue: Vec<LightNode> = vec![];
    let mut count = 0;

    queue.push(LightNode {
        chunk_idx,
        block_idx,
    });

    while !queue.is_empty() {
        let node = queue.remove(0);

        let pos = terrain.get_block_world_position(node.chunk_idx, node.block_idx);
        let block = terrain.get_block_detail(pos[0], pos[1], pos[2]);

        if block.block.is_opaque() && !block.block.is_light_source() {
            continue;
        }

        let world_x = pos[0] as i32;
        let world_y = pos[1] as i32;
        let world_z = pos[2] as i32;

        let neighbors = [
            [world_x + 1, world_y, world_z],
            [world_x - 1, world_y, world_z],
            [world_x, world_y + 1, world_z],
            [world_x, world_y - 1, world_z],
            [world_x, world_y, world_z - 1],
            [world_x, world_y, world_z + 1],
        ];

        count = count + 1;

        for [n_x, n_y, n_z] in neighbors {
            let n_detail = terrain.get_block_detail_i32(n_x, n_y, n_z);

            if n_detail.light + 2 <= block.light && !n_detail.block.is_opaque() {
                let [n_chunk_idx, n_block_idx] =
                    terrain.get_block_indexes(n_x as u32, n_y as u32, n_z as u32);

                terrain.set_torchlight(n_x as u32, n_y as u32, n_z as u32, block.light - 1);

                queue.push(LightNode {
                    chunk_idx: n_chunk_idx,
                    block_idx: n_block_idx,
                });
            }
        }
    }

    println!("checked {} blocks!", count);
}
