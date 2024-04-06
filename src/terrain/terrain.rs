use bevy::ecs::system::Resource;
use ndshape::{RuntimeShape, Shape};

use crate::{common::sig_num, Block, BlockBuffer, BlockDetail, BlockFace, LightNode};

#[derive(Resource)]
pub struct Terrain {
    pub chunk_count_x: u32,
    pub chunk_count_y: u32,
    pub chunk_count_z: u32,
    pub chunk_size: u32,
    pub chunk_count: u32,
    pub shape: RuntimeShape<u32, 3>,
    pub chunk_shape: RuntimeShape<u32, 3>,
    pub chunks: Vec<BlockBuffer>,
    pub lights_queue_add: Vec<LightNode>,
    pub lights_queue_remove: Vec<LightNode>,
    pub sunlight_queue_add: Vec<LightNode>,
    pub sunlight_queue_remove: Vec<LightNode>,
}

pub struct RayResult {
    pub is_hit: bool,
    pub block: Block,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub attempts: u32,
    pub face: BlockFace,
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

        Self {
            chunk_count_x,
            chunk_count_y,
            chunk_count_z,
            chunk_size,
            chunk_count: shape.size(),
            chunk_shape: chunk_shape.clone(),
            chunks: vec![BlockBuffer::new(chunk_shape); shape.size() as usize],
            shape,
            lights_queue_add: vec![],
            lights_queue_remove: vec![],
            sunlight_queue_add: vec![],
            sunlight_queue_remove: vec![],
        }
    }

    pub fn init_chunk(&mut self, chunk_idx: u32) {
        let chunk_pos = self.shape.delinearize(chunk_idx);
        let chunk = self.chunks.get_mut(chunk_idx as usize).unwrap();

        chunk.chunk_idx = chunk_idx;
        chunk.world_x = self.chunk_size * chunk_pos[0];
        chunk.world_y = self.chunk_size * chunk_pos[1];
        chunk.world_z = self.chunk_size * chunk_pos[2];
        chunk.chunk_size = self.chunk_size;
    }

    pub fn world_size_x(&self) -> u32 {
        self.chunk_count_x * self.chunk_size
    }

    pub fn world_size_y(&self) -> u32 {
        self.chunk_count_y * self.chunk_size
    }

    pub fn world_size_z(&self) -> u32 {
        self.chunk_count_z * self.chunk_size
    }

    pub fn is_oob(&self, x: i32, y: i32, z: i32) -> bool {
        x < 0
            || y < 0
            || z < 0
            || x >= self.world_size_x() as i32
            || y >= self.world_size_y() as i32
            || z >= self.world_size_z() as i32
    }

    pub fn get_chunk(&self, chunk_idx: u32) -> Option<&BlockBuffer> {
        return self.chunks.get(chunk_idx as usize);
    }

    pub fn get_chunk_dirty(&self, chunk_idx: u32) -> bool {
        if let Some(chunk) = self.chunks.get(chunk_idx as usize) {
            return chunk.is_dirty;
        }
        false
    }

    pub fn set_chunk_dirty(&mut self, chunk_idx: u32, value: bool) {
        if let Some(chunk) = self.chunks.get_mut(chunk_idx as usize) {
            chunk.is_dirty = value;
        }
    }

    pub fn get_chunk_mut(&mut self, chunk_idx: u32) -> Option<&mut BlockBuffer> {
        return self.chunks.get_mut(chunk_idx as usize);
    }

    pub fn get_chunk_offset(&self, chunk_idx: u32) -> [u32; 3] {
        let pos = self.shape.delinearize(chunk_idx);

        [
            pos[0] * self.chunk_size,
            pos[1] * self.chunk_size,
            pos[2] * self.chunk_size,
        ]
    }

    pub fn get_block_world_pos(&self, chunk_idx: u32, block_idx: u32) -> [u32; 3] {
        let chunk_pos = self.get_chunk_offset(chunk_idx);
        let block_pos = self.chunk_shape.delinearize(block_idx);

        [
            chunk_pos[0] + block_pos[0],
            chunk_pos[1] + block_pos[1],
            chunk_pos[2] + block_pos[2],
        ]
    }

    pub fn get_block_indexes(&self, x: u32, y: u32, z: u32) -> [u32; 2] {
        let chunk_pos = [
            x / self.chunk_size,
            y / self.chunk_size,
            z / self.chunk_size,
        ];
        let block_pos = [
            x % self.chunk_size,
            y % self.chunk_size,
            z % self.chunk_size,
        ];
        let chunk_idx = self.shape.linearize(chunk_pos);
        let block_idx = self.chunk_shape.linearize(block_pos);

        [chunk_idx, block_idx]
    }

    pub fn set_block(&mut self, x: u32, y: u32, z: u32, value: Block) {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        if let Some(chunk) = self.get_chunk_mut(chunk_idx) {
            chunk.set_block(block_idx, value);
            self.remove_sunlight(x, y, z);

            if value.is_light() {
                self.add_light(x, y, z, value.get_light_level());
            } else {
                self.remove_light(x, y, z);
            }
        }

        let local_x = x % self.chunk_size;
        let local_y = y % self.chunk_size;
        let local_z = z % self.chunk_size;
        let chunk_x = x / self.chunk_size;
        let chunk_y = y / self.chunk_size;
        let chunk_z = z / self.chunk_size;

        // what chunks does this block touch?
        if local_x == 0 && x > 0 {
            // update chunk left
            let left_chunk_idx = self.shape.linearize([chunk_x - 1, chunk_y, chunk_z]);
            self.set_chunk_dirty(left_chunk_idx, true);
        } else if local_x == self.chunk_size - 1 && x < self.world_size_x() - 1 {
            // update chunk right
            let right_chunk_idx = self.shape.linearize([chunk_x + 1, chunk_y, chunk_z]);
            self.set_chunk_dirty(right_chunk_idx, true);
        }

        if local_y == 0 && y > 0 {
            // update chunk below
            let below_chunk_idx = self.shape.linearize([chunk_x, chunk_y - 1, chunk_z]);
            self.set_chunk_dirty(below_chunk_idx, true);
        } else if local_y == self.chunk_size - 1 && y < self.world_size_y() - 1 {
            // update chunk above
            let above_chunk_idx = self.shape.linearize([chunk_x, chunk_y + 1, chunk_z]);
            self.set_chunk_dirty(above_chunk_idx, true);
        }

        if local_z == 0 && z > 0 {
            // update chunk forward
            let forward_chunk_idx = self.shape.linearize([chunk_x, chunk_y, chunk_z - 1]);
            self.set_chunk_dirty(forward_chunk_idx, true);
        } else if local_z == self.chunk_size - 1 && z < self.world_size_z() - 1 {
            // update chunk behind
            let behind_chunk_idx = self.shape.linearize([chunk_x, chunk_y, chunk_z + 1]);
            self.set_chunk_dirty(behind_chunk_idx, true);
        }
    }

    pub fn init_block(&mut self, x: u32, y: u32, z: u32, value: Block) {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        let chunk = self.get_chunk_mut(chunk_idx).unwrap();
        chunk.set_block(block_idx, value);

        if value.is_light() {
            self.add_light(x, y, z, value.get_light_level());
        }
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Block {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        self.get_block_by_idx(chunk_idx, block_idx)
    }

    pub fn get_block_detail(&self, x: u32, y: u32, z: u32) -> BlockDetail {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        self.get_block_detail_by_idx(chunk_idx, block_idx)
    }

    pub fn get_block_detail_by_idx(&self, chunk_idx: u32, block_idx: u32) -> BlockDetail {
        if let Some(chunk) = self.get_chunk(chunk_idx) {
            let block = chunk.get_block(block_idx);
            let light = chunk.get_torchlight(block_idx);
            let sunlight = chunk.get_sunlight(block_idx);
            return BlockDetail {
                block,
                light,
                sunlight,
            };
        }

        BlockDetail {
            block: Block::OOB,
            light: 0,
            sunlight: 0,
        }
    }

    pub fn get_block_by_idx(&self, chunk_idx: u32, block_idx: u32) -> Block {
        if let Some(chunk) = self.get_chunk(chunk_idx) {
            return chunk.get_block(block_idx);
        }

        Block::OOB
    }

    pub fn add_light(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.set_torchlight(x, y, z, value);
        self.lights_queue_add.push(LightNode { x, y, z, value });
    }

    pub fn remove_light(&mut self, x: u32, y: u32, z: u32) {
        let value = self.get_torchlight_xyz(x, y, z);
        self.set_torchlight(x, y, z, 0);
        self.lights_queue_remove.push(LightNode { x, y, z, value });
    }

    pub fn add_sunlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.set_sunlight(x, y, z, value);
        self.sunlight_queue_add.push(LightNode { x, y, z, value });
    }

    pub fn remove_sunlight(&mut self, x: u32, y: u32, z: u32) {
        let value = self.get_sunlight_xyz(x, y, z);
        self.set_sunlight(x, y, z, 0);
        self.sunlight_queue_remove
            .push(LightNode { x, y, z, value });
    }

    pub fn set_sunlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        if let Some(chunk) = self.get_chunk_mut(chunk_idx) {
            chunk.set_sunlight(block_idx, value);
        }
    }

    pub fn set_torchlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        if let Some(chunk) = self.get_chunk_mut(chunk_idx) {
            chunk.set_torchlight(block_idx, value);
        }
    }

    pub fn get_sunlight(&self, chunk_idx: u32, block_idx: u32) -> u8 {
        if let Some(chunk) = self.get_chunk(chunk_idx) {
            return chunk.get_sunlight(block_idx);
        }

        0
    }

    pub fn get_sunlight_xyz(&self, x: u32, y: u32, z: u32) -> u8 {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        self.get_sunlight(chunk_idx, block_idx)
    }

    pub fn get_torchlight_xyz(&self, x: u32, y: u32, z: u32) -> u8 {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        self.get_torchlight(chunk_idx, block_idx)
    }

    pub fn get_torchlight(&self, chunk_idx: u32, block_idx: u32) -> u8 {
        if let Some(chunk) = self.get_chunk(chunk_idx) {
            return chunk.get_torchlight(block_idx);
        }

        0
    }

    pub fn get_block_i32(&self, x: i32, y: i32, z: i32) -> Block {
        if self.is_oob(x, y, z) {
            return Block::OOB;
        }

        self.get_block(x as u32, y as u32, z as u32)
    }

    pub fn get_block_detail_i32(&self, x: i32, y: i32, z: i32) -> BlockDetail {
        if self.is_oob(x, y, z) {
            return BlockDetail {
                block: Block::OOB,
                light: 0,
                sunlight: 0,
            };
        }

        self.get_block_detail(x as u32, y as u32, z as u32)
    }

    pub fn set_partition_id(&mut self, chunk_idx: u32, block_idx: u32, value: u32) {
        if let Some(chunk) = self.get_chunk_mut(chunk_idx) {
            chunk.set_partition_id(block_idx, value);
        }
    }

    pub fn get_partition_id(&self, chunk_idx: u32, block_idx: u32) -> Option<&u32> {
        let chunk = self.get_chunk(chunk_idx)?;

        return chunk.get_partition_id(block_idx);
    }

    pub fn get_partition_id_u32(&self, x: u32, y: u32, z: u32) -> Option<&u32> {
        let [chunk_idx, block_idx] = self.get_block_indexes(x, y, z);

        let chunk = self.get_chunk(chunk_idx)?;

        chunk.get_partition_id(block_idx)
    }

    #[allow(dead_code)]
    pub fn get_neighbors_immediate(&self, x: u32, y: u32, z: u32) -> [Block; 6] {
        [
            self.get_block(x + 1, y, z),
            self.get_block(x - 1, y, z),
            self.get_block(x, y + 1, z),
            self.get_block(x, y - 1, z),
            self.get_block(x, y, z + 1),
            self.get_block(x, y, z - 1),
        ]
    }

    pub fn get_neighbors_detail(&self, x: u32, y: u32, z: u32) -> [BlockDetail; 26] {
        let x_i32 = x as i32;
        let y_i32 = y as i32;
        let z_i32 = z as i32;

        let above = y_i32 + 1;
        let below = y_i32 - 1;
        let left = x_i32 - 1;
        let right = x_i32 + 1;
        let forward = z_i32 - 1;
        let behind = z_i32 + 1;

        [
            // ABOVE
            self.get_block_detail_i32(left, above, forward), // above, forward, left -- 0
            self.get_block_detail_i32(x_i32, above, forward), // above, forward, middle -- 1
            self.get_block_detail_i32(right, above, forward), // above, forward, right -- 2
            self.get_block_detail_i32(left, above, z_i32),   // above, left -- 3
            self.get_block_detail_i32(x_i32, above, z_i32),  // above -- 4
            self.get_block_detail_i32(right, above, z_i32),  // above, right -- 5
            self.get_block_detail_i32(left, above, behind),  // above, behind, left -- 6
            self.get_block_detail_i32(x_i32, above, behind), // above, behind, middle -- 7
            self.get_block_detail_i32(right, above, behind), // above, behind, right -- 8
            // MIDDLE
            self.get_block_detail_i32(left, y_i32, forward), // middle, forward, left -- 9
            self.get_block_detail_i32(x_i32, y_i32, forward), // middle, forward, middle -- 10
            self.get_block_detail_i32(right, y_i32, forward), // middle, forward, right -- 11
            self.get_block_detail_i32(left, y_i32, z_i32),   // middle, left -- 12
            self.get_block_detail_i32(right, y_i32, z_i32),  // middle, right -- 13
            self.get_block_detail_i32(left, y_i32, behind),  // middle, behind, left -- 14
            self.get_block_detail_i32(x_i32, y_i32, behind), // middle, behind, middle -- 15
            self.get_block_detail_i32(right, y_i32, behind), // middle, behind, right -- 16
            // BELOW
            self.get_block_detail_i32(left, below, forward), // below, forward, left -- 17
            self.get_block_detail_i32(x_i32, below, forward), // below, forward, middle -- 18
            self.get_block_detail_i32(right, below, forward), // below, forward, right -- 19
            self.get_block_detail_i32(left, below, z_i32),   // below, left -- 20
            self.get_block_detail_i32(x_i32, below, z_i32),  // below -- 21
            self.get_block_detail_i32(right, below, z_i32),  // below, right -- 22
            self.get_block_detail_i32(left, below, behind),  // below, behind, left -- 23
            self.get_block_detail_i32(x_i32, below, behind), // below, behind, middle -- 24
            self.get_block_detail_i32(right, below, behind), // below, behind, right -- 25
        ]
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

        let mut face = BlockFace::PosY;

        if direction_x == 0. && direction_y == 0. && direction_z == 0. {
            return RayResult {
                is_hit: false,
                block: Block::OOB,
                x: 0,
                y: 0,
                z: 0,
                attempts: 0,
                face,
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
            && if step_z > 0 { z < wz } else { z >= 0 }
            && attempts < 1000)
        {
            attempts += 1;
            if !(y >= slice_y as i32 || x < 0 || y < 0 || z < 0 || x > wx || y > wy || z > wz) {
                let b = self.get_block(x as u32, y as u32, z as u32);
                if b.is_filled() {
                    return RayResult {
                        is_hit: true,
                        block: b,
                        x: x as u32,
                        y: y as u32,
                        z: z as u32,
                        attempts,
                        face,
                    };
                }
            }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x > r {
                        break;
                    }
                    x += step_x;
                    t_max_x += t_delta_x;
                    face = if step_x > 0 {
                        BlockFace::NegX
                    } else {
                        BlockFace::PosX
                    };
                } else {
                    if t_max_x > r {
                        break;
                    }
                    z += step_z;
                    t_max_z += t_delta_z;
                    face = if step_z > 0 {
                        BlockFace::NegZ
                    } else {
                        BlockFace::PosZ
                    };
                }
            } else if t_max_y < t_max_z {
                if t_max_y > r {
                    break;
                }
                y += step_y;
                t_max_y += t_delta_y;
                face = if step_y > 0 {
                    BlockFace::NegY
                } else {
                    BlockFace::PosY
                };
            } else {
                if t_max_z > r {
                    break;
                }
                z += step_z;
                t_max_z += t_delta_z;
                face = if step_z > 0 {
                    BlockFace::NegZ
                } else {
                    BlockFace::PosZ
                };
            }
        }

        RayResult {
            is_hit: false,
            block: Block::OOB,
            x: 0,
            y: 0,
            z: 0,
            attempts,
            face: BlockFace::PosY,
        }
    }
}

fn int_bound(s: f32, ds: f32) -> f32 {
    if ds < 0. {
        return int_bound(-s, -ds);
    }

    let m = (s % 1. + 1.) % 1.;

    (1. - m) / ds
}
