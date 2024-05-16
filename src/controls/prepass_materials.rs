use bevy::{
    asset::{Asset, Assets, Handle},
    ecs::{
        component::Component,
        query::With,
        system::{Local, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    pbr::{AlphaMode, Material},
    reflect::TypePath,
    render::{
        color::Color,
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
    },
    text::Text,
};

#[derive(Debug, Clone, Default, ShaderType)]
pub struct ShowPrepassSettings {
    show_depth: u32,
    show_normals: u32,
    show_motion_vectors: u32,
    padding_1: u32,
    padding_2: u32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PrepassOutputMaterial {
    #[uniform(0)]
    pub settings: ShowPrepassSettings,
}

impl Material for PrepassOutputMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/show_prepass.wgsl".into()
    }

    // This needs to be transparent in order to show the scene behind the mesh
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Every time you press space, it will cycle between transparent, depth and normals view
pub fn toggle_prepass_view(
    mut prepass_view: Local<u32>,
    keycode: Res<ButtonInput<KeyCode>>,
    material_handle: Query<&Handle<PrepassOutputMaterial>>,
    mut materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    if keycode.just_pressed(KeyCode::Space) {
        *prepass_view = (*prepass_view + 1) % 4;

        let handle = material_handle.single();
        let mat = materials.get_mut(handle).unwrap();
        mat.settings.show_depth = (*prepass_view == 1) as u32;
        mat.settings.show_normals = (*prepass_view == 2) as u32;
        mat.settings.show_motion_vectors = (*prepass_view == 3) as u32;
    }
}
