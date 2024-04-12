use std::fmt::{Display, Formatter};

use bevy::ecs::component::Component;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
    pub struct NavigationFlags: u8 {
        const NONE = 0;
        const SOLID_GROUND = 1;
        const LADDER = 2;
        const TALL = 4;
        const COLONIST = Self::TALL.bits() | Self::LADDER.bits();
        const CAT = Self::SOLID_GROUND.bits();
    }
}

impl Display for NavigationFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}
