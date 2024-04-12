use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    math::{vec3, Vec3},
    time::Time,
    transform::components::Transform,
};

use crate::Terrain;

use super::{InInventory, InPartition, NavigationGraph};

#[derive(Event)]
pub struct MovedEvent {
    pub entity: Entity,
    pub position: [u32; 3],
}

pub fn update_item_partition(
    mut graph: ResMut<NavigationGraph>,
    terrain: Res<Terrain>,
    mut cmd: Commands,
    mut ev_moved: EventReader<MovedEvent>,
    q_in_inventory: Query<&InInventory>,
    q_in_partition: Query<&InPartition>,
) {
    for ev in ev_moved.read() {
        let mut ecmd = cmd.entity(ev.entity);

        // remove the item from whatever partition it is in
        if let Ok(in_partition) = q_in_partition.get(ev.entity) {
            let partition_id = in_partition.partition_id;
            if let Some(partition) = graph.get_partition_mut(&partition_id) {
                partition.items.remove(&ev.entity);
            }
            ecmd.remove::<InPartition>();
        };

        if q_in_inventory.contains(ev.entity) {
            continue;
        }

        let [x, y, z] = ev.position;
        let Some(new_partition_id) = terrain.get_partition_id_u32(x, y, z) else {
            // not in a partition
            continue;
        };
        let Some(new_partition) = graph.get_partition_mut(new_partition_id) else {
            continue;
        };

        new_partition.items.insert(ev.entity);
        ecmd.insert(InPartition {
            partition_id: *new_partition_id,
        });
    }
}

#[derive(Component)]
pub struct BlockMove {
    pub speed: f32,
    pub target: [i32; 3],
    pub look_at: bool,
}

pub fn block_move_system(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &BlockMove, &mut Transform)>,
    mut ev_moved: EventWriter<MovedEvent>,
) {
    for (entity, block_move, mut transform) in query.iter_mut() {
        let pos = vec3(
            block_move.target[0] as f32 + 0.5,
            block_move.target[1] as f32,
            block_move.target[2] as f32 + 0.5,
        );

        let direction = (pos - transform.translation).normalize();
        let distance = transform.translation.distance(pos);
        let move_dist = time.delta_seconds() * block_move.speed;

        if distance < move_dist {
            transform.translation = pos;
            cmd.entity(entity).remove::<BlockMove>();
            ev_moved.send(MovedEvent {
                entity,
                position: [
                    block_move.target[0] as u32,
                    block_move.target[1] as u32,
                    block_move.target[2] as u32,
                ],
            });
        } else {
            transform.translation += direction * move_dist;
            if block_move.look_at {
                let target_rot = transform.looking_at(pos, Vec3::Y).rotation;
                transform.rotation = transform
                    .rotation
                    .slerp(target_rot, time.delta_seconds() * 20.);
            }
        }
    }
}
