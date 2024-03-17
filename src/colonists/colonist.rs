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
use big_brain::{actions::Steps, pickers::Highest, thinker::Thinker};

use super::{
    task::{Fatigue, FatigueScorer, SleepAct},
    FollowPathAct, GeneratePathAct, PickWanderSpotAct, WanderScorer,
};

#[derive(Component, Default)]
pub struct Colonist;

#[derive(Component, Default)]
pub struct Agent;

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

    for ev in ev_spawn_colonist.read() {
        commands.spawn((
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
            Agent,
            Colonist,
            Fatigue {
                is_sleeping: false,
                per_second: 1.,
                level: 0.,
            },
            Thinker::build()
                .label("Colonist Thinker")
                .picker(Highest)
                .when(FatigueScorer, Steps::build().label("Sleep").step(SleepAct))
                .when(
                    WanderScorer,
                    Steps::build()
                        .label("WanderAndPonder")
                        .step(PickWanderSpotAct)
                        .step(GeneratePathAct)
                        .step(FollowPathAct),
                ),
        ));
    }
}
