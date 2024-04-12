use std::sync::Arc;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::default,
    render::{color::Color, mesh::Mesh, texture::Image},
    transform::components::Transform,
};

use super::{
    Actor, Faller, Fatigue, Inventory, NavigationFlags, ScorerMine, ScorerWander, Thinker,
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
) {
    for ev in ev_spawn_colonist.read() {
        for i in 0..10 {
            let texture: Handle<Image> = asset_server.load("textures/colonist.png");
            let mesh: Handle<Mesh> = asset_server.load("meshes/basemesh.obj");
            let material = materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                unlit: true,
                ..default()
            });

            cmd.spawn((
                MaterialMeshBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
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
                    score_builders: vec![Arc::new(ScorerWander), Arc::new(ScorerMine::default())],
                },
                Faller,
                NavigationFlags::COLONIST,
            ));
        }
    }
}
