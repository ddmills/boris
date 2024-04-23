use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::MaterialMeshBundle,
    prelude::default,
    render::{
        color::Color,
        texture::{
            Image, ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
            ImageSamplerDescriptor,
        },
    },
    transform::components::Transform,
};

use crate::{
    colonists::{Faller, Item, ItemTag},
    rendering::BasicMaterial,
    Position,
};

#[derive(Event)]
pub struct SpawnPickaxeEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_pickaxe(
    mut cmd: Commands,
    mut ev_spawn_pickaxe: EventReader<SpawnPickaxeEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_pickaxe.read() {
        let settings = |s: &mut ImageLoaderSettings| {
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                address_mode_w: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Nearest,
                mipmap_filter: ImageFilterMode::Nearest,
                ..default()
            });
        };

        let stone_texture: Handle<Image> =
            asset_server.load_with_settings("textures/stone.png", settings);
        let mesh = asset_server.load("axe.gltf#Mesh0/Primitive0");
        let material = materials.add(BasicMaterial {
            texture: Some(stone_texture.clone()),
            sunlight: 8,
            torchlight: 8,
            color: Color::WHITE,
        });

        cmd.spawn((
            Name::new("Pickaxe"),
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
            Item {
                tags: vec![ItemTag::Pickaxe],
                reserved: None,
            },
            Faller,
            Position::default(),
        ));
    }
}
