use bevy::{
    ecs::system::{Res, Resource},
    gizmos::gizmos::Gizmos,
    math::Vec3,
    render::color::Color,
};

use crate::Terrain;

use super::{NavigationGraph, Partition};

#[derive(Resource, Default)]
pub struct PartitionDebug {
    pub id: u32,
    pub show: bool,
}

pub fn partition_debug(
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    debug: Res<PartitionDebug>,
    mut gizmos: Gizmos,
) {
    if !debug.show {
        return;
    }

    let Some(partition) = graph.get_partition(&debug.id) else {
        panic!("BAD DEBUG PARTITION ID");
    };

    debug_partition(
        partition,
        &terrain,
        &mut gizmos,
        Color::OLIVE,
        Color::ORANGE,
    );

    for neighbor_id in partition.neighbor_ids.iter() {
        let Some(neighbor) = graph.get_partition(neighbor_id) else {
            panic!("BAD NEIGHBOR ID");
        };

        debug_partition(neighbor, &terrain, &mut gizmos, Color::GRAY, Color::GRAY);
    }
}

fn debug_partition(
    partition: &Partition,
    terrain: &Res<Terrain>,
    gizmos: &mut Gizmos,
    color: Color,
    color_extents: Color,
) {
    for block_idx in partition.blocks.iter() {
        let [x, y, z] = terrain.get_block_world_pos(partition.chunk_idx, *block_idx);
        let pos = Vec3::new(x as f32, y as f32 + 0.02, z as f32);

        gizmos.line(pos, pos + Vec3::new(1., 0., 0.), color);
        gizmos.line(pos, pos + Vec3::new(0., 0., 1.), color);

        gizmos.line(pos, pos + Vec3::new(1., 0., 0.), color);
        gizmos.line(pos, pos + Vec3::new(0., 0., 1.), color);

        gizmos.line(
            pos + Vec3::new(1., 0., 1.),
            pos + Vec3::new(1., 0., 0.),
            color,
        );
        gizmos.line(
            pos + Vec3::new(1., 0., 1.),
            pos + Vec3::new(0., 0., 1.),
            color,
        );

        let extents = &partition.extents;

        let ex_min = Vec3::new(
            extents.min_x as f32,
            extents.min_y as f32,
            extents.min_z as f32,
        );
        let ex_max = Vec3::new(
            extents.max_x as f32 + 1.,
            extents.max_y as f32 + 1.,
            extents.max_z as f32 + 1.,
        );

        gizmos.line(
            ex_min,
            Vec3::new(ex_max.x, ex_min.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            ex_min,
            Vec3::new(ex_min.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            ex_min,
            Vec3::new(ex_min.x, ex_min.y, ex_max.z),
            color_extents,
        );

        gizmos.line(
            ex_max,
            Vec3::new(ex_min.x, ex_max.y, ex_max.z),
            color_extents,
        );
        gizmos.line(
            ex_max,
            Vec3::new(ex_max.x, ex_min.y, ex_max.z),
            color_extents,
        );
        gizmos.line(
            ex_max,
            Vec3::new(ex_max.x, ex_max.y, ex_min.z),
            color_extents,
        );

        gizmos.line(
            Vec3::new(ex_max.x, ex_min.y, ex_min.z),
            Vec3::new(ex_max.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            Vec3::new(ex_min.x, ex_max.y, ex_max.z),
            Vec3::new(ex_min.x, ex_min.y, ex_max.z),
            color_extents,
        );

        gizmos.line(
            Vec3::new(ex_min.x, ex_max.y, ex_min.z),
            Vec3::new(ex_max.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            Vec3::new(ex_min.x, ex_min.y, ex_max.z),
            Vec3::new(ex_max.x, ex_min.y, ex_max.z),
            color_extents,
        );

        gizmos.line(
            Vec3::new(ex_min.x, ex_max.y, ex_max.z),
            Vec3::new(ex_min.x, ex_max.y, ex_min.z),
            color_extents,
        );
        gizmos.line(
            Vec3::new(ex_max.x, ex_min.y, ex_min.z),
            Vec3::new(ex_max.x, ex_min.y, ex_max.z),
            color_extents,
        );
    }
}
