use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::{Quat, Vec3},
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{color::Color, texture::Image, view::Visibility},
    transform::components::Transform,
};

use crate::{
    colonists::{get_block_flags, InSlot, ItemTag, JobBuild, JobCancelEvent, NavigationFlags},
    items::image_loader_settings,
    rendering::{BasicMaterial, SlotIndex},
    Position, StructureTileDetail, Terrain,
};

use super::{
    BlueprintHotspot, BlueprintTile, BlueprintType, Blueprints, BuildSlots, TileRequirement,
};

#[derive(Debug, Clone, Component)]
pub struct StructureGuide {
    pub tile_idx: usize,
    pub is_valid: bool,
    pub is_hotspot: bool,
}

pub enum StructureMode {
    Placing,
    Placed,
    Built,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureTile {
    pub requirements: TileRequirement,
    pub nav_flags: NavigationFlags,
    pub is_blocker: bool,
    pub is_occupied: bool,
    pub hotspot: Option<BlueprintHotspot>,
    pub position: [i32; 3],
}

#[derive(Clone)]
pub struct PartSlot {
    pub idx: SlotIndex,
    pub flags: Vec<ItemTag>,
    pub content: Option<Entity>,
}

impl PartSlot {
    pub fn is_empty(&self) -> bool {
        self.content.is_none()
    }
}

#[derive(Component)]
pub struct PartSlots {
    pub slot_0: Option<PartSlot>,
    pub slot_1: Option<PartSlot>,
    pub slot_2: Option<PartSlot>,
}

impl PartSlots {
    pub fn from_build_slots(build: &BuildSlots) -> Self {
        Self {
            slot_0: build.slot_0.as_ref().map(|x| PartSlot {
                idx: SlotIndex::Slot0,
                flags: x.flags.clone(),
                content: None,
            }),
            slot_1: build.slot_1.as_ref().map(|x| PartSlot {
                idx: SlotIndex::Slot1,
                flags: x.flags.clone(),
                content: None,
            }),
            slot_2: build.slot_2.as_ref().map(|x| PartSlot {
                idx: SlotIndex::Slot2,
                flags: x.flags.clone(),
                content: None,
            }),
        }
    }

    pub fn as_vec(&self) -> Vec<&PartSlot> {
        let mut res = vec![];

        if let Some(slot) = self.slot_0.as_ref() {
            res.push(slot);
        }
        if let Some(slot) = self.slot_1.as_ref() {
            res.push(slot);
        }
        if let Some(slot) = self.slot_2.as_ref() {
            res.push(slot);
        }

        res
    }

