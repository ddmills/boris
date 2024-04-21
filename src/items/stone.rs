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
        render_resource::AddressMode,
        texture::{
            Image, ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
            ImageSamplerDescriptor,
        },
    },
    transform::components::Transform,
};

use crate::{
    colonists::{Faller, InPartition, Item, ItemTag, NavigationGraph},
    rendering::BasicMaterial,
    Terrain,
};

#[derive(Event)]
pub struct SpawnStoneEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_stone(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    mut graph: ResMut<NavigationGraph>,
    mut ev_spawn_stone: EventReader<SpawnStoneEvent>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_stone.read() {
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
        let mesh = asset_server.load("sphere.gltf#Mesh0/Primitive0");
        let material = materials.add(BasicMaterial {
            texture: Some(stone_texture.clone()),
            light: 8,
            color: Color::WHITE,
        });

        let entity = cmd
            .spawn((
                Name::new("Stone"),
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
                    tags: vec![ItemTag::Stone],
                    reserved: None,
                },
                Faller,
            ))
            .id();

        let Some(partition_id) = terrain.get_partition_id_u32(ev.pos[0], ev.pos[1], ev.pos[2])
        else {
            continue;
        };

        let Some(partition) = graph.get_partition_mut(&partition_id) else {
            println!("Missing partition trying to insert item! {}", partition_id);
            continue;
        };

        partition.items.insert(entity);
        cmd.entity(entity).insert(InPartition { partition_id });
    }
}
