use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity},
    render::mesh::Mesh,
    utils::hashbrown::{HashMap, HashSet},
};
use ndshape::{AbstractShape, RuntimeShape};

use crate::{colonists::NavigationFlags, Block, BlockType};

#[derive(Component)]
pub struct ChunkMesh {
    pub chunk_idx: u32,
    pub mesh_handle: Handle<Mesh>,
}

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Box<[Block]>,
    pub items: Box<[HashSet<Entity>]>,
    pub trees: Box<[HashSet<Entity>]>,
    pub structures: Box<[HashMap<Entity, StructureTileDetail>]>,
    pub lamps: Box<[HashMap<Entity, LampDetail>]>,
    pub chunk_idx: u32,
    pub chunk_size: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
    pub is_mesh_dirty: bool,
    pub is_nav_dirty: bool,
}

#[derive(Clone, Copy)]
pub struct LampDetail {
    pub torchlight: u8,
}

#[derive(Clone, Copy)]
pub struct StructureTileDetail {
    pub flags: Option<NavigationFlags>,
    pub is_built: bool,
    pub is_blocker: bool,
    pub is_occupied: bool,
}

impl Chunk {
    pub fn new(shape: RuntimeShape<u32, 3>) -> Self {
        Self {
            blocks: vec![Block::default(); shape.size() as usize].into_boxed_slice(),
            items: vec![HashSet::new(); shape.size() as usize].into_boxed_slice(),
            trees: vec![HashSet::new(); shape.size() as usize].into_boxed_slice(),
            structures: vec![HashMap::new(); shape.size() as usize].into_boxed_slice(),
            lamps: vec![HashMap::new(); shape.size() as usize].into_boxed_slice(),
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

    pub fn get_structures(&self, block_idx: u32) -> HashMap<Entity, StructureTileDetail> {
        if let Some(structures) = self.structures.get(block_idx as usize) {
            return structures.clone();
        }

        HashMap::new()
    }

    pub fn add_structure(
        &mut self,
        block_idx: u32,
        structure: Entity,
        detail: StructureTileDetail,
    ) {
        if let Some(structures) = self.structures.get_mut(block_idx as usize) {
            structures.insert(structure, detail);
        }
    }

    pub fn remove_structure(
        &mut self,
        block_idx: u32,
        structure: &Entity,
    ) -> Option<StructureTileDetail> {
        if let Some(structures) = self.structures.get_mut(block_idx as usize) {
            return structures.remove(structure);
        }

        None
    }

    pub fn get_lamps(&self, block_idx: u32) -> HashMap<Entity, LampDetail> {
        if let Some(lamps) = self.lamps.get(block_idx as usize) {
            return lamps.clone();
        }

        HashMap::new()
    }

    pub fn add_lamp(&mut self, block_idx: u32, lamp: Entity, detail: LampDetail) {
        if let Some(lamps) = self.lamps.get_mut(block_idx as usize) {
            lamps.insert(lamp, detail);
        }
    }

    pub fn remove_lamp(&mut self, block_idx: u32, lamp: &Entity) -> Option<LampDetail> {
        if let Some(lamps) = self.lamps.get_mut(block_idx as usize) {
            return lamps.remove(lamp);
        }

        None
    }

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
