#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BlockFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl BlockFace {
    pub fn bit(&self) -> u32 {
        match self {
            BlockFace::PosX => 0,
            BlockFace::NegX => 1,
            BlockFace::PosY => 2,
            BlockFace::NegY => 3,
            BlockFace::PosZ => 4,
            BlockFace::NegZ => 5,
        }
    }

    pub fn offset(&self) -> [i32; 3] {
        match self {
            BlockFace::PosX => [1, 0, 0],
            BlockFace::NegX => [-1, 0, 0],
            BlockFace::PosY => [0, 1, 0],
            BlockFace::NegY => [0, -1, 0],
            BlockFace::PosZ => [0, 0, 1],
            BlockFace::NegZ => [0, 0, -1],
        }
    }
}
