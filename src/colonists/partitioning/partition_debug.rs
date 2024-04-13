use bevy::{
    ecs::system::{Res, ResMut, Resource},
    gizmos::gizmos::Gizmos,
    math::Vec3,
    render::color::Color,
};

use crate::Terrain;

use super::{NavigationGraph, Partition};

#[derive(Resource, Default)]
pub struct PartitionDebug {
    pub partition_id: Option<u32>,
}

pub fn partition_debug(
    terrain: Res<Terrain>,
    graph: Res<NavigationGraph>,
    mut debug: ResMut<PartitionDebug>,
    mut gizmos: Gizmos,
) {
    let Some(debug_partition_id) = debug.partition_id else {
        return;
    };

    let Some(partition) = graph.get_partition(&debug_partition_id) else {
        debug.partition_id = None;
        return;
    };

    let Some(region) = graph.get_region(&partition.region_id) else {
        panic!("No region declared for partition! {}", partition.id);
    };

    for partition_id in region.partition_ids.iter() {
        if *partition_id == debug_partition_id {
            debug_partition(
                partition,
                &terrain,
                &mut gizmos,
                Color::OLIVE,
                Color::ORANGE,
            );
            continue;
        }

        let part = graph.get_partition(partition_id).unwrap();

        debug_partition(part, &terrain, &mut gizmos, Color::GRAY, Color::GRAY);
    }

    for neighbor_id in region.neighbor_ids.iter() {
        let neighbor = graph.get_region(neighbor_id).unwrap();

        for partition_id in neighbor.partition_ids.iter() {
            let part = graph.get_partition(partition_id).unwrap();
            debug_partition(part, &terrain, &mut gizmos, Color::BLUE, Color::BLUE);
        }
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
