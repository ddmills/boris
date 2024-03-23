use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::default,
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};

use super::{Actor, Fatigue};

#[derive(Component, Default)]
pub struct Colonist {}

#[derive(Event)]
pub struct SpawnColonistEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_colonist(
    mut commands: Commands,
    mut ev_spawn_colonist: EventReader<SpawnColonistEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = asset_server.load("meshes/colonist.obj");
    let material = materials.add(Color::ORANGE);

    for ev in ev_spawn_colonist.read() {
        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(
                    ev.pos[0] as f32,
                    ev.pos[1] as f32,
                    ev.pos[2] as f32,
                ),
                ..default()
            },
            Fatigue {
                value: 30.,
                per_second: 5.,
            },
            Actor,
            Colonist::default(),
        ));
    }
}
