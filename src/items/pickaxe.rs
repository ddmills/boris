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
    transform::{commands, components::Transform},
};

#[derive(Event)]
pub struct SpawnPickaxeEvent {
    pub pos: [u32; 3],
}

#[derive(Component)]
pub struct Item;

pub fn on_spawn_pickaxe(
    mut cmd: Commands,
    mut ev_spawn_pickaxe: EventReader<SpawnPickaxeEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = asset_server.load("meshes/sphere.obj");
    let material = materials.add(Color::CYAN);

    for ev in ev_spawn_pickaxe.read() {
        cmd.spawn((
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
            Item,
        ));
    }
}
