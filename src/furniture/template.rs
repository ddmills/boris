use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::{Commands, Res, Resource},
    hierarchy::BuildChildren,
    math::{Quat, Vec3},
    render::{mesh::Mesh, texture::Image},
    transform::commands::BuildChildrenTransformExt,
    utils::hashbrown::HashMap,
};

use crate::items::image_loader_settings;

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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TemplateTileRequirement: u16 {
        const WALKABLE = 1;
        const EMPTY = 2;
        const ATTACHABLE = 4;
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
    pub fn to_quat(&self) -> Quat {
        match self {
            Self::North => Quat::from_rotation_y(0.),
            Self::East => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
            Self::South => Quat::from_rotation_y(std::f32::consts::PI),
            Self::West => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateInterface {
    pub is_optional: bool,
    pub direction: DirectionSimple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateTile {
    pub requirements: TemplateTileRequirement,
    pub interface: Option<TemplateInterface>,
    pub position: [i32; 3],
}

#[derive(Component)]
pub struct TemplateInstance {
    pub is_valid: bool,
    pub position: [u32; 3],
    pub template_type: TemplateType,
}

#[derive(Component)]
pub struct TemplateInterfaces {
    pub interfaces: Vec<Entity>,
}

#[derive(Event)]
pub struct SpawnTemplateEvent {
    pub pos: [u32; 3],
    pub entity: Entity,
    pub template_type: TemplateType,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TemplateType {
    Workbench,
    Bigbench,
    Ladder,
}

pub struct Template {
    pub center: [u32; 3],
    pub tiles: Vec<TemplateTile>,
    pub texture: Handle<Image>,
    pub mesh: Handle<Mesh>,
}

#[derive(Resource)]
pub struct Templates {
    pub templates: HashMap<TemplateType, Template>,
    pub interfaces: Vec<Entity>,
}

pub fn on_spawn_template(
    mut cmd: Commands,
    mut ev_spawn_template: EventReader<SpawnTemplateEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
    templates: Res<Templates>,
) {
    for ev in ev_spawn_template.read() {
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

        cmd.entity(ev.entity).insert((
            Name::new("Workbench template"),
            TemplateInstance {
                position: ev.pos,
                is_valid: false,
                template_type: ev.template_type,
            },
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

        let interface_mesh = asset_server.load("interface.gltf#Mesh0/Primitive0");
        let interface_material = materials.add(BasicMaterial {
            color: Color::GREEN,
            texture: None,
            sunlight: 15,
            torchlight: 15,
        });

        let interfaces = template
            .tiles
            .iter()
            .filter_map(|tile| {
                let interface: TemplateInterface = tile.interface?;

                let center = [
                    (template.center[0] as i32),
                    template.center[1] as i32,
                    (template.center[2] as i32),
                ];

                let transform = Transform::from_translation(Vec3::new(
                    (tile.position[0] - center[0]) as f32,
                    (tile.position[1] - center[1]) as f32,
                    -(tile.position[2] - center[2]) as f32,
                ))
                .with_rotation(interface.direction.to_quat());
                // .with_scale(Vec3::new(1., 1., -1.));

                let interface_e = cmd
                    .spawn(MaterialMeshBundle {
                        mesh: interface_mesh.clone(),
                        material: interface_material.clone(),
                        visibility: Visibility::Inherited,
                        transform,
                        ..default()
                    })
                    .id();

                cmd.entity(interface_e).set_parent(ev.entity);

                // cmd.entity(ev.entity).add_child(interface_e);
                Some(interface_e)
            })
            .collect::<Vec<_>>();

        cmd.entity(ev.entity)
            .insert(TemplateInterfaces { interfaces });
    }
}

pub fn template_material_update(
    q_templates: Query<(&TemplateInstance, &Handle<BasicMaterial>)>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
) {
    for (template, material_handle) in q_templates.iter() {
        let Some(material) = basic_materials.get_mut(material_handle) else {
            continue;
        };

        material.color = match template.is_valid {
            true => Color::GREEN,
            false => Color::RED,
        }
    }
}

pub fn setup_templates(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BasicMaterial>>,
) {
    let interface_mesh = asset_server.load("interface.gltf#Mesh0/Primitive0");
    let interface_material = materials.add(BasicMaterial {
        color: Color::GREEN,
        texture: None,
        sunlight: 15,
        torchlight: 15,
    });

    let interfaces = (0..20)
        .map(|_| {
            cmd.spawn(MaterialMeshBundle {
                mesh: interface_mesh.clone(),
                material: interface_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            })
            .id()
        })
        .collect::<Vec<_>>();

    let mut templates = HashMap::new();

    let stone_texture: Handle<Image> =
        asset_server.load_with_settings("textures/stone.png", image_loader_settings);

    templates.insert(
        TemplateType::Workbench,
        Template {
            center: [0, 0, 0],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 0, 0],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [0, 0, -1],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::North,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                },
                TemplateTile {
                    position: [1, 0, -1],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::North,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                },
                TemplateTile {
                    position: [0, 1, 0],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 1, 0],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("workbench.gltf#Mesh0/Primitive0"),
        },
    );

    templates.insert(
        TemplateType::Ladder,
        Template {
            center: [0, 0, 0],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::North,
                    }),
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [0, 0, 1],
                    interface: None,
                    requirements: TemplateTileRequirement::ATTACHABLE,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("ladder.gltf#Mesh0/Primitive0"),
        },
    );

    templates.insert(
        TemplateType::Bigbench,
        Template {
            center: [1, 0, 2],
            tiles: vec![
                TemplateTile {
                    position: [0, 0, 0],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [0, 1, 0],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 0, 0],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::West,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                },
                TemplateTile {
                    position: [0, 0, 1],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [0, 1, 1],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 0, 1],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::West,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                },
                TemplateTile {
                    position: [0, 0, 2],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [0, 1, 2],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 0, 2],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::North,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                },
                TemplateTile {
                    position: [2, 0, 2],
                    interface: Some(TemplateInterface {
                        is_optional: true,
                        direction: DirectionSimple::North,
                    }),
                    requirements: TemplateTileRequirement::empty(),
                },
                TemplateTile {
                    position: [0, 0, 3],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [0, 1, 3],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 0, 3],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [1, 1, 3],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [2, 0, 3],
                    interface: None,
                    requirements: TemplateTileRequirement::WALKABLE
                        | TemplateTileRequirement::EMPTY,
                },
                TemplateTile {
                    position: [2, 1, 3],
                    interface: None,
                    requirements: TemplateTileRequirement::EMPTY,
                },
            ],
            texture: stone_texture.clone(),
            mesh: asset_server.load("bigbench.gltf#Mesh0/Primitive0"),
        },
    );

    cmd.insert_resource(Templates {
        templates,
        interfaces,
    });
}