    pub fn get_mut(&mut self, idx: SlotIndex) -> Option<&mut PartSlot> {
        match idx {
            SlotIndex::Slot0 => self.slot_0.as_mut(),
            SlotIndex::Slot1 => self.slot_1.as_mut(),
            SlotIndex::Slot2 => self.slot_2.as_mut(),
        }
    }
}

#[derive(Component)]
pub struct Structure {
    pub is_valid: bool,
    pub is_dirty: bool,
    pub mode: StructureMode,
    pub is_hotspots_valid: bool,
    pub blueprint_type: BlueprintType,
    pub guides: Vec<Entity>,
    pub tiles: Vec<StructureTile>,
    pub position: [u32; 3],
    pub rotation: u8,
    pub is_flipped: bool,
}

impl Structure {
    pub fn is_built(&self) -> bool {
        matches!(self.mode, StructureMode::Built)
    }
}

#[derive(Event)]
pub struct RemoveStructureEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct SpawnStructureEvent {
    pub pos: [u32; 3],
    pub entity: Entity,
    pub blueprint_type: BlueprintType,
}

#[derive(Event)]
pub struct BuildStructureEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct BuiltStructureEvent {
    pub entity: Entity,
    pub blueprint_type: BlueprintType,
}

pub fn on_spawn_structure(
    mut cmd: Commands,
    mut ev_spawn_structure: EventReader<SpawnStructureEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
    blueprints: Res<Blueprints>,
) {
    for ev in ev_spawn_structure.read() {
        let Some(blueprint) = blueprints.0.get(&ev.blueprint_type) else {
            println!("Missing blueprint type");
            continue;
        };
        let terrain_texture: Handle<Image> =
            asset_server.load_with_settings("textures/comfy.png", image_loader_settings);

        let basic_material = BasicMaterial {
            texture: None,
            color: Color::WHITE,
            is_lit: false,
            enable_vertex_colors: true,
            enable_slots: false,
            slots_texture: Some(terrain_texture),
            ..Default::default()
        };

        let material = materials.add(basic_material);

        let hotspot_mesh_req = asset_server.load("interface.gltf#Mesh0/Primitive0");
        let hotspot_mesh_opt = asset_server.load("interface_opt.gltf#Mesh0/Primitive0");
        let wire_tile_mesh = asset_server.load("tile_wire.gltf#Mesh0/Primitive0");
        let hotspot_material = materials.add(BasicMaterial::from_color(Color::BLUE));

        let guides = blueprint
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| {
                let center = [
                    (blueprint.center[0] as i32),
                    blueprint.center[1] as i32,
                    (blueprint.center[2] as i32),
                ];

                let transform = Transform::from_translation(Vec3::new(
                    (tile.position[0] - center[0]) as f32,
                    (tile.position[1] - center[1]) as f32,
                    -(tile.position[2] - center[2]) as f32,
                ));

                if let Some(hotspot) = tile.hotspot {
                    let hotspot_e = cmd
                        .spawn((
                            MaterialMeshBundle {
                                mesh: match hotspot.is_optional {
                                    true => hotspot_mesh_opt.clone(),
                                    false => hotspot_mesh_req.clone(),
                                },
                                material: hotspot_material.clone(),
                                visibility: Visibility::Inherited,
                                transform: transform.with_rotation(hotspot.direction.as_quat()),
                                ..default()
                            },
                            StructureGuide {
                                tile_idx: idx,
                                is_valid: false,
                                is_hotspot: true,
                            },
                        ))
                        .id();

                    cmd.entity(hotspot_e).set_parent(ev.entity);
                    return Some(hotspot_e);
                };

                if tile.requirements.contains(TileRequirement::IS_WALKABLE) {
                    let hotspot_e = cmd
                        .spawn((
                            MaterialMeshBundle {
                                mesh: wire_tile_mesh.clone(),
                                material: hotspot_material.clone(),
                                visibility: Visibility::Inherited,
                                transform,
                                ..default()
                            },
                            StructureGuide {
                                tile_idx: idx,
                                is_valid: false,
                                is_hotspot: false,
                            },
                        ))
                        .id();

                    cmd.entity(hotspot_e).set_parent(ev.entity);
                    return Some(hotspot_e);
                }

                None
            })
            .collect::<Vec<_>>();

        cmd.entity(ev.entity).insert((
            Name::new(blueprint.name.clone()),
            Structure {
                position: ev.pos,
                is_valid: false,
                is_dirty: false,
                is_hotspots_valid: false,
                blueprint_type: ev.blueprint_type,
                guides,
                rotation: 0,
                is_flipped: true,
                tiles: vec![],
                mode: StructureMode::Placing,
            },
            PartSlots::from_build_slots(&blueprint.slots),
            Position::default(),
            MaterialMeshBundle {
                mesh: blueprint.mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(
                    ev.pos[0] as f32 + 0.5,
                    ev.pos[1] as f32,
                    ev.pos[2] as f32 + 0.5,
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
        ));
    }
}

pub fn on_build_structure(
    mut ev_build_structure: EventReader<BuildStructureEvent>,
    mut ev_built_structure: EventWriter<BuiltStructureEvent>,
    mut q_structures: Query<&mut Structure>,
    mut terrain: ResMut<Terrain>,
) {
    for ev in ev_build_structure.read() {
        let Ok(mut structure) = q_structures.get_mut(ev.entity) else {
            println!("Structure cannot be built - does not exist");
            continue;
        };

        if !structure.is_valid {
            println!("structure no longer valid! Cannot build.");
            continue;
        }

        structure.mode = StructureMode::Built;
        structure.is_dirty = true;

        for tile in structure.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);

            let flags = if tile.nav_flags == NavigationFlags::NONE {
                None
            } else {
                Some(tile.nav_flags)
            };

            terrain.add_structure(
                chunk_idx,
                block_idx,
                ev.entity,
                StructureTileDetail {
                    is_built: true,
                    flags,
                    is_blocker: tile.is_blocker,
                    is_occupied: tile.is_occupied,
                },
            );

            terrain.set_chunk_nav_dirty(chunk_idx, true);
        }

        ev_built_structure.send(BuiltStructureEvent {
            entity: ev.entity,
            blueprint_type: structure.blueprint_type,
        });
    }
}

