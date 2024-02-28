#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct BlockDetail {
    pub block: Block,
    pub light: u8,
    pub sunlight: u8,
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Block(pub u8);

impl Block {
    pub const OOB: Self = Self(0);
    pub const EMPTY: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const STONE: Self = Self(3);
    pub const GRASS: Self = Self(4);
    pub const LAMP: Self = Self(5);
    pub const LAVA: Self = Self(6);
}

impl Block {
    pub fn is_filled(&self) -> bool {
        match self {
            &Self::OOB => false,
            &Self::EMPTY => false,
            &Self::STONE => true,
            &Self::DIRT => true,
            &Self::GRASS => true,
            &Self::LAMP => true,
            &Self::LAVA => true,
            _ => false,
        }
    }

    pub fn is_opaque(&self) -> bool {
        match self {
            &Self::OOB => true,
            &Self::EMPTY => false,
            &Self::STONE => true,
            &Self::DIRT => true,
            &Self::GRASS => true,
            &Self::LAMP => true,
            &Self::LAVA => true,
            _ => true,
        }
    }

    pub fn is_light_source(&self) -> bool {
        match self {
            &Self::LAMP => true,
            &Self::LAVA => true,
            _ => false,
        }
    }

    pub fn texture_idx(&self) -> u32 {
        match self {
            &Self::DIRT => 1,
            &Self::GRASS => 2,
            &Self::STONE => 3,
            &Self::LAMP => 5,
            &Self::LAVA => 6,
            _ => 0,
        }
    }

    pub fn name(&self) -> String {
        match self {
            &Self::OOB => String::from("out of bounds"),
            &Self::EMPTY => String::from("empty"),
            &Self::DIRT => String::from("dirt"),
            &Self::GRASS => String::from("grass"),
            &Self::STONE => String::from("stone"),
            &Self::LAMP => String::from("lamp"),
            &Self::LAVA => String::from("lava"),
            _ => String::from("unknown"),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::EMPTY
    }
}
