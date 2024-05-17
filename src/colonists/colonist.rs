use std::sync::Arc;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        event::{Event, EventReader},
        system::{Commands, Res, ResMut},
    },
    math::primitives::Cuboid,
    prelude::default,
    render::{
        mesh::{shape::Cube, Mesh},
        view::Visibility,
    },
    scene::SceneBundle,
    transform::components::Transform,
};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::SimplifiedMesh,
    events::{Down, Pointer},
    prelude::On,
    PickableBundle,
};

use crate::{
    rendering::{BasicMaterial, GltfBinding},
    ui::ColonistClickedEvent,
    Position,
};

use super::{
    Actor, Faller, Fatigue, Inventory, NavigationFlags, ScorerBuild, ScorerChop, ScorerMine,
    ScorerPlaceBlock, ScorerSupply, ScorerWander, Thinker,
};

#[derive(Component, Default)]
pub struct Colonist {}

#[derive(Component)]
pub struct ChildMaterials(pub Handle<BasicMaterial>);

#[derive(Event)]
pub struct SpawnColonistEvent {
    pub pos: [u32; 3],
}

pub fn on_spawn_colonist(
    mut cmd: Commands,
    mut ev_spawn_colonist: EventReader<SpawnColonistEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_colonist.read() {
        let gltf = asset_server.load("human.gltf#Scene0");

        cmd.spawn((
            Name::new("Colonist"),
            SceneBundle {
                scene: gltf,
                transform: Transform::from_xyz(
                    ev.pos[0] as f32 + 0.5,
                    ev.pos[1] as f32,
                    ev.pos[2] as f32 + 0.5,
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
            SimplifiedMesh {
                mesh: meshes.add(Cuboid::default()),
            },
            GltfBinding {
                armature_name: Some("Armature".into()),
                mesh_name: "HumanMesh".into(),
                texture_path: Some("textures/colonist.png".into()),
            },
            Fatigue {
                value: 30.,
                per_second: 5.,
            },
            Actor,
            Inventory::default(),
            Colonist::default(),
            PickableBundle::default(),
            On::<Pointer<Down>>::send_event::<ColonistClickedEvent>(),
            Thinker {
                score_builders: vec![
                    Arc::new(ScorerWander),
                    Arc::new(ScorerMine::default()),
                    Arc::new(ScorerChop::default()),
                    Arc::new(ScorerPlaceBlock::default()),
                    Arc::new(ScorerBuild::default()),
                    Arc::new(ScorerSupply::default()),
                ],
            },
            Faller,
            Position::default(),
            NavigationFlags::COLONIST,
        ));
    }
}
