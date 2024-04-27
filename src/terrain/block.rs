#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Block {
    pub block: BlockType,
    pub light: u8,
    pub sunlight: u8,
    pub partition_id: Option<u32>,
    pub flag_mine: bool,
    pub flag_chop: bool,
    pub flag_blueprint: bool,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            block: BlockType::EMPTY,
            light: 0,
            sunlight: 0,
            partition_id: None,
            flag_mine: false,
            flag_chop: false,
            flag_blueprint: false,
        }
    }
}

impl Block {
    pub const OOB: Self = Self {
        block: BlockType::OOB,
        light: 0,
        sunlight: 0,
        partition_id: None,
        flag_mine: false,
        flag_chop: false,
        flag_blueprint: false,
    };

    pub fn is_oob(&self) -> bool {
        self.block == BlockType::OOB
    }

    pub fn is_rendered(&self) -> bool {
        if self.flag_blueprint {
            return true;
        }

        !matches!(self.block, BlockType::OOB | BlockType::EMPTY)
    }

    pub fn is_walkable(&self) -> bool {
        if self.flag_blueprint {
            return false;
        }

        !matches!(
            self.block,
            BlockType::OOB
                | BlockType::EMPTY
                | BlockType::LADDER
                | BlockType::MAGMA
                | BlockType::LEAVES
        )
    }

    pub fn is_attachable(&self) -> bool {
        if self.flag_blueprint {
            return false;
        }

        !matches!(
            self.block,
            BlockType::OOB
                | BlockType::EMPTY
                | BlockType::LADDER
                | BlockType::MAGMA
                | BlockType::LEAVES
        )
    }

    pub fn is_empty(&self) -> bool {
        self.flag_blueprint || matches!(self.block, BlockType::EMPTY)
    }

    pub fn is_opaque(&self) -> bool {
        match self.block {
            BlockType::OOB => true,
            BlockType::EMPTY => false,
            _ => true,
        }
    }

    pub fn get_light_level(&self) -> u8 {
        match self.block {
            BlockType::LAMP => 12,
            BlockType::MAGMA => 6,
            _ => 0,
        }
    }

    pub fn is_light(&self) -> bool {
        self.get_light_level() > 0
    }

    pub fn is_mineable(&self) -> bool {
        if self.flag_blueprint {
            return false;
        }

        matches!(
            self.block,
            BlockType::DIRT
                | BlockType::GRASS
                | BlockType::STONE
                | BlockType::ASHLAR
                | BlockType::ASHLAR_LARGE
        )
    }

    pub fn texture_idx(&self) -> u32 {
        match self.block {
            BlockType::DIRT => 1,
            BlockType::GRASS => 2,
            BlockType::STONE => 3,
            BlockType::ASHLAR_LARGE => 4,
            BlockType::ASHLAR => 5,
            BlockType::MAGMA => 6,
            BlockType::LADDER => 7,
            BlockType::LAMP => 8,
            BlockType::LEAVES => 40,
            BlockType::TREE_TRUNK => 41,
            _ => 0,
        }
    }

    pub fn name(&self) -> String {
        match self.block {
            BlockType::OOB => String::from("out of bounds"),
            BlockType::EMPTY => String::from("empty"),
            BlockType::DIRT => String::from("dirt"),
            BlockType::GRASS => String::from("grass"),
            BlockType::STONE => String::from("stone"),
            BlockType::LAMP => String::from("lamp"),
            BlockType::MAGMA => String::from("magma"),
            BlockType::ASHLAR_LARGE => String::from("ashlar (large)"),
            BlockType::ASHLAR => String::from("ashlar"),
            BlockType::LADDER => String::from("ladder"),
            _ => String::from("unknown"),
        }
    }
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
    pub const TREE_TRUNK: Self = Self(10);
    pub const LEAVES: Self = Self(11);
}

impl BlockType {
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
            Self::TREE_TRUNK => String::from("trunk"),
            Self::LEAVES => String::from("leaves"),
            _ => String::from("unknown"),
        }
    }
}

impl Default for BlockType {
    fn default() -> Self {
        Self::EMPTY
    }
}
