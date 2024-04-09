use bevy::{
    ecs::{
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    transform::components::Transform,
};

use crate::{colonists::BlockMove, Terrain};

use super::{get_block_flags, Actor, NavigationFlags};

pub fn fix_colonist_positions(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    mut q_colonists: Query<(Entity, &mut Transform), (With<Actor>, Without<BlockMove>)>,
) {
    for (entity, mut transform) in q_colonists.iter_mut() {
        let x = transform.translation.x as u32;
        let y = transform.translation.y as u32;
        let z = transform.translation.z as u32;

        if terrain.get_partition_id_u32(x, y, z).is_some() {
            continue;
        }

        let world_y = terrain.world_size_y();

        let mut delta_y = 0;

        loop {
            delta_y += 1;

            if delta_y < y {
                let sub_y = y - delta_y;
                let below = get_block_flags(&terrain, x as i32, sub_y as i32, z as i32);
                println!("below {} {}", sub_y, below);
                if below.intersects(NavigationFlags::COLONIST) {
                    cmd.entity(entity).insert(BlockMove {
                        speed: 12.,
                        target: [x as i32, sub_y as i32, z as i32],
                    });
                    break;
                }
            }

            if delta_y < world_y {
                let add_y = y + delta_y;
                let above = get_block_flags(&terrain, x as i32, add_y as i32, z as i32);
                println!("above {} {}", add_y, above);
                if above.intersects(NavigationFlags::COLONIST) {
                    cmd.entity(entity).insert(BlockMove {
                        speed: 12.,
                        target: [x as i32, add_y as i32, z as i32],
                    });
                    break;
                }
            } else {
                println!("no good spot to land {}", delta_y);
                break;
            }
        }
    }
}
