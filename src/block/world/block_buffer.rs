use ndshape::{RuntimeShape, Shape};

use super::block::Block;

#[derive(Clone)]
pub struct BlockBuffer {
    pub shape: RuntimeShape<u32, 3>,
    pub blocks: Box<[Block]>,
    pub light: Box<[u8]>,
    pub block_count: u32,
    pub chunk_idx: u32,
    pub chunk_size: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
    pub is_dirty: bool,
}

impl BlockBuffer {
    pub fn new(shape: RuntimeShape<u32, 3>) -> Self {
        Self {
            blocks: vec![Block::EMPTY; shape.size() as usize].into_boxed_slice(),
            light: vec![0; shape.size() as usize].into_boxed_slice(),
            block_count: shape.size(),
            shape: shape,
            chunk_idx: 0,
            chunk_size: 0,
            world_x: 0,
            world_y: 0,
            world_z: 0,
            is_dirty: true,
        }
    }

    pub fn set_block(&mut self, block_idx: u32, value: Block) {
        self.blocks[block_idx as usize] = value;
        self.is_dirty = true;
    }

    pub fn get_block(&self, block_idx: u32) -> Block {
        if let Some(block) = self.blocks.get(block_idx as usize) {
            return *block;
        }

        return Block::OOB;
    }

    pub fn get_sunlight(&self, block_idx: u32) -> u8 {
        if let Some(light) = self.light.get(block_idx as usize) {
            return light >> 4 & 0xf;
        }

        return 0;
    }

    pub fn get_torchlight(&self, block_idx: u32) -> u8 {
        if let Some(light) = self.light.get(block_idx as usize) {
            return light & 0xf;
        }

        return 0;
    }

    #[inline]
    pub fn set_sunlight(&mut self, block_idx: u32, value: u8) {
        self.light[block_idx as usize] =
            self.light[block_idx as usize] & 0xf | ((value << 4) & 0xf0);
        self.is_dirty = true;
    }

    #[inline]
    pub fn set_torchlight(&mut self, block_idx: u32, value: u8) {
        self.light[block_idx as usize] = (self.light[block_idx as usize] & 0xf0) | (value & 0xf);
        self.is_dirty = true;
    }
}

pub struct Neighbor(pub u8);

impl Neighbor {
    pub const ABOVE_FORWARD_LEFT: Self = Self(0);
    pub const ABOVE_FORWARD: Self = Self(1);
    pub const ABOVE_FORWARD_RIGHT: Self = Self(2);
    pub const ABOVE_LEFT: Self = Self(3);
    pub const ABOVE: Self = Self(4);
    pub const ABOVE_RIGHT: Self = Self(5);
    pub const ABOVE_BEHIND_LEFT: Self = Self(6);
    pub const ABOVE_BEHIND: Self = Self(7);
    pub const ABOVE_BEHIND_RIGHT: Self = Self(8);
    pub const FORWARD_LEFT: Self = Self(9);
    pub const FORWARD: Self = Self(10);
    pub const FORWARD_RIGHT: Self = Self(11);
    pub const LEFT: Self = Self(12);
    pub const RIGHT: Self = Self(13);
    pub const BEHIND_LEFT: Self = Self(14);
    pub const BEHIND: Self = Self(15);
    pub const BEHIND_RIGHT: Self = Self(16);
    pub const BELOW_FORWARD_LEFT: Self = Self(17);
    pub const BELOW_FORWARD: Self = Self(18);
    pub const BELOW_FORWARD_RIGHT: Self = Self(19);
    pub const BELOW_LEFT: Self = Self(20);
    pub const BELOW: Self = Self(21);
    pub const BELOW_RIGHT: Self = Self(22);
    pub const BELOW_BEHIND_LEFT: Self = Self(23);
    pub const BELOW_BEHIND: Self = Self(24);
    pub const BELOW_BEHIND_RIGHT: Self = Self(25);

    pub fn idx(&self) -> usize {
        return self.0 as usize;
    }
}
