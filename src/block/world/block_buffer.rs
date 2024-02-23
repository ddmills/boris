use ndshape::{RuntimeShape, Shape};

use super::block::Block;

#[derive(Clone)]
pub struct BlockBuffer {
    pub shape: RuntimeShape<u32, 3>,
    pub blocks: Box<[Block]>,
    pub block_count: u32,
    pub chunk_idx: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
}

impl BlockBuffer {
    pub fn new(shape: RuntimeShape<u32, 3>) -> Self {
        Self {
            blocks: vec![Block::EMPTY; shape.size() as usize].into_boxed_slice(),
            block_count: shape.size(),
            shape: shape,
            chunk_idx: 0,
            world_x: 0,
            world_y: 0,
            world_z: 0,
        }
    }

    pub fn set(&mut self, block_idx: u32, value: Block) {
        self.blocks[block_idx as usize] = value;
    }

    pub fn get_block_idx(&self, x: u32, y: u32, z: u32) -> u32 {
        return self.shape.linearize([x, y, z]);
    }

    pub fn get_block_xyz(&self, block_idx: u32) -> [u32; 3] {
        return self.shape.delinearize(block_idx);
    }

    pub fn get_block(&self, block_idx: u32) -> Block {
        if let Some(block) = self.blocks.get(block_idx as usize) {
            return *block;
        }

        return Block::OOB;
    }

    pub fn get_block_by_xyz(&self, x: u32, y: u32, z: u32) -> Block {
        let block_idx = self.get_block_idx(x, y, z);

        if let Some(block) = self.blocks.get(block_idx as usize) {
            return *block;
        }

        return Block::OOB;
    }

    pub fn get_immediate_neighbors(&self, x: u32, y: u32, z: u32) -> [Block; 6] {
        return [
            self.get_block_by_xyz(x, y + 1, z), // above
            self.get_block_by_xyz(x, y, z - 1), // front
            self.get_block_by_xyz(x + 1, y, z), // right
            self.get_block_by_xyz(x, y, z + 1), // behind
            self.get_block_by_xyz(x - 1, y, z), // left
            self.get_block_by_xyz(x, y - 1, z), // below
        ];
    }
}
