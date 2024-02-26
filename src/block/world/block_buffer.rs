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

    pub fn set(&mut self, block_idx: u32, value: Block) {
        self.blocks[block_idx as usize] = value;
        self.is_dirty = true;
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

    // pub fn get_sunlight(&self, block_idx: u32) -> u8 {
    //     if let Some(light) = self.light.get(block_idx as usize) {
    //         return light >> 4 & 0xf;
    //     }

    //     return 0;
    // }

    pub fn get_torchlight(&self, block_idx: u32) -> u8 {
        if let Some(light) = self.light.get(block_idx as usize) {
            return *light;
        }

        return 0;
    }

    // #[inline]
    // pub fn set_sunlight(&mut self, block_idx: u32, value: u8) {
    //     self.light[block_idx as usize] = self.get_torchlight(block_idx) | (value << 4);
    // }

    #[inline]
    pub fn set_torchlight(&mut self, block_idx: u32, value: u8) {
        self.light[block_idx as usize] = value;
        self.is_dirty = true;
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

    pub fn get_neighbors(&self, x: u32, y: u32, z: u32) -> [Block; 26] {
        let x_i32 = x as i32;
        let y_i32 = y as i32;
        let z_i32 = z as i32;

        let above = y_i32 + 1;
        let below = y_i32 - 1;
        let left = x_i32 - 1;
        let right = x_i32 + 1;
        let forward = z_i32 - 1;
        let behind = z_i32 + 1;

        return [
            // ABOVE
            self.get_block_by_xyz(left, above, forward), // above, forward, left -- 0
            self.get_block_by_xyz(x_i32, above, forward), // above, forward, middle -- 1
            self.get_block_by_xyz(right, above, forward), // above, forward, right -- 2
            self.get_block_by_xyz(left, above, z_i32),   // above, left -- 3
            self.get_block_by_xyz(x_i32, above, z_i32),  // above -- 4
            self.get_block_by_xyz(right, above, z_i32),  // above, right -- 5
            self.get_block_by_xyz(left, above, behind),  // above, behind, left -- 6
            self.get_block_by_xyz(x_i32, above, behind), // above, behind, middle -- 7
            self.get_block_by_xyz(right, above, behind), // above, behind, right -- 8
            // MIDDLE
            self.get_block_by_xyz(left, y_i32, forward), // middle, forward, left -- 9
            self.get_block_by_xyz(x_i32, y_i32, forward), // middle, forward, middle -- 10
            self.get_block_by_xyz(right, y_i32, forward), // middle, forward, right -- 11
            self.get_block_by_xyz(left, y_i32, z_i32),   // middle, left -- 12
            self.get_block_by_xyz(right, y_i32, z_i32),  // middle, right -- 13
            self.get_block_by_xyz(left, y_i32, behind),  // middle, behind, left -- 14
            self.get_block_by_xyz(x_i32, y_i32, behind), // middle, behind, middle -- 15
            self.get_block_by_xyz(right, y_i32, behind), // middle, behind, right -- 16
            // BELOW
            self.get_block_by_xyz(left, below, forward), // below, forward, left -- 17
            self.get_block_by_xyz(x_i32, below, forward), // below, forward, middle -- 18
            self.get_block_by_xyz(right, below, forward), // below, forward, right -- 19
            self.get_block_by_xyz(left, below, z_i32),   // below, left -- 20
            self.get_block_by_xyz(x_i32, below, z_i32),  // below -- 21
            self.get_block_by_xyz(right, below, z_i32),  // below, right -- 22
            self.get_block_by_xyz(left, below, behind),  // below, behind, left -- 23
            self.get_block_by_xyz(x_i32, below, behind), // below, behind, middle -- 24
            self.get_block_by_xyz(right, below, behind), // below, behind, right -- 25
        ];
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
