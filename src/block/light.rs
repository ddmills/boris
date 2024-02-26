use bevy::{
    ecs::{
        event::EventWriter,
        system::{ResMut, Resource},
    },
    render::render_resource::encase::rts_array::Length,
};

use super::world::{
    block::Block,
    terrain::{Terrain, TerrainModifiedEvent},
};

#[derive(Resource)]
pub struct Lights {
    queue: Vec<LightNode>,
}

impl Lights {
    pub fn new() -> Self {
        Self { queue: vec![] }
    }

    pub fn add_light(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.queue.push(LightNode { x, y, z, value });
    }
}

struct LightNode {
    x: u32,
    y: u32,
    z: u32,
    value: u8,
}

pub fn light_system(mut terrain: ResMut<Terrain>, mut lights: ResMut<Lights>) {
    let mut count = 0;

    while !lights.queue.is_empty() {
        let node = lights.queue.remove(0);
        let block = terrain.get_block_detail(node.x, node.y, node.z);

        if block.block.is_opaque() && !block.block.is_light_source() {
            continue;
        }

        if block.light >= node.value {
            continue;
        }

        terrain.set_torchlight(node.x, node.y, node.z, node.value);
        count = count + 1;

        let world_x = node.x as i32;
        let world_y = node.y as i32;
        let world_z = node.z as i32;

        let neighbors = [
            [world_x + 1, world_y, world_z],
            [world_x - 1, world_y, world_z],
            [world_x, world_y + 1, world_z],
            [world_x, world_y - 1, world_z],
            [world_x, world_y, world_z - 1],
            [world_x, world_y, world_z + 1],
        ];

        for [n_x, n_y, n_z] in neighbors {
            let n_detail = terrain.get_block_detail_i32(n_x, n_y, n_z);

            if (n_detail.light + 2) <= node.value && !n_detail.block.is_opaque() {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                lights.queue.push(LightNode {
                    x: n_x_u32,
                    y: n_y_u32,
                    z: n_z_u32,
                    value: node.value - 1,
                });
            }
        }
    }

    if (count > 0) {
        println!("checked {} blocks!", count);
    }
}
