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

use super::{Job, JobBuild, JobLocation, JobMine, JobType};

#[derive(Event)]
pub struct SpawnJobMineEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_job_mine(
    mut cmd: Commands,
    mut ev_spawn_job_mine: EventReader<SpawnJobMineEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_job_mine.read() {
        let mesh: Handle<Mesh> = asset_server.load("meshes/cube.obj");
        let material = materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            unlit: true,
            depth_bias: 2.,
            ..default()
        });

        cmd.spawn((
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
            Job {
                job_type: JobType::Mine,
                assignee: None,
            },
            JobMine,
            JobLocation { pos: ev.pos },
        ));
    }
}
