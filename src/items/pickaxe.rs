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
    colonists::{Item, ItemTag, Partition, PartitionGraph},
    Terrain,
};

#[derive(Event)]
pub struct SpawnPickaxeEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_pickaxe(
    mut cmd: Commands,
    terrain: Res<Terrain>,
    mut graph: ResMut<PartitionGraph>,
    mut ev_spawn_pickaxe: EventReader<SpawnPickaxeEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh: Handle<Mesh> = asset_server.load("meshes/sphere.obj");
    let material = materials.add(Color::CYAN);

    for ev in ev_spawn_pickaxe.read() {
        let entity = cmd
            .spawn((
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
                Item {
                    tags: vec![ItemTag::PickAxe],
                },
            ))
            .id();

        let partition_id = terrain.get_partition_id_u32(ev.pos[0], ev.pos[1], ev.pos[2]);

        if partition_id != Partition::NONE {
            let Some(partition) = graph.get_partition_mut(partition_id) else {
                println!("Missing partition trying to insert item! {}", partition_id);
                continue;
            };

            partition.items.push(entity);
        }
    }
}
