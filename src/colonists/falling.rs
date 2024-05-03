use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    transform::components::Transform,
};

use crate::{colonists::BlockMove, Terrain};

use super::{InInventory, InSlot, NavigationFlags, NavigationGraph};

#[derive(Component)]
pub struct Faller;

pub fn apply_falling(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    q_fallers: Query<
        (Entity, &Transform, Option<&NavigationFlags>),
        (
            With<Faller>,
            Without<BlockMove>,
            Without<InSlot>,
            Without<InInventory>,
        ),
    >,
) {
    for (entity, transform, opt_flags) in q_fallers.iter() {
        let x = transform.translation.x as u32;
        let y = transform.translation.y as u32;
        let z = transform.translation.z as u32;

        let [chunk_idx, block_idx] = terrain.get_block_indexes(x, y, z);

        if terrain.get_partition_id(chunk_idx, block_idx).is_some() {
            continue;
        }

        if terrain.get_is_chunk_nav_dirty(chunk_idx) {
            continue;
        }

        let mut ecmd = cmd.entity(entity);

        let world_y = terrain.world_size_y();

        let mut delta_y = 0;

        loop {
            delta_y += 1;

            if delta_y < y {
                let sub_y = y - delta_y;
                let mut flag_ok = true;

                if let Some(partition_id) = terrain.get_partition_id_u32(x, sub_y, z) {
                    if let Some(partition) = graph.get_partition(&partition_id) {
                        if let Some(flags) = opt_flags {
                            flag_ok = partition.flags.intersects(*flags);
                        }

                        if flag_ok {
                            ecmd.insert(BlockMove {
                                speed: 12.,
                                target: [x as i32, sub_y as i32, z as i32],
                                look_at: false,
                            });
                            break;
                        }
                    }
                }
            }

            if delta_y < world_y {
                let add_y = y + delta_y;
                let mut flag_ok = true;

                if let Some(partition_id) = terrain.get_partition_id_u32(x, add_y, z) {
                    if let Some(partition) = graph.get_partition(&partition_id) {
                        if let Some(flags) = opt_flags {
                            flag_ok = partition.flags.intersects(*flags);
                        }

                        if flag_ok {
                            ecmd.insert(BlockMove {
                                speed: 12.,
                                target: [x as i32, add_y as i32 + 1, z as i32],
                                look_at: false,
                            });
                            break;
                        }
                    }
                }
            } else {
                println!("no good spot to land {}", delta_y);
                break;
            }
        }
    }
}
