use std::sync::Arc;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::Children,
    pbr::StandardMaterial,
    prelude::default,
    render::{color::Color, texture::Image, view::Visibility},
    scene::SceneBundle,
    transform::components::Transform,
};

use crate::{colonists::AnimState, rendering::BasicMaterial, HumanGltf, Position};

use super::{
    get_child_by_name_recursive, Actor, AnimClip, Animator, Faller, Fatigue, Inventory,
    NavigationFlags, ScorerBuild, ScorerMine, ScorerWander, Thinker,
};

#[derive(Component, Default)]
pub struct Colonist {}

#[derive(Component)]
pub struct ChildMaterials(pub Handle<BasicMaterial>);

#[derive(Event)]
pub struct SpawnColonistEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_colonist(
    mut cmd: Commands,
    mut ev_spawn_colonist: EventReader<SpawnColonistEvent>,
    human_gltf: Res<HumanGltf>,
) {
    for ev in ev_spawn_colonist.read() {
        cmd.spawn((
            Name::new("Colonist"),
            SceneBundle {
                scene: human_gltf.0.clone(),
                transform: Transform::from_xyz(
                    ev.pos[0] as f32 + 0.5,
                    ev.pos[1] as f32,
                    ev.pos[2] as f32 + 0.5,
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
            Fatigue {
                value: 30.,
                per_second: 5.,
            },
            Actor,
            Inventory::default(),
            Colonist::default(),
            Thinker {
                score_builders: vec![
                    Arc::new(ScorerWander),
                    Arc::new(ScorerMine::default()),
                    Arc::new(ScorerBuild::default()),
                ],
            },
            Faller,
            Position::default(),
            NavigationFlags::COLONIST,
        ));
    }
}

pub fn setup_colonists(
    mut cmd: Commands,
    q_children: Query<&Children>,
    q_names: Query<&Name>,
    q_colonists: Query<Entity, (With<Colonist>, Without<Animator>)>,
    asset_server: Res<AssetServer>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
) {
    for colonist in q_colonists.iter() {
        if let Some(armature) =
            get_child_by_name_recursive(&colonist, "Armature", &q_names, &q_children)
        {
            let mut e_cmd = cmd.entity(colonist);
            e_cmd.insert(Animator {
                clip: AnimClip::Idle,
                armature,
                prev_clip: AnimClip::None,
                state: AnimState::Completed,
            });
        }

        if let Some(mesh) =
            get_child_by_name_recursive(&colonist, "HumanMesh", &q_names, &q_children)
        {
            let mut mesh_cmd = cmd.entity(mesh);

            let texture: Handle<Image> = asset_server.load("textures/colonist.png");

            let basic_material = basic_materials.add(BasicMaterial {
                color: Color::WHITE,
                texture: Some(texture.clone()),
                sunlight: 15,
                torchlight: 15,
            });

            mesh_cmd.insert(basic_material.clone());
            mesh_cmd.remove::<Handle<StandardMaterial>>();

            let mut e_cmd = cmd.entity(colonist);
            e_cmd.insert(ChildMaterials(basic_material.clone()));
        }

        let mut e_cmd = cmd.entity(colonist);
        e_cmd.insert(Visibility::Visible);
    }
}
