use std::fmt::{Display, Formatter};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PartitionFlags: u8 {
        const NONE = 0;
        const SOLID_GROUND = 1;
        const LADDER = 2;
        const TALL = 4;
    }
}

impl PartitionFlags {
    pub fn has(&self, flag: PartitionFlags) -> bool {
        (flag & *self) != PartitionFlags::NONE
    }
}

impl Display for PartitionFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}