pub fn on_remove_structure(
    mut cmd: Commands,
    mut terrain: ResMut<Terrain>,
    q_structures: Query<(&Structure, &PartSlots, &Handle<BasicMaterial>, &Position)>,
    mut ev_remove_structure: EventReader<RemoveStructureEvent>,
    q_jobs: Query<(Entity, &JobBuild)>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    mut ev_job_cancel: EventWriter<JobCancelEvent>,
    mut q_transforms: Query<&mut Transform>,
) {
    for ev in ev_remove_structure.read() {
        cmd.entity(ev.entity).despawn_recursive();

        let Ok((structure, slots, mat_handle, position)) = q_structures.get(ev.entity) else {
            println!("Cannot remove structure, doesn't exist?");
            continue;
        };

        for tile in structure.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
            terrain.remove_structure(chunk_idx, block_idx, &ev.entity);
            terrain.set_chunk_nav_dirty(chunk_idx, true);
        }

        if matches!(structure.mode, StructureMode::Placed) {
            for (job_entity, job_build) in q_jobs.iter() {
                if job_build.structure == ev.entity {
                    ev_job_cancel.send(JobCancelEvent(job_entity));

                    break;
                }
            }
        }

        for slot in slots.as_vec() {
            let Some(content) = slot.content else {
                continue;
            };

            let mut ecmd = cmd.entity(content);
            ecmd.insert(Visibility::Inherited);
            ecmd.remove::<InSlot>();

            if let Ok(mut transform) = q_transforms.get_mut(content) {
                transform.translation.x = position.x as f32 + 0.5;
                transform.translation.y = position.y as f32;
                transform.translation.z = position.z as f32 + 0.5;
            }

            if let Some(material) = basic_materials.get_mut(mat_handle) {
                material.remove_slot(slot.idx);
            };
        }
    }
}

