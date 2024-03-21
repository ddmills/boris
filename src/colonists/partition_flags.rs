use std::fmt::{Display, Formatter};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct PartitionFlags: u8 {
        const NONE = 0;
        const SOLID_GROUND = 1;
        const LADDER = 2;
        const TALL = 4;
    }
}

impl Display for PartitionFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}
