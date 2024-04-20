use std::sync::Arc;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        event::{Event, EventReader},
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    gltf::{Gltf, GltfMesh},
    pbr::{Material, MaterialMeshBundle, PbrBundle, StandardMaterial},
    prelude::default,
    render::{
        color::Color,
        mesh::{skinning::SkinnedMesh, Mesh},
        texture::Image,
    },
    scene::{Scene, SceneBundle},
    transform::components::Transform,
};

use crate::HumanGltf;

use super::{
    Actor, Faller, Fatigue, Inventory, NavigationFlags, ScorerBuild, ScorerMine, ScorerWander,
    Thinker,
};

#[derive(Component, Default)]
pub struct Colonist {}

#[derive(Event)]
pub struct SpawnColonistEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_colonist(
    mut cmd: Commands,
    mut ev_spawn_colonist: EventReader<SpawnColonistEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    human_gltf: Res<HumanGltf>,
    mut scenes: ResMut<Assets<Scene>>,
) {
    let Some(scene) = scenes.get_mut(human_gltf.0.clone()) else {
        println!("gltf not loaded yet?");
        return;
    };
    for ev in ev_spawn_colonist.read() {
        for i in 0..1 {
            let texture: Handle<Image> = asset_server.load("textures/colonist.png");

            for material_handle in scene
                .world
                .query::<&Handle<StandardMaterial>>()
                .iter(&scene.world)
            {
                let Some(material) = materials.get_mut(material_handle) else {
                    continue;
                };
                material.unlit = true;
                material.base_color = Color::WHITE;
                material.base_color_texture = Some(texture.clone());
            }

            cmd.spawn((
                Name::new("Colonist"),
                SceneBundle {
                    scene: human_gltf.0.clone(),
                    transform: Transform::from_xyz(
                        ev.pos[0] as f32 + 0.5,
                        ev.pos[1] as f32,
                        ev.pos[2] as f32 + 0.5,
                    ),
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
                NavigationFlags::COLONIST,
            ));
        }
    }
}
