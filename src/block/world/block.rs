#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Block(pub u8);

impl Block {
    pub const OOB: Self = Self(0);
    pub const EMPTY: Self = Self(1);
    // pub const DIRT: Self = Self(2);
    pub const STONE: Self = Self(3);
}

impl Block {
    pub fn is_filled(&self) -> bool {
        match self {
            &Self::OOB => false,
            &Self::EMPTY => false,
            &Self::STONE => true,
            _ => false,
        }
    }

    pub fn texture_idx(&self) -> u32 {
        match self {
            &Self::STONE => 3,
            _ => 0,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::EMPTY
    }
}
