use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Res, ResMut, Resource},
    },
    input::mouse::MouseWheel,
    pbr::{Material, MaterialMeshBundle, MaterialPipeline, MaterialPipelineKey, StandardMaterial},
    prelude::default,
    reflect::TypePath,
    render::{
        color::Color,
        mesh::{Indices, Mesh, MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
        texture::{Image, ImageLoaderSettings, ImageSampler},
    },
    transform::components::Transform,
};

use crate::block::world::terrain::Terrain;

#[derive(Resource)]
pub struct TerrainSlice {
    y: u32,
    min: u32,
    max: u32,
    is_enabled: bool,
    mesh_handle: Handle<Mesh>,
}

impl TerrainSlice {
    pub fn set_value(&mut self, v: i32) -> u32 {
        self.y = v.clamp(self.min as i32, self.max as i32) as u32;
        return self.get_value();
    }

    pub fn get_value(&self) -> u32 {
        return if self.is_enabled { self.y } else { self.max };
    }
}

pub fn setup_terrain_slice(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain: Res<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SliceMaterial>>,
) {
    let settings = |s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest();
    let slice_texture: Handle<Image> =
        asset_server.load_with_settings("textures/slice.png", settings);

    let slice_material = materials.add(SliceMaterial {
        texture: slice_texture,
        color: Color::WHITE,
    });

    let max = terrain.chunk_size * terrain.chunk_count_y;
    let mesh_data = build_slice_mesh(terrain.as_ref(), 16);

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
    .with_inserted_indices(Indices::U32(mesh_data.indicies));

    let mesh_handle = meshes.add(mesh);

    commands.spawn(MaterialMeshBundle {
        mesh: mesh_handle.clone(),
        material: slice_material,
        ..default()
    });

    commands.insert_resource(TerrainSlice {
        y: 16,
        max: max,
        min: 0,
        is_enabled: true,
        mesh_handle: mesh_handle,
    });
}

pub fn update_slice_mesh(
    terrain_slice: Res<TerrainSlice>,
    terrain: Res<Terrain>,
    mut ev_slice_changed: EventReader<TerrainSliceChanged>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if ev_slice_changed.is_empty() {
        return;
    }

    ev_slice_changed.clear();

    if let Some(mesh) = meshes.get_mut(terrain_slice.mesh_handle.clone()) {
        let mesh_buffer = build_slice_mesh(&terrain, terrain_slice.get_value());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_buffer.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_buffer.normals);
        mesh.insert_indices(Indices::U32(mesh_buffer.indicies));
    }
}

#[derive(Event)]
pub struct TerrainSliceChanged {
    prev: u32,
    value: u32,
}

pub fn scroll_events(
    mut scroll_evt: EventReader<MouseWheel>,
    mut terrain_slice: ResMut<TerrainSlice>,
    mut ev_terrain_slice: EventWriter<TerrainSliceChanged>,
) {
    for ev in scroll_evt.read() {
        match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                let cur_slice = terrain_slice.get_value();
                let scroll = ev.y as i32;
                let slice = terrain_slice.y as i32;
                terrain_slice.set_value(slice + scroll);
                ev_terrain_slice.send(TerrainSliceChanged {
                    prev: cur_slice,
                    value: terrain_slice.get_value(),
                });
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {}
        }
    }
}

#[derive(Default)]
struct SliceMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
}

fn build_slice_mesh(terrain: &Terrain, slice_y: u32) -> SliceMeshData {
    let mut data = SliceMeshData::default();
    data.positions = vec![];
    data.normals = vec![];
    data.indicies = vec![];

    let mut idx = 0;

    if slice_y <= 0 {
        return data;
    }

    for x in 0..terrain.world_size_x() {
        for z in 0..terrain.world_size_z() {
            let block = terrain.get_block(x, slice_y - 1, z);

            if !block.is_filled() {
                continue;
            }

            let fx = x as f32;
            let fy = slice_y as f32;
            let fz = z as f32;

            data.positions.push([fx, fy + 0.01, fz]);
            data.positions.push([fx + 1., fy + 0.01, fz]);
            data.positions.push([fx + 1., fy + 0.01, fz + 1.]);
            data.positions.push([fx, fy + 0.01, fz + 1.]);

            data.normals.push([0., 1., 0.]);
            data.normals.push([0., 1., 0.]);
            data.normals.push([0., 1., 0.]);
            data.normals.push([0., 1., 0.]);

            data.indicies.push(idx + 2);
            data.indicies.push(idx + 1);
            data.indicies.push(idx + 0);
            data.indicies.push(idx + 0);
            data.indicies.push(idx + 3);
            data.indicies.push(idx + 2);

            idx = idx + 4;
        }
    }

    return data;
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SliceMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform[2]]
    pub color: Color,
}

impl Material for SliceMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/slice.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/slice.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
