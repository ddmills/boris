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

use crate::{
    colonists::{get_block_flags, NavigationFlags},
    EmplacementTileDetail, Terrain,
};

use super::{
    BlueprintGuide, TemplateHotspot, TemplateTile, TemplateTileRequirement, TemplateType, Templates,
};

pub enum BlueprintMode {
    Placing,
    Placed,
    Built,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlueprintTile {
    pub requirements: TemplateTileRequirement,
    pub nav_flags: NavigationFlags,
    pub is_blocker: bool,
    pub is_occupied: bool,
    pub hotspot: Option<TemplateHotspot>,
    pub position: [i32; 3],
}

#[derive(Component)]
pub struct Blueprint {
    pub is_valid: bool,
    pub is_dirty: bool,
    pub mode: BlueprintMode,
    pub is_hotspots_valid: bool,
    pub template_type: TemplateType,
    pub guides: Vec<Entity>,
    pub tiles: Vec<BlueprintTile>,
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

        for tile in blueprint.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
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
        if !blueprint.is_dirty {
            continue;
        }

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
        blueprint.is_dirty = false;

        // remove current tiles affecting terrain
        for tile in blueprint.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
            terrain.remove_blueprint(chunk_idx, block_idx, &entity);
        }

        blueprint.tiles = template
            .tiles
            .iter()
            .enumerate()
            .map(|(idx, tile)| {
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

                let mut guide_is_valid = true;

                if let Some(hotspot) = tile.hotspot {
                    let block_flags = get_block_flags(&terrain, pos_i32[0], pos_i32[1], pos_i32[2]);

                    guide_is_valid = block_flags.contains(hotspot.nav_flag_requirements);

                    let [chunk_idx, block_idx] = terrain.get_block_indexes(
                        pos_i32[0] as u32,
                        pos_i32[1] as u32,
                        pos_i32[2] as u32,
                    );

                    let blueprints = terrain.get_blueprints(chunk_idx, block_idx);

                    if guide_is_valid {
                        if tile.is_blocker {
                            guide_is_valid = blueprints.is_empty();
                        } else {
                            guide_is_valid = !blueprints.values().any(|v| v.is_blocker)
                        }
                    }

                    if !hotspot.is_optional && !guide_is_valid {
                        blueprint.is_hotspots_valid = false;
                    }
                }

                let tile_is_valid = is_blueprint_tile_valid(tile, pos_i32, &terrain);
                if !tile_is_valid {
                    blueprint.is_valid = false;
                    guide_is_valid = false;
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

                BlueprintTile {
                    requirements: tile.requirements,
                    nav_flags: tile.nav_flags,
                    is_blocker: tile.is_blocker,
                    is_occupied: tile.is_occupied,
                    hotspot: tile.hotspot,
                    position: pos_i32,
                }
            })
            .collect::<Vec<_>>();

        // set tiles affecting terrain
        for tile in blueprint.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
            let flags = if tile.nav_flags == NavigationFlags::NONE {
                None
            } else {
                Some(tile.nav_flags)
            };

            terrain.add_blueprint(
                chunk_idx,
                block_idx,
                entity,
                EmplacementTileDetail {
                    is_built: matches!(blueprint.mode, BlueprintMode::Built),
                    flags,
                    is_blocker: tile.is_blocker,
                    is_occupied: tile.is_occupied,
                },
            );
        }
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

            if tile.is_blocker || tile.is_occupied {
                !blueprints.values().any(|v| v.is_blocker || v.is_occupied)
            } else {
                !blueprints.values().any(|v| v.is_blocker)
            }
        }
        TemplateTileRequirement::IS_ATTACHABLE => block.is_attachable(),
        TemplateTileRequirement::IS_WALKABLE => {
            let below = terrain.get_block_i32(pos[0], pos[1] - 1, pos[2]);

            below.is_walkable()
        }
        _ => true,
    })
}
