#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Block(pub u8);

impl Block {
    pub const OOB: Self = Self(0);
    pub const EMPTY: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const STONE: Self = Self(3);
    pub const GRASS: Self = Self(4);
}

impl Block {
    pub fn is_filled(&self) -> bool {
        match self {
            &Self::OOB => false,
            &Self::EMPTY => false,
            &Self::STONE => true,
            &Self::DIRT => true,
            &Self::GRASS => true,
            _ => false,
        }
    }

    pub fn texture_idx(&self) -> u32 {
        match self {
            &Self::DIRT => 1,
            &Self::GRASS => 2,
            &Self::STONE => 3,
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
            _ => String::from("unknown"),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::EMPTY
    }
}
