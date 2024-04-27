use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity},
    render::mesh::Mesh,
    utils::hashbrown::{HashMap, HashSet},
};
use ndshape::{AbstractShape, RuntimeShape};

use crate::{Block, BlockType};

#[derive(Component)]
pub struct ChunkMesh {
    pub chunk_idx: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
    pub mesh_handle: Handle<Mesh>,
}

#[derive(Clone)]
pub struct Chunk {
    pub shape: RuntimeShape<u32, 3>,
    pub blocks: Box<[Block]>,
    pub items: Box<[HashSet<Entity>]>,
    pub trees: Box<[HashSet<Entity>]>,
    // pub furniture: Box<[HashMap<Entity, TemplateTileType>]>,
    pub block_count: u32,
    pub chunk_idx: u32,
    pub chunk_size: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
    pub is_mesh_dirty: bool,
    pub is_nav_dirty: bool,
}

impl Chunk {
    pub fn new(shape: RuntimeShape<u32, 3>) -> Self {
        Self {
            blocks: vec![Block::default(); shape.size() as usize].into_boxed_slice(),
            items: vec![HashSet::new(); shape.size() as usize].into_boxed_slice(),
            trees: vec![HashSet::new(); shape.size() as usize].into_boxed_slice(),
            // furniture: vec![HashMap::new(); shape.size() as usize].into_boxed_slice(),
            block_count: shape.size(),
            shape,
            chunk_idx: 0,
            chunk_size: 0,
            world_x: 0,
            world_y: 0,
            world_z: 0,
            is_mesh_dirty: true,
            is_nav_dirty: true,
        }
    }

    pub fn set_block_type(&mut self, block_idx: u32, value: BlockType) {
        self.blocks[block_idx as usize].block = value;
        self.is_mesh_dirty = true;
        self.is_nav_dirty = true;
    }

    pub fn get_block(&self, block_idx: u32) -> Block {
        if let Some(block) = self.blocks.get(block_idx as usize) {
            return *block;
        }

        Block::OOB
    }

    pub fn get_items(&self, block_idx: u32) -> HashSet<Entity> {
        if let Some(items) = self.items.get(block_idx as usize) {
            return items.clone();
        }

        HashSet::new()
    }

    pub fn add_item(&mut self, block_idx: u32, item: Entity) {
        if let Some(items) = self.items.get_mut(block_idx as usize) {
            items.insert(item);
        }
    }

    pub fn remove_item(&mut self, block_idx: u32, item: &Entity) -> bool {
        if let Some(items) = self.items.get_mut(block_idx as usize) {
            return items.remove(item);
        }

        false
    }

    pub fn get_trees(&self, block_idx: u32) -> HashSet<Entity> {
        if let Some(trees) = self.trees.get(block_idx as usize) {
            return trees.clone();
        }

        HashSet::new()
    }

    pub fn add_tree(&mut self, block_idx: u32, tree: Entity) {
        if let Some(trees) = self.trees.get_mut(block_idx as usize) {
            trees.insert(tree);
        }
    }

    pub fn remove_tree(&mut self, block_idx: u32, tree: &Entity) -> bool {
        if let Some(trees) = self.trees.get_mut(block_idx as usize) {
            return trees.remove(tree);
        }

        false
    }

    // pub fn get_furniture(&self, block_idx: u32) -> HashMap<Entity, TemplateTileType> {
    //     if let Some(furniture) = self.furniture.get(block_idx as usize) {
    //         return furniture.clone();
    //     }

    //     HashMap::new()
    // }

    // pub fn add_furniture(
    //     &mut self,
    //     block_idx: u32,
    //     furniture: Entity,
    //     tile_type: TemplateTileType,
    // ) {
    //     if let Some(furnitures) = self.furniture.get_mut(block_idx as usize) {
    //         self.is_nav_dirty = true;
    //         furnitures.insert(furniture, tile_type);
    //     }
    // }

    // pub fn remove_furniture(&mut self, block_idx: u32, furniture: &Entity) -> bool {
    //     if let Some(furnitures) = self.furniture.get_mut(block_idx as usize) {
    //         self.is_nav_dirty = true;
    //         return furnitures.remove(furniture).is_some();
    //     }

    //     false
    // }

    pub fn set_partition_id(&mut self, block_idx: u32, value: u32) {
        self.blocks[block_idx as usize].partition_id = Some(value);
    }

    pub fn unset_partition_id(&mut self, block_idx: u32) {
        self.blocks[block_idx as usize].partition_id = None;
    }

    pub fn get_partition_id(&self, block_idx: u32) -> Option<u32> {
        self.blocks
            .get(block_idx as usize)
            .and_then(|block| block.partition_id)
    }

    pub fn get_sunlight(&self, block_idx: u32) -> u8 {
        self.get_block(block_idx).sunlight
    }

    pub fn get_torchlight(&self, block_idx: u32) -> u8 {
        self.get_block(block_idx).light
    }

    pub fn set_flag_blueprint(&mut self, block_idx: u32, value: bool) -> bool {
        let block = self.blocks[block_idx as usize];
        let is_changed = block.flag_blueprint != value;
        self.blocks[block_idx as usize].flag_blueprint = value;
        if is_changed {
            self.is_mesh_dirty = true;
        }
        is_changed
    }

    pub fn set_flag_mine(&mut self, block_idx: u32, value: bool) -> bool {
        let block = self.blocks[block_idx as usize];
        let is_changed = block.flag_mine != value;
        self.blocks[block_idx as usize].flag_mine = value;
        if is_changed {
            self.is_mesh_dirty = true;
        }
        is_changed
    }

    pub fn set_flag_chop(&mut self, block_idx: u32, value: bool) -> bool {
        let block = self.blocks[block_idx as usize];
        let is_changed = block.flag_chop != value;
        self.blocks[block_idx as usize].flag_chop = value;
        if is_changed {
            self.is_mesh_dirty = true;
        }
        is_changed
    }

    #[inline]
    pub fn set_sunlight(&mut self, block_idx: u32, value: u8) {
        self.blocks[block_idx as usize].sunlight = value;
        self.is_mesh_dirty = true;
    }

    #[inline]
    pub fn set_torchlight(&mut self, block_idx: u32, value: u8) {
        self.blocks[block_idx as usize].light = value;
        self.is_mesh_dirty = true;
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
        self.0 as usize
    }
}
