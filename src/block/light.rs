use bevy::ecs::system::ResMut;

use super::world::terrain::Terrain;

pub struct LightNode {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub value: u8,
}

pub fn light_system(mut terrain: ResMut<Terrain>) {
    let max_sunlight_passes = 1000;
    let mut sunlight_passes = 0;

    while !terrain.lights_queue_remove.is_empty() {
        let node = terrain.lights_queue_remove.remove(0);

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

            if n_detail.block.is_oob() {
                continue;
            }

            if n_detail.light != 0 && n_detail.light < node.value {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                terrain.set_torchlight(n_x_u32, n_y_u32, n_z_u32, 0);

                terrain.lights_queue_remove.push(LightNode {
                    x: n_x_u32,
                    y: n_y_u32,
                    z: n_z_u32,
                    value: n_detail.light,
                });
            } else if n_detail.light >= node.value {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                terrain.lights_queue_add.push(LightNode {
                    x: n_x_u32,
                    y: n_y_u32,
                    z: n_z_u32,
                    value: n_detail.light,
                });
            }
        }
    }

    while !terrain.lights_queue_add.is_empty() {
        let node = terrain.lights_queue_add.remove(0);

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

        let current_light = terrain.get_torchlight_xyz(node.x, node.y, node.z);

        for [n_x, n_y, n_z] in neighbors {
            let n_detail = terrain.get_block_detail_i32(n_x, n_y, n_z);

            if n_detail.block.is_opaque() {
                continue;
            }

            if (n_detail.light + 2) <= current_light {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                terrain.add_light(n_x_u32, n_y_u32, n_z_u32, current_light - 1);
            }
        }
    }

    while !terrain.sunlight_queue_remove.is_empty() {
        let node = terrain.sunlight_queue_remove.remove(0);

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

            if (n_detail.sunlight == 15 && n_y == world_y - 1)
                || (n_detail.sunlight != 0 && n_detail.sunlight < node.value)
            {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                terrain.set_sunlight(n_x_u32, n_y_u32, n_z_u32, 0);
                terrain.sunlight_queue_remove.push(LightNode {
                    x: n_x_u32,
                    y: n_y_u32,
                    z: n_z_u32,
                    value: n_detail.sunlight,
                });
            } else if n_detail.sunlight >= node.value {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                terrain.sunlight_queue_add.push(LightNode {
                    x: n_x_u32,
                    y: n_y_u32,
                    z: n_z_u32,
                    value: 0,
                });
            }
        }
    }

    while !terrain.sunlight_queue_add.is_empty() {
        sunlight_passes = sunlight_passes + 1;
        if sunlight_passes > max_sunlight_passes {
            break;
        }

        let node = terrain.sunlight_queue_add.remove(0);

        let block_detail = terrain.get_block_detail(node.x, node.y, node.z);

        if block_detail.block.is_opaque() {
            continue;
        }

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

            if n_detail.block.is_opaque() {
                continue;
            }

            if n_detail.sunlight + 2 <= block_detail.sunlight
                || (block_detail.sunlight == 15 && n_detail.sunlight != 15 && n_y == world_y - 1)
            {
                let n_x_u32 = n_x as u32;
                let n_y_u32 = n_y as u32;
                let n_z_u32 = n_z as u32;

                if block_detail.sunlight == 15 && n_y == world_y - 1 {
                    terrain.add_sunlight(n_x_u32, n_y_u32, n_z_u32, block_detail.sunlight);
                } else if block_detail.sunlight == 15 && n_y == world_y + 1 {
                    continue;
                } else {
                    terrain.add_sunlight(n_x_u32, n_y_u32, n_z_u32, block_detail.sunlight - 1);
                }
            }
        }
    }
}