pub fn check_structures(
    mut terrain: ResMut<Terrain>,
    mut q_structures: Query<(Entity, &mut Structure, &mut Transform)>,
    mut q_guides: Query<&mut StructureGuide>,
    mut ev_remove_structure: EventWriter<RemoveStructureEvent>,
    blueprints: Res<Blueprints>,
) {
    for (entity, mut structure, mut transform) in q_structures.iter_mut() {
        if !structure.is_dirty {
            continue;
        }

        let Some(blueprint) = blueprints.0.get(&structure.blueprint_type) else {
            println!("Missing blueprint type!");
            continue;
        };

        transform.translation.x = structure.position[0] as f32 + 0.5;
        transform.translation.y = structure.position[1] as f32;
        transform.translation.z = structure.position[2] as f32 + 0.5;

        if structure.is_flipped {
            transform.scale = Vec3::new(1., 1., 1.);
            transform.rotation = match structure.rotation {
                1 => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
                2 => Quat::from_rotation_y(std::f32::consts::PI),
                3 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                _ => Quat::from_rotation_y(0.),
            };
        } else {
            transform.scale = Vec3::new(1., 1., -1.);
            transform.rotation = match structure.rotation {
                1 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                2 => Quat::from_rotation_y(std::f32::consts::PI),
                3 => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
                _ => Quat::from_rotation_y(0.),
            };
        }

        let center = [
            blueprint.center[0] as i32,
            blueprint.center[1] as i32,
            blueprint.center[2] as i32,
        ];

        structure.is_valid = true;
        structure.is_hotspots_valid = true;
        structure.is_dirty = false;

        // remove current tiles affecting terrain
        for tile in structure.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
            terrain.remove_structure(chunk_idx, block_idx, &entity);
        }

        structure.tiles = blueprint
            .tiles
            .iter()
            .enumerate()
            .map(|(idx, tile)| {
                let tr = apply_transforms(
                    center,
                    tile.position,
                    structure.rotation,
                    structure.is_flipped,
                );
                let pos_i32 = [
                    structure.position[0] as i32 + tr[0],
                    structure.position[1] as i32 + tr[1],
                    structure.position[2] as i32 + tr[2],
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

                    let structures = terrain.get_structures(chunk_idx, block_idx);

                    if guide_is_valid {
                        if tile.is_blocker {
                            guide_is_valid = structures.is_empty();
                        } else {
                            guide_is_valid = !structures.values().any(|v| v.is_blocker)
                        }
                    }

                    if !hotspot.is_optional && !guide_is_valid {
                        structure.is_hotspots_valid = false;
                    }
                }

                let tile_is_valid =
                    is_structure_tile_valid(tile, pos_i32, &terrain, structure.is_built());
                if !tile_is_valid {
                    structure.is_valid = false;
                    guide_is_valid = false;
                }

                structure.guides.iter().enumerate().for_each(|(_, h)| {
                    let Ok(mut guide) = q_guides.get_mut(*h) else {
                        return;
                    };

                    if guide.tile_idx != idx {
                        return;
                    }

                    guide.is_valid = guide_is_valid;
                });

                StructureTile {
                    requirements: tile.requirements,
                    nav_flags: tile.nav_flags,
                    is_blocker: tile.is_blocker,
                    is_occupied: tile.is_occupied,
                    hotspot: tile.hotspot,
                    position: pos_i32,
                }
            })
            .collect::<Vec<_>>();

        if matches!(structure.mode, StructureMode::Built | StructureMode::Placed)
            && !structure.is_valid
        {
            ev_remove_structure.send(RemoveStructureEvent { entity });
        }

        // set tiles affecting terrain
        for tile in structure.tiles.iter() {
            let [x, y, z] = tile.position;
            let [chunk_idx, block_idx] = terrain.get_block_indexes(x as u32, y as u32, z as u32);
            let flags = if tile.nav_flags == NavigationFlags::NONE {
                None
            } else {
                Some(tile.nav_flags)
            };

            terrain.add_structure(
                chunk_idx,
                block_idx,
                entity,
                StructureTileDetail {
                    is_built: structure.is_built(),
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

fn is_structure_tile_valid(
    tile: &BlueprintTile,
    pos: [i32; 3],
    terrain: &Terrain,
    is_built: bool,
) -> bool {
    let block = terrain.get_block_i32(pos[0], pos[1], pos[2]);

    if block.is_oob() {
        return false;
    }

    let [chunk_idx, block_idx] =
        terrain.get_block_indexes(pos[0] as u32, pos[1] as u32, pos[2] as u32);

    tile.requirements.iter().all(|flag| match flag {
        TileRequirement::IS_EMPTY => {
            if !block.is_empty() {
                return false;
            }

            let structures = terrain.get_structures(chunk_idx, block_idx);

            if tile.is_blocker || tile.is_occupied {
                !structures.values().any(|v| {
                    if is_built {
                        v.is_built && (v.is_blocker || v.is_occupied)
                    } else {
                        v.is_blocker || v.is_occupied
                    }
                })
            } else if is_built {
                !structures.values().any(|v| v.is_blocker && v.is_built)
            } else {
                !structures.values().any(|v| v.is_blocker)
            }
        }
        TileRequirement::IS_ATTACHABLE => block.is_attachable(),
        TileRequirement::IS_WALKABLE => {
            let below = terrain.get_block_i32(pos[0], pos[1] - 1, pos[2]);

            below.is_walkable()
        }
        _ => true,
    })
}
