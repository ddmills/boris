use std::cmp::Ordering;

use bevy::{
    ecs::system::{Query, Res},
    gizmos::gizmos::Gizmos,
    math::Vec3,
    render::color::Color,
};

use crate::colonists::Path;

use super::debug_settings::DebugSettings;

pub fn path_debug(settings: Res<DebugSettings>, mut gizmos: Gizmos, pathers: Query<&Path>) {
    if !settings.path {
        return;
    }

    for path in pathers.iter() {
        for i in 1..path.blocks.len() {
            let current = path.blocks[i - 1];
            let next = path.blocks[i];

            let mid = Vec3::new(0.5, 0.5, 0.5);

            let color = match i.cmp(&path.current_block_idx) {
                Ordering::Less => Color::ORANGE,
                Ordering::Equal => Color::ORANGE_RED,
                Ordering::Greater => Color::GRAY,
            };

            gizmos.line(
                Vec3::new(current[0] as f32, current[1] as f32, current[2] as f32) + mid,
                Vec3::new(next[0] as f32, next[1] as f32, next[2] as f32) + mid,
                color,
            );
        }

        for g in path.goals.iter() {
            let pos = Vec3::new(g[0] as f32, g[1] as f32 + 0.04, g[2] as f32);

            gizmos.line(pos, pos + Vec3::new(1., 0., 0.), Color::CYAN);
            gizmos.line(pos, pos + Vec3::new(0., 0., 1.), Color::CYAN);

            gizmos.line(pos, pos + Vec3::new(1., 0., 0.), Color::CYAN);
            gizmos.line(pos, pos + Vec3::new(0., 0., 1.), Color::CYAN);

            gizmos.line(
                pos + Vec3::new(1., 0., 1.),
                pos + Vec3::new(1., 0., 0.),
                Color::CYAN,
            );
            gizmos.line(
                pos + Vec3::new(1., 0., 1.),
                pos + Vec3::new(0., 0., 1.),
                Color::CYAN,
            );
        }
    }
}
