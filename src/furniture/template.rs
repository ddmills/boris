use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::{Commands, Res, Resource},
    hierarchy::BuildChildren,
    math::{Quat, Vec3},
    render::{mesh::Mesh, texture::Image},
    utils::hashbrown::HashMap,
};

use crate::{colonists::NavigationFlags, items::image_loader_settings, Position};

use bevy::{
    asset::Assets,
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::{Query, ResMut},
    },
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{color::Color, view::Visibility},
    transform::components::Transform,
};

use crate::rendering::BasicMaterial;

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use super::{Blueprint, BlueprintMode};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TemplateTileRequirement: u16 {
        const IS_WALKABLE = 1;
        const IS_EMPTY = 2;
        const IS_ATTACHABLE = 4;
    }
}

impl Display for TemplateTileRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionSimple {
    North,
    East,
    South,
    West,
}

impl DirectionSimple {
    pub fn as_quat(&self) -> Quat {
        match self {
            Self::North => Quat::from_rotation_y(0.),
            Self::East => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
            Self::South => Quat::from_rotation_y(std::f32::consts::PI),
            Self::West => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateHotspot {
    pub is_optional: bool,
    pub direction: DirectionSimple,
    pub nav_flag_requirements: NavigationFlags,
}

#[derive(Debug, Clone, Component)]
pub struct BlueprintGuide {
    pub tile_idx: usize,
    pub is_valid: bool,
    pub is_hotspot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateTile {
    pub requirements: TemplateTileRequirement,
    pub nav_flags: NavigationFlags,
    pub is_blocker: bool,
    pub is_occupied: bool,
    pub hotspot: Option<TemplateHotspot>,
    pub position: [i32; 3],
}

#[derive(Event)]
pub struct SpawnBlueprintEvent {
    pub pos: [u32; 3],
    pub entity: Entity,
    pub template_type: TemplateType,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TemplateType {
    Workbench,
    Bigbench,
    Ladder,
    TorchStanding,
    TorchWall,
}

pub struct Template {
    pub name: String,
    pub center: [u32; 3],
    pub tiles: Vec<TemplateTile>,
    pub texture: Handle<Image>,
    pub mesh: Handle<Mesh>,
}

#[derive(Resource)]
pub struct Templates {
    pub templates: HashMap<TemplateType, Template>,
}

pub fn on_spawn_blueprint(
    mut cmd: Commands,
    mut ev_spawn_blueprint: EventReader<SpawnBlueprintEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
    templates: Res<Templates>,
) {
    for ev in ev_spawn_blueprint.read() {
        let Some(template) = templates.templates.get(&ev.template_type) else {
            println!("Missing template type");
            continue;
        };

        let material = materials.add(BasicMaterial {
            texture: Some(template.texture.clone()),
            sunlight: 8,
            torchlight: 8,
            color: Color::RED,
        });

        let hotspot_mesh_req = asset_server.load("interface.gltf#Mesh0/Primitive0");
        let hotspot_mesh_opt = asset_server.load("interface_opt.gltf#Mesh0/Primitive0");
        let wire_tile_mesh = asset_server.load("tile_wire.gltf#Mesh0/Primitive0");
        let hotspot_material = materials.add(BasicMaterial::from_color(Color::BLUE));

        let guides = template
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| {
                let center = [
                    (template.center[0] as i32),
                    template.center[1] as i32,
                    (template.center[2] as i32),
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
                            BlueprintGuide {
                                tile_idx: idx,
                                is_valid: false,
                                is_hotspot: true,
                            },
                        ))
                        .id();

                    cmd.entity(hotspot_e).set_parent(ev.entity);
                    return Some(hotspot_e);
                };

                if tile
                    .requirements
                    .contains(TemplateTileRequirement::IS_WALKABLE)
                {
                    let hotspot_e = cmd
                        .spawn((
                            MaterialMeshBundle {
                                mesh: wire_tile_mesh.clone(),
                                material: hotspot_material.clone(),
                                visibility: Visibility::Inherited,
                                transform,
                                ..default()
                            },
                            BlueprintGuide {
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
            Name::new(template.name.clone()),
            Blueprint {
                position: ev.pos,
                is_valid: false,
                is_dirty: false,
                is_hotspots_valid: false,
                template_type: ev.template_type,
                guides,
                rotation: 0,
                is_flipped: true,
                tiles: vec![],
                mode: BlueprintMode::Placing,
            },
            Position::default(),
            MaterialMeshBundle {
                mesh: template.mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(
                    ev.pos[0] as f32 + 0.5,
                    ev.pos[1] as f32,
                    ev.pos[2] as f32 + 0.5,
                ),
                visibility: Visibility::Visible,
                ..default()
            },
        ));
    }
}

pub fn blueprint_material_update(
    mut cmd: Commands,
    q_blueprints: Query<(&Blueprint, &Handle<BasicMaterial>)>,
    q_guides: Query<(&BlueprintGuide, &Handle<BasicMaterial>)>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
) {
    for (blueprint, material_handle) in q_blueprints.iter() {
        for guide_e in blueprint.guides.iter() {
            let Ok((guide, guide_mat_handle)) = q_guides.get(*guide_e) else {
                continue;
            };

            let Some(guide_material) = basic_materials.get_mut(guide_mat_handle) else {
                continue;
            };

            if matches!(blueprint.mode, BlueprintMode::Placed | BlueprintMode::Built) {
                cmd.entity(*guide_e).insert(Visibility::Hidden);
                continue;
            }

            if guide.is_hotspot {
                if blueprint.is_valid && guide.is_valid {
                    cmd.entity(*guide_e).insert(Visibility::Inherited);
                } else {
                    cmd.entity(*guide_e).insert(Visibility::Hidden);
                }
            }

            guide_material.color = match blueprint.is_valid {
                true => match blueprint.is_hotspots_valid {
                    true => Color::rgb_from_array([0.435, 0.656, 0.851]),
                    false => Color::YELLOW,
                },
                false => Color::RED,
            };
        }

        let Some(material) = basic_materials.get_mut(material_handle) else {
            continue;
        };

        if (matches!(blueprint.mode, BlueprintMode::Placing) && blueprint.is_valid)
            || matches!(blueprint.mode, BlueprintMode::Built)
        {
            material.color = Color::WHITE;
            continue;
        }

        material.color = match blueprint.is_valid {
            true => match blueprint.is_hotspots_valid {
                true => Color::rgb_from_array([0.435, 0.656, 0.851]),
                false => Color::YELLOW,
            },
            false => Color::RED,
        };
    }
}

pub fn setup_templates(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let mut templates = HashMap::new();

    let stone_texture: Handle<Image> =
        asset_server.load_with_settings("textures/stone.png", image_loader_settings);

    templates.insert(
        TemplateType::Workbench,
        Template {
            name: "Workbench".to_string(),
            center: [0, 0, 0],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 0, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 0, -1],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [1, 0, -1],
                    hotspot: Some(TemplateHotspot {
                        is_optional: false,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 1, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("workbench.gltf#Mesh0/Primitive0"),
        },
    );

    templates.insert(
        TemplateType::Ladder,
        Template {
            name: "Ladder".to_string(),
            center: [0, 0, 0],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::LADDER,
                    is_blocker: false,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::LADDER,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 1],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_ATTACHABLE,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("ladder.gltf#Mesh0/Primitive0"),
        },
    );

    templates.insert(
        TemplateType::TorchWall,
        Template {
            name: "Wall torch".to_string(),
            center: [0, 0, 0],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, -1, 0],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 1],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_ATTACHABLE,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("torch_wall.gltf#Mesh0/Primitive0"),
        },
    );

    templates.insert(
        TemplateType::TorchStanding,
        Template {
            name: "Standing torch".to_string(),
            center: [0, 0, 0],
            tiles: vec![
                TemplateTile {
                    position: [-1, 0, 0],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::East,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [1, 0, 0],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::West,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 1],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::South,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, -1],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY
                        | TemplateTileRequirement::IS_WALKABLE,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("torch_standing.gltf#Mesh0/Primitive0"),
        },
    );

    templates.insert(
        TemplateType::Bigbench,
        Template {
            name: "Big workbench".to_string(),
            center: [1, 0, 3],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 1, 0],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 0, 0],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::West,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 1],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 1, 1],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 0, 1],
                    hotspot: Some(TemplateHotspot {
                        is_optional: false,
                        direction: DirectionSimple::West,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 2],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 1, 2],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 0, 2],
                    hotspot: Some(TemplateHotspot {
                        is_optional: false,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [2, 0, 2],
                    hotspot: Some(TemplateHotspot {
                        is_optional: true,
                        direction: DirectionSimple::North,
                        nav_flag_requirements: NavigationFlags::TALL,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: false,
                    is_occupied: false,
                },
                TemplateTile {
                    position: [0, 0, 3],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [0, 1, 3],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 0, 3],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [1, 1, 3],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [2, 0, 3],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_WALKABLE
                        | TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
                TemplateTile {
                    position: [2, 1, 3],
                    hotspot: None,
                    requirements: TemplateTileRequirement::IS_EMPTY,
                    nav_flags: NavigationFlags::NONE,
                    is_blocker: true,
                    is_occupied: true,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("bigbench.gltf#Mesh0/Primitive0"),
        },
    );

    cmd.insert_resource(Templates { templates });
}
