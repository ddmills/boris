use bevy::ecs::system::Resource;
use ndshape::{RuntimeShape, Shape};

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

pub struct RayResult {
    pub is_hit: bool,
    pub block: Block,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub attempts: u32,
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

    pub fn world_size_y(&self) -> u32 {
        return self.chunk_count_y * self.chunk_size;
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

    pub fn raycast(
        &self,
        origin_x: f32,
        origin_y: f32,
        origin_z: f32,
        direction_x: f32,
        direction_y: f32,
        direction_z: f32,
        slice_y: u32,
        radius: u32,
    ) -> RayResult {
        let mut x = (origin_x).floor() as i32;
        let mut y = (origin_y).floor() as i32;
        let mut z = (origin_z).floor() as i32;

        let step_x = sig_num(direction_x);
        let step_y = sig_num(direction_y);
        let step_z = sig_num(direction_z);

        let mut t_max_x = int_bound(origin_x, direction_x);
        let mut t_max_y = int_bound(origin_y, direction_y);
        let mut t_max_z = int_bound(origin_z, direction_z);

        let t_delta_x = step_x as f32 / direction_x;
        let t_delta_y = step_y as f32 / direction_y;
        let t_delta_z = step_z as f32 / direction_z;

        if direction_x == 0. && direction_y == 0. && direction_z == 0. {
            return RayResult {
                is_hit: false,
                block: Block::OOB,
                x: 0,
                y: 0,
                z: 0,
                attempts: 0,
            };
        }

        let r = radius as f32
            / f32::sqrt(
                direction_x * direction_x + direction_y * direction_y + direction_z * direction_z,
            );

        let wx = self.world_size_x() as i32;
        let wy = self.world_size_y() as i32;
        let wz = self.world_size_z() as i32;
        let mut attempts = 0;

        while (if step_x > 0 { x < wx } else { x >= 0 }
            && if step_y > 0 { y < wy } else { y >= 0 }
            && if step_z > 0 { z < wz } else { z >= 0 })
        {
            attempts = attempts + 1;
            if !(y > slice_y as i32 || x < 0 || y < 0 || z < 0 || x > wx || y > wy || z > wz) {
                let b = self.get_block(x as u32, y as u32, z as u32);
                if b.is_filled() {
                    return RayResult {
                        is_hit: true,
                        block: b,
                        x: x as u32,
                        y: y as u32,
                        z: z as u32,
                        attempts: attempts,
                    };
                }
            }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x > r {
                        break;
                    }
                    x = x + step_x;
                    t_max_x = t_max_x + t_delta_x;
                } else {
                    if t_max_x > r {
                        break;
                    }
                    z = z + step_z;
                    t_max_z = t_max_z + t_delta_z;
                }
            } else {
                if t_max_y < t_max_z {
                    if t_max_y > r {
                        break;
                    }
                    y = y + step_y;
                    t_max_y = t_max_y + t_delta_y;
                } else {
                    if t_max_z > r {
                        break;
                    }
                    z = z + step_z;
                    t_max_z = t_max_z + t_delta_z;
                }
            }
        }

        return RayResult {
            is_hit: false,
            block: Block::OOB,
            x: 0,
            y: 0,
            z: 0,
            attempts: attempts,
        };
    }
}

fn sig_num(v: f32) -> i32 {
    return if v > 0. {
        1
    } else if v < 0. {
        -1
    } else {
        0
    };
}

fn int_bound(s: f32, ds: f32) -> f32 {
    if ds < 0. {
        return int_bound(-s, -ds);
    }

    let m = (s % 1. + 1.) % 1.;

    return (1. - m) / ds;
}
