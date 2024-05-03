use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{color::Color, texture::Image, view::Visibility},
    transform::components::Transform,
};

use crate::{
    colonists::{Faller, Item, ItemTag},
    rendering::BasicMaterial,
    Position,
};

use super::image_loader_settings;

#[derive(Event)]
pub struct SpawnLogEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_log(
    mut cmd: Commands,
    mut ev_spawn_log: EventReader<SpawnLogEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_log.read() {
        let log_texture: Handle<Image> =
            asset_server.load_with_settings("textures/wood.png", image_loader_settings);
        let mesh = asset_server.load("log.gltf#Mesh0/Primitive0");
        let material = materials.add(BasicMaterial {
            texture: Some(log_texture.clone()),
            sunlight: 8,
            torchlight: 8,
            color: Color::WHITE,
        });

        cmd.spawn((
            Name::new("Log"),
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(
                    ev.pos[0] as f32 + 0.5,
                    ev.pos[1] as f32,
                    ev.pos[2] as f32 + 0.5,
                ),
                visibility: Visibility::Visible,
                ..default()
            },
            Item {
                tags: vec![ItemTag::Log, ItemTag::BasicBuildMaterial],
                reserved: None,
            },
            Faller,
            Position::default(),
        ));
    }
}
