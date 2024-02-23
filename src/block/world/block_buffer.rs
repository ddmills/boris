use ndshape::{RuntimeShape, Shape};

use super::block::Block;

#[derive(Clone)]
pub struct BlockBuffer {
    pub shape: RuntimeShape<u32, 3>,
    pub blocks: Box<[Block]>,
    pub block_count: u32,
    pub chunk_idx: u32,
    pub chunk_size: u32,
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
            chunk_size: 0,
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

    pub fn is_oob(&self, x: i32, y: i32, z: i32) -> bool {
        let chunk_size_i32 = self.chunk_size as i32;
        return x < 0
            || y < 0
            || z < 0
            || x >= chunk_size_i32
            || y >= chunk_size_i32
            || z >= chunk_size_i32;
    }

    pub fn get_block_by_xyz(&self, x: i32, y: i32, z: i32) -> Block {
        if self.is_oob(x, y, z) {
            return Block::OOB;
        }

        let block_idx = self.get_block_idx(x as u32, y as u32, z as u32);

        if let Some(block) = self.blocks.get(block_idx as usize) {
            return *block;
        }

        return Block::OOB;
    }

    pub fn get_immediate_neighbors(&self, x: u32, y: u32, z: u32) -> [Block; 6] {
        let x_i32 = x as i32;
        let y_i32 = y as i32;
        let z_i32 = z as i32;

        return [
            self.get_block_by_xyz(x_i32, y_i32 + 1, z_i32), // above
            self.get_block_by_xyz(x_i32, y_i32, z_i32 - 1), // front
            self.get_block_by_xyz(x_i32 + 1, y_i32, z_i32), // right
            self.get_block_by_xyz(x_i32, y_i32, z_i32 + 1), // behind
            self.get_block_by_xyz(x_i32 - 1, y_i32, z_i32), // left
            self.get_block_by_xyz(x_i32, y_i32 - 1, z_i32), // below
        ];
    }
}
