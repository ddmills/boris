use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{texture::Image, view::Visibility},
    transform::components::Transform,
};

use crate::{
    colonists::{Faller, Item, ItemTag},
    rendering::BasicMaterial,
    Position,
};

use super::image_loader_settings;

#[derive(Event)]
pub struct SpawnAxeEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_axe(
    mut cmd: Commands,
    mut ev_spawn_axe: EventReader<SpawnAxeEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_axe.read() {
        let stone_texture: Handle<Image> =
            asset_server.load_with_settings("textures/stone.png", image_loader_settings);
        let mesh = asset_server.load("axe.gltf#Mesh0/Primitive0");
        let material = materials.add(BasicMaterial {
            texture: Some(stone_texture.clone()),
            ..Default::default()
        });

        cmd.spawn((
            Name::new("Axe"),
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
                tags: vec![ItemTag::Axe],
                reserved: None,
            },
            Faller,
            Position::default(),
        ));
    }
}
