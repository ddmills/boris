use std::vec;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    math::{Quat, Vec3},
    transform::components::Transform,
};

use crate::{colonists::get_block_flags, Terrain};

use super::{BlueprintGuide, TemplateTile, TemplateTileRequirement, TemplateType, Templates};

#[derive(Component)]
pub struct Blueprint {
    pub is_valid: bool,
    pub is_placed: bool,
    pub is_dirty: bool,
    pub is_hotspots_valid: bool,
    pub template_type: TemplateType,
    pub guides: Vec<Entity>,
    pub tiles: Vec<[i32; 3]>,
    pub position: [u32; 3],
    pub rotation: u8,
    pub is_flipped: bool,
}

#[derive(Event)]
pub struct RemoveBlueprintEvent {
    pub entity: Entity,
}

pub fn on_remove_blueprint(
    mut cmd: Commands,
    mut terrain: ResMut<Terrain>,
    q_blueprints: Query<&Blueprint>,
    mut ev_remove_blueprint: EventReader<RemoveBlueprintEvent>,
) {
    for ev in ev_remove_blueprint.read() {
        cmd.entity(ev.entity).despawn_recursive();

        let Ok(blueprint) = q_blueprints.get(ev.entity) else {
            continue;
        };

        for [x, y, z] in blueprint.tiles.iter() {
            let [chunk_idx, block_idx] = terrain.get_block_indexes(*x as u32, *y as u32, *z as u32);
            terrain.remove_blueprint(chunk_idx, block_idx, &ev.entity);
        }
    }
}

pub fn check_blueprints(
    mut terrain: ResMut<Terrain>,
    mut q_blueprints: Query<(Entity, &mut Blueprint, &mut Transform)>,
    mut q_guides: Query<&mut BlueprintGuide>,
    templates: Res<Templates>,
) {
    for (entity, mut blueprint, mut transform) in q_blueprints.iter_mut() {
        let Some(template) = templates.templates.get(&blueprint.template_type) else {
            println!("Missing template type!");
            continue;
        };

        transform.translation.x = blueprint.position[0] as f32 + 0.5;
        transform.translation.y = blueprint.position[1] as f32;
        transform.translation.z = blueprint.position[2] as f32 + 0.5;

        if blueprint.is_flipped {
            transform.scale = Vec3::new(1., 1., 1.);
            transform.rotation = match blueprint.rotation {
                1 => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
                2 => Quat::from_rotation_y(std::f32::consts::PI),
                3 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                _ => Quat::from_rotation_y(0.),
            };
        } else {
            transform.scale = Vec3::new(1., 1., -1.);
            transform.rotation = match blueprint.rotation {
                1 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                2 => Quat::from_rotation_y(std::f32::consts::PI),
                3 => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
                _ => Quat::from_rotation_y(0.),
            };
        }

        let center = [
            template.center[0] as i32,
            template.center[1] as i32,
            template.center[2] as i32,
        ];

        blueprint.is_valid = true;
        blueprint.is_hotspots_valid = true;

        // remove current tiles affecting terrain
        // if blueprint.is_placed {
        for [x, y, z] in blueprint.tiles.iter() {
            let [chunk_idx, block_idx] = terrain.get_block_indexes(*x as u32, *y as u32, *z as u32);
            terrain.remove_blueprint(chunk_idx, block_idx, &entity);
        }
        // }

        blueprint.tiles = template
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| {
                let tr = apply_transforms(
                    center,
                    tile.position,
                    blueprint.rotation,
                    blueprint.is_flipped,
                );
                let pos_i32 = [
                    blueprint.position[0] as i32 + tr[0],
                    blueprint.position[1] as i32 + tr[1],
                    blueprint.position[2] as i32 + tr[2],
                ];

                let tile_is_valid = is_blueprint_tile_valid(tile, pos_i32, &terrain);
                let mut guide_is_valid = true;

                if !tile_is_valid {
                    blueprint.is_valid = false;
                }

                if let Some(hotspot) = tile.hotspot {
                    let block_flags = get_block_flags(&terrain, pos_i32[0], pos_i32[1], pos_i32[2]);

                    guide_is_valid = block_flags.intersects(hotspot.nav_flag_requirements);

                    let [chunk_idx, block_idx] = terrain.get_block_indexes(
                        pos_i32[0] as u32,
                        pos_i32[1] as u32,
                        pos_i32[2] as u32,
                    );

                    let blueprints = terrain.get_blueprints(chunk_idx, block_idx);

                    if !blueprints.is_empty() {
                        guide_is_valid = false;
                    }

                    if !hotspot.is_optional && !guide_is_valid {
                        blueprint.is_hotspots_valid = false;
                    }
                }

                blueprint.guides.iter().enumerate().for_each(|(_, h)| {
                    let Ok(mut guide) = q_guides.get_mut(*h) else {
                        return;
                    };

                    if guide.tile_idx != idx {
                        return;
                    }

                    guide.is_valid = guide_is_valid;
                });

                if tile.is_blocker {
                    Some(pos_i32)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // set tiles affecting terrain
        // if blueprint.is_placed {
        for [x, y, z] in blueprint.tiles.iter() {
            let [chunk_idx, block_idx] = terrain.get_block_indexes(*x as u32, *y as u32, *z as u32);
            terrain.add_blueprint(chunk_idx, block_idx, entity);
        }
        // }
    }
}

fn apply_transforms(center: [i32; 3], point: [i32; 3], r: u8, f: bool) -> [i32; 3] {
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    let z = point[2] - center[2];

    let [rx, ry, rz] = match r {
        1 => [z, y, -x],
        2 => [-x, y, -z],
        3 => [-z, y, x],
        _ => [x, y, z],
    };

    match f {
        true => [rx, ry, -rz],
        false => [rx, ry, rz],
    }
}

fn is_blueprint_tile_valid(tile: &TemplateTile, pos: [i32; 3], terrain: &Terrain) -> bool {
    let block = terrain.get_block_i32(pos[0], pos[1], pos[2]);

    if block.is_oob() {
        return false;
    }

    let [chunk_idx, block_idx] =
        terrain.get_block_indexes(pos[0] as u32, pos[1] as u32, pos[2] as u32);

    tile.requirements.iter().all(|flag| match flag {
        TemplateTileRequirement::IS_EMPTY => {
            if !block.is_empty() {
                return false;
            }

            let blueprints = terrain.get_blueprints(chunk_idx, block_idx);
            blueprints.is_empty()
        }
        TemplateTileRequirement::IS_ATTACHABLE => block.is_attachable(),
        TemplateTileRequirement::IS_WALKABLE => {
            let below = terrain.get_block_i32(pos[0], pos[1] - 1, pos[2]);

            below.is_walkable()
        }
        _ => true,
    })
}
