#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Block {
    pub block: BlockType,
    pub light: u8,
    pub sunlight: u8,
    pub partition_id: Option<u32>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            block: BlockType::EMPTY,
            light: 0,
            sunlight: 0,
            partition_id: None,
        }
    }
}

impl Block {
    pub const OOB: Self = Self {
        block: BlockType::OOB,
        light: 0,
        sunlight: 0,
        partition_id: None,
    };
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct BlockType(pub u8);

impl BlockType {
    pub const OOB: Self = Self(0);
    pub const EMPTY: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const STONE: Self = Self(3);
    pub const GRASS: Self = Self(4);
    pub const LAMP: Self = Self(5);
    pub const MAGMA: Self = Self(6);
    pub const ASHLAR_LARGE: Self = Self(7);
    pub const ASHLAR: Self = Self(8);
    pub const LADDER: Self = Self(9);
    pub const BLUEPRINT: Self = Self(10);
}

impl BlockType {
    pub fn is_oob(&self) -> bool {
        *self == Self::OOB
    }

    pub fn is_rendered(&self) -> bool {
        !matches!(*self, Self::OOB | Self::EMPTY)
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(
            *self,
            Self::OOB | Self::EMPTY | Self::LADDER | Self::MAGMA | Self::BLUEPRINT
        )
    }

    pub fn is_empty(&self) -> bool {
        matches!(*self, Self::EMPTY | Self::BLUEPRINT)
    }

    pub fn is_opaque(&self) -> bool {
        match *self {
            Self::OOB => true,
            Self::EMPTY => false,
            _ => true,
        }
    }

    pub fn get_light_level(&self) -> u8 {
        match *self {
            Self::LAMP => 12,
            Self::MAGMA => 6,
            _ => 0,
        }
    }

    pub fn is_light(&self) -> bool {
        self.get_light_level() > 0
    }

    pub fn texture_idx(&self) -> u32 {
        match *self {
            Self::DIRT => 1,
            Self::GRASS => 2,
            Self::STONE => 3,
            Self::ASHLAR_LARGE => 4,
            Self::ASHLAR => 5,
            Self::MAGMA => 6,
            Self::LADDER => 7,
            Self::LAMP => 8,
            Self::BLUEPRINT => 16,
            _ => 0,
        }
    }

    pub fn name(&self) -> String {
        match *self {
            Self::OOB => String::from("out of bounds"),
            Self::EMPTY => String::from("empty"),
            Self::DIRT => String::from("dirt"),
            Self::GRASS => String::from("grass"),
            Self::STONE => String::from("stone"),
            Self::LAMP => String::from("lamp"),
            Self::MAGMA => String::from("magma"),
            Self::ASHLAR_LARGE => String::from("ashlar (large)"),
            Self::ASHLAR => String::from("ashlar"),
            Self::LADDER => String::from("ladder"),
            Self::BLUEPRINT => String::from("blueprint"),
            _ => String::from("unknown"),
        }
    }
}

impl Default for BlockType {
    fn default() -> Self {
        Self::EMPTY
    }
}
