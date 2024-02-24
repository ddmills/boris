use bevy::ecs::system::Resource;
use ndshape::{RuntimeShape, Shape};

use crate::block::meshing::chunk_material::ChunkMaterial;

use super::{block::Block, block_buffer::BlockBuffer};

#[derive(Resource)]
pub struct Terrain {
    pub chunk_count_x: u32,
    pub chunk_count_y: u32,
    pub chunk_count_z: u32,
    pub chunk_size: u32,
    pub chunk_count: u32,
    pub shape: RuntimeShape<u32, 3>,
    pub chunks: Vec<BlockBuffer>,
}

impl Terrain {
    pub fn new(
        chunk_count_x: u32,
        chunk_count_y: u32,
        chunk_count_z: u32,
        chunk_size: u32,
    ) -> Self {
        let shape = RuntimeShape::<u32, 3>::new([chunk_count_x, chunk_count_y, chunk_count_z]);
        let chunk_shape = RuntimeShape::<u32, 3>::new([chunk_size, chunk_size, chunk_size]);

        return Self {
            chunk_count_x: chunk_count_x,
            chunk_count_y: chunk_count_y,
            chunk_count_z: chunk_count_z,
            chunk_size: chunk_size,
            chunk_count: shape.size(),
            chunks: vec![BlockBuffer::new(chunk_shape); shape.size() as usize],
            shape: shape,
        };
    }

    pub fn world_size_x(&self) -> u32 {
        return self.chunk_count_x * self.chunk_size;
    }

    pub fn world_size_z(&self) -> u32 {
        return self.chunk_count_z * self.chunk_size;
    }

    pub fn get_chunk(&self, chunk_idx: u32) -> Option<&BlockBuffer> {
        return self.chunks.get(chunk_idx as usize);
    }

    pub fn get_chunk_mut(&mut self, chunk_idx: u32) -> Option<&mut BlockBuffer> {
        return self.chunks.get_mut(chunk_idx as usize);
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Block {
        let chunk_pos = [
            x / self.chunk_size,
            y / self.chunk_size,
            z / self.chunk_size,
        ];
        let chunk_idx = self.shape.linearize(chunk_pos);
        if let Some(chunk) = self.get_chunk(chunk_idx) {
            let block_pos = [
                x % self.chunk_size,
                y % self.chunk_size,
                z % self.chunk_size,
            ];
            let block_idx = chunk.get_block_idx(block_pos[0], block_pos[1], block_pos[2]);
            return chunk.get_block(block_idx);
        }

        return Block::OOB;
    }

    pub fn get_chunks_in_y(&self, world_y: u32) -> Vec<u32> {
        let chunk_y = world_y / self.chunk_size;
        let mut chunk_idxes: Vec<u32> = vec![];

        for chunk_x in 0..self.chunk_count_x {
            for chunk_z in 0..self.chunk_count_z {
                let chunk_idx = self.shape.linearize([chunk_x, chunk_y, chunk_z]);
                chunk_idxes.push(chunk_idx);
            }
        }

        return chunk_idxes;
    }
}
