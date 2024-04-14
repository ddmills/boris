use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
};

use crate::{BlockType, Terrain};

use super::{Job, JobBuild, JobLocation, JobType};

#[derive(Event)]
pub struct SpawnJobBuildEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_job_build(
    mut cmd: Commands,
    mut terrain: ResMut<Terrain>,
    mut ev_spawn_job_mine: EventReader<SpawnJobBuildEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_job_mine.read() {
        let mesh: Handle<Mesh> = asset_server.load("meshes/cube.obj");
        let material = materials.add(StandardMaterial {
            base_color: Color::MIDNIGHT_BLUE,
            unlit: true,
            ..default()
        });

        terrain.set_block_type(ev.pos[0], ev.pos[1], ev.pos[2], BlockType::BLUEPRINT);

        cmd.spawn((
            // MaterialMeshBundle {
            //     mesh: mesh.clone(),
            //     material: material.clone(),
            //     transform: Transform::from_xyz(
            //         ev.pos[0] as f32 + 0.5,
            //         ev.pos[1] as f32,
            //         ev.pos[2] as f32 + 0.5,
            //     ),
            //     ..default()
            // },
            Job {
                job_type: JobType::BuildWall,
                assignee: None,
            },
            JobBuild,
            JobLocation { pos: ev.pos },
        ));
    }
}
