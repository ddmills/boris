use bitflags::bitflags;
use std::fmt::{Display, Formatter};

use bevy::{
    asset::Handle,
    ecs::system::Resource,
    math::Quat,
    render::{mesh::Mesh, texture::Image},
    scene::Scene,
    utils::HashMap,
};

use crate::colonists::{ItemTag, NavigationFlags};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TileRequirement: u16 {
        const IS_WALKABLE = 1;
        const IS_EMPTY = 2;
        const IS_ATTACHABLE = 4;
    }
}

impl Display for TileRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionSimple {
    North,
    East,
    South,
    West,
}

impl DirectionSimple {
    pub fn as_quat(&self) -> Quat {
        match self {
            Self::North => Quat::from_rotation_y(0.),
            Self::East => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
            Self::South => Quat::from_rotation_y(std::f32::consts::PI),
            Self::West => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlueprintHotspot {
    pub is_optional: bool,
    pub direction: DirectionSimple,
    pub nav_flag_requirements: NavigationFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlueprintTile {
    pub requirements: TileRequirement,
    pub nav_flags: NavigationFlags,
    pub is_blocker: bool,
    pub is_occupied: bool,
    pub hotspot: Option<BlueprintHotspot>,
    pub position: [i32; 3],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum BlueprintType {
    Workbench,
    Ladder,
    TorchStanding,
    TorchWall,
    Door,
}

#[derive(Clone)]
pub struct BuildSlot {
    pub flags: Vec<ItemTag>,
}

#[derive(Clone)]
pub struct BuildSlots {
    pub slot_0: Option<BuildSlot>,
    pub slot_1: Option<BuildSlot>,
    pub slot_2: Option<BuildSlot>,
}

pub struct Blueprint {
    pub name: String,
    pub center: [u32; 3],
    pub tiles: Vec<BlueprintTile>,
    pub texture: Option<Handle<Image>>,
    pub mesh: Handle<Mesh>,
    pub slots: BuildSlots,
}

#[derive(Resource, Default)]
pub struct Blueprints(pub HashMap<BlueprintType, Blueprint>);
