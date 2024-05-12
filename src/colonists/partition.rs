use crate::Terrain;

use super::NavigationFlags;

pub fn get_block_flags(terrain: &Terrain, x: i32, y: i32, z: i32) -> NavigationFlags {
    let block = terrain.get_block_i32(x, y, z);

    let mut flags = NavigationFlags::NONE;

    if !block.is_empty() {
        return NavigationFlags::NONE;
    }

    if !block.is_oob() {
        let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
        let structures = terrain.get_structures(chunk_idx, block_idx);

        for tile in structures.values() {
            if !tile.is_built {
                continue;
            }

            if let Some(tile_flags) = tile.flags {
                return tile_flags;
            }

            if tile.is_blocker {
                return NavigationFlags::NONE;
            }
        }
    }

    let nblock_below = terrain.get_block_i32(x, y - 1, z);

    if nblock_below.is_walkable() {
        flags |= NavigationFlags::SHORT;

        let nblock_above = terrain.get_block_i32(x, y + 1, z);

        if nblock_above.is_empty() {
            flags |= NavigationFlags::TALL;
        }
    } else if nblock_below.is_empty() {
        let nblock_below2 = terrain.get_block_i32(x, y - 2, z);
        let nblock_above = terrain.get_block_i32(x, y + 1, z);

        if nblock_below2.is_walkable() && nblock_above.is_empty() {
            let left = terrain.get_block_i32(x - 1, y, z);
            let right = terrain.get_block_i32(x + 1, y, z);
            let fwd = terrain.get_block_i32(x, y, z - 1);
            let back = terrain.get_block_i32(x, y, z + 1);

            let below_left = terrain.get_block_i32(x - 1, y - 1, z);
            let below_right = terrain.get_block_i32(x + 1, y - 1, z);
            let below_fwd = terrain.get_block_i32(x, y - 1, z - 1);
            let below_back = terrain.get_block_i32(x, y - 1, z + 1);

            if (left.is_empty() && below_left.is_walkable())
                || (right.is_empty() && below_right.is_walkable())
                || (fwd.is_empty() && below_fwd.is_walkable())
                || (back.is_empty() && below_back.is_walkable())
            {
                flags |= NavigationFlags::CLIMB;
            }
        }
    }

    flags
}
