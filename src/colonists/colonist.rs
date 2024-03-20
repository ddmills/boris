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
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};

use super::{ActFindBed, ActNone, ActSleep, ActState, Actor, Behavior, Brain, Fatigue};

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
    let cube: Handle<Mesh> = asset_server.load("meshes/cube.obj");
    let material = materials.add(Color::ORANGE);
    let mut pairs = vec![];

    for ev in ev_spawn_colonist.read() {
        let behavior_id = commands
            .spawn((Behavior { idx: 0 }, ActState::Success))
            .id();

        let actor_id = commands
            .spawn((
                MaterialMeshBundle {
                    mesh: cube.clone(),
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
                Brain {
                    behavior: behavior_id,
                    actions: vec![Arc::new(ActFindBed), Arc::new(ActSleep), Arc::new(ActNone)],
                },
                Colonist::default(),
            ))
            .id();

        pairs.push((actor_id, behavior_id));
    }

    for (actor_id, behavior_id) in pairs {
        commands.entity(behavior_id).insert(Actor(actor_id));
    }
}
