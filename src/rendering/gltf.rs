use bevy::{
    asset::{AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        query::Without,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::Children,
    pbr::StandardMaterial,
    render::{texture::Image, view::Visibility},
};

use crate::{
    colonists::{get_child_by_name_recursive, AnimClip, AnimState, Animator, ChildMaterials},
    items::image_loader_settings,
};

use super::BasicMaterial;

#[derive(Component)]
pub struct GltfBinding {
    pub mesh_name: String,
    pub armature_name: Option<String>,
    pub texture_path: Option<String>,
}

pub fn setup_gltf_objects(
    mut cmd: Commands,
    q_children: Query<&Children>,
    q_names: Query<&Name>,
    q_bindings: Query<(Entity, &GltfBinding), Without<Animator>>,
    asset_server: Res<AssetServer>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
) {
    for (entity, binding) in q_bindings.iter() {
        if let Some(armature_name) = &binding.armature_name {
            if let Some(armature) =
                get_child_by_name_recursive(&entity, armature_name, &q_names, &q_children)
            {
                let mut e_cmd = cmd.entity(entity);
                e_cmd.insert(Animator {
                    clip: AnimClip::Idle,
                    armature,
                    prev_clip: AnimClip::None,
                    state: AnimState::Completed,
                });
            } else {
                println!("Armature for GLTF object not found. Name={}", armature_name);
            }
        }

        if let Some(mesh) =
            get_child_by_name_recursive(&entity, &binding.mesh_name, &q_names, &q_children)
        {
            let mut mesh_cmd = cmd.entity(mesh);

            let texture: Option<Handle<Image>> = binding
                .texture_path
                .clone()
                .map(|path| asset_server.load_with_settings(path.clone(), image_loader_settings));

            let basic_material = basic_materials.add(BasicMaterial {
                texture,
                is_lit: true,
                ..Default::default()
            });

            mesh_cmd.insert(basic_material.clone());
            mesh_cmd.remove::<Handle<StandardMaterial>>();

            let mut e_cmd = cmd.entity(entity);
            e_cmd.insert(ChildMaterials(basic_material.clone()));
        } else {
            println!(
                "Child mesh for GLTF object not found. Name={}",
                binding.mesh_name
            );
        }

        let mut e_cmd = cmd.entity(entity);
        e_cmd.insert(Visibility::Visible);
    }
}
