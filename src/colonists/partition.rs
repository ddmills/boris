use bevy::ecs::event::Event;

use crate::{BlockType, Terrain};

use super::NavigationFlags;

#[derive(Event)]
pub struct PartitionEvent {
    pub chunk_idx: u32,
}

pub fn get_block_flags(terrain: &Terrain, x: i32, y: i32, z: i32) -> NavigationFlags {
    let block = terrain.get_block_type_i32(x, y, z);

    let mut flags = NavigationFlags::NONE;

    if block == BlockType::LADDER {
        return NavigationFlags::LADDER;
    }

    if !block.is_empty() {
        return NavigationFlags::NONE;
    }

    let nblock_below = terrain.get_block_type_i32(x, y - 1, z);

    if nblock_below == BlockType::LADDER {
        return NavigationFlags::LADDER;
    }

    if nblock_below.is_walkable() {
        flags |= NavigationFlags::SOLID_GROUND;

        let nblock_above = terrain.get_block_type_i32(x, y + 1, z);

        if nblock_above.is_empty() {
            flags |= NavigationFlags::TALL;
        }
    }

    flags
}
