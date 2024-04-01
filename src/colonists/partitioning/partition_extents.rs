use crate::common::{max_3, Distance};

#[derive(Default)]
pub struct PartitionExtents {
    is_init: bool,
    pub min_x: u32,
    pub min_y: u32,
    pub min_z: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub max_z: u32,
    pub traversal_distance: f32,
}

impl PartitionExtents {
    pub fn center(&self) -> [u32; 3] {
        [
            self.min_x + (self.max_x - self.min_x) / 2,
            self.min_y + (self.max_y - self.min_y) / 2,
            self.min_z + (self.max_z - self.min_z) / 2,
        ]
    }

    pub fn extend(&mut self, pos: [u32; 3]) {
        if !self.is_init {
            self.min_x = pos[0];
            self.min_y = pos[1];
            self.min_z = pos[2];
            self.max_x = pos[0];
            self.max_y = pos[1];
            self.max_z = pos[2];
            self.is_init = true;
            return;
        };

        self.min_x = pos[0].min(self.min_x);
        self.min_y = pos[1].min(self.min_y);
        self.min_z = pos[2].min(self.min_z);
        self.max_x = pos[0].max(self.max_x);
        self.max_y = pos[1].max(self.max_y);
        self.max_z = pos[2].max(self.max_z);
    }

    pub fn distance_to_edge(&self, x: i32, _y: i32, z: i32) -> f32 {
        // TODO: this only works in 2D space
        let dx = max_3(self.min_x as i32 - x, 0, x - self.max_x as i32).abs();
        let dz = max_3(self.min_z as i32 - z, 0, z - self.max_z as i32).abs();
        // let dz = max_3(self.min_z as i32 - z, 0, z - self.max_z as i32).abs();

        (dx + dz) as f32 - (0.59 * dx.min(dz) as f32)
    }

    pub fn update_traversal_distance(&mut self) {
        self.traversal_distance = Distance::diagonal(
            [self.min_x as i32, self.min_y as i32, self.min_z as i32],
            [self.max_x as i32, self.max_y as i32, self.max_z as i32],
        );
    }
}
