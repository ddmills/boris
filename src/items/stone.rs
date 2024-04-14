use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::default,
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};

use crate::{
    colonists::{Faller, InPartition, Item, ItemTag, NavigationGraph},
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh: Handle<Mesh> = asset_server.load("meshes/sphere.obj");
    let material = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        unlit: true,
        ..default()
    });

    for ev in ev_spawn_stone.read() {
        let entity = cmd
            .spawn((
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

        println!("spawning stone {}", entity.index());

        let Some(partition_id) = terrain.get_partition_id_u32(ev.pos[0], ev.pos[1], ev.pos[2])
        else {
            continue;
        };

        let Some(partition) = graph.get_partition_mut(partition_id) else {
            println!("Missing partition trying to insert item! {}", partition_id);
            continue;
        };

        partition.items.insert(entity);
        cmd.entity(entity).insert(InPartition {
            partition_id: *partition_id,
        });
    }
}
