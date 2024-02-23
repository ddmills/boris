use std::iter;

use bevy::{
    pbr::wireframe::Wireframe,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_asset::RenderAssetUsages,
        render_resource::VertexFormat,
        texture::{ImageLoaderSettings, ImageSampler},
        view::NoFrustumCulling,
    },
};
use ndshape::AbstractShape;

use crate::block::{
    slice::slice::{TerrainSlice, TerrainSliceChanged},
    world::{block::Block, block_buffer::BlockBuffer, terrain::Terrain},
};

use super::chunk_material::ChunkMaterial;

#[derive(Component)]
pub struct Chunk {
    pub chunk_idx: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
    pub mesh_handle: Handle<Mesh>,
}

#[derive(Component)]
pub struct DirtyChunk;

pub const ATTRIBUTE_PACKED_BLOCK: MeshVertexAttribute =
    MeshVertexAttribute::new("PackedBlock", 9985136798, VertexFormat::Uint32);

pub fn setup_chunk_meshes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    terrain: Res<Terrain>,
    slice: Res<TerrainSlice>,
) {
    let settings = |s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest();
    let terrain_texture: Handle<Image> =
        asset_server.load_with_settings("textures/terrain.png", settings);

    let chunk_material = materials.add(ChunkMaterial {
        color: Color::YELLOW_GREEN,
        texture: terrain_texture,
        texture_count: 4,
        terrain_slice_y: slice.get_value(),
    });

    for chunk_idx in 0..terrain.chunk_count {
        if let Some(block_buffer) = terrain.get_chunk(chunk_idx) {
            let chunk_pos = terrain.shape.delinearize(chunk_idx);
            let x = chunk_pos[0] * terrain.chunk_size;
            let y = chunk_pos[1] * terrain.chunk_size;
            let z = chunk_pos[2] * terrain.chunk_size;
            let mesh_data = build_chunk_mesh(block_buffer, slice.get_value());
            let mesh = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
            .with_inserted_attribute(ATTRIBUTE_PACKED_BLOCK, mesh_data.packed)
            .with_inserted_indices(Indices::U32(mesh_data.indicies));

            let mesh_handle = meshes.add(mesh);

            commands.spawn((
                Chunk {
                    chunk_idx: chunk_idx,
                    mesh_handle: mesh_handle.clone(),
                    world_x: x,
                    world_y: y,
                    world_z: z,
                },
                MaterialMeshBundle {
                    mesh: mesh_handle.clone(),
                    material: chunk_material.clone(),
                    transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                    ..default()
                },
                NoFrustumCulling,
                // Wireframe,
            ));
        }
    }
}

pub fn process_dirty_chunks(
    mut commands: Commands,
    terrain: Res<Terrain>,
    terrain_slice: Res<TerrainSlice>,
    mut meshes: ResMut<Assets<Mesh>>,
    dirty_chunk_query: Query<(Entity, &Chunk), With<DirtyChunk>>,
) {
    let maximum = 30;
    let mut cur = 0;
    dirty_chunk_query.iter().for_each(|(entity, chunk)| {
        cur = cur + 1;
        if (cur > maximum) {
            return;
        }
        let block_buffer = terrain.get_chunk(chunk.chunk_idx).unwrap();
        let mesh_data = build_chunk_mesh(&block_buffer, terrain_slice.get_value());
        if let Some(mesh) = meshes.get_mut(chunk.mesh_handle.clone()) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
            mesh.insert_attribute(ATTRIBUTE_PACKED_BLOCK, mesh_data.packed);
            mesh.insert_indices(Indices::U32(mesh_data.indicies));
        }
        commands.entity(entity).remove::<DirtyChunk>();
    });
}

pub fn on_slice_changed(
    mut commands: Commands,
    terrain_slice: Res<TerrainSlice>,
    terrain: Res<Terrain>,
    mut ev_slice_changed: EventReader<TerrainSliceChanged>,
    non_dirty_chunk_query: Query<(Entity, &Chunk), Without<DirtyChunk>>,
) {
    if ev_slice_changed.is_empty() {
        return;
    }

    if let Some(ev) = ev_slice_changed.read().last() {
        let s = terrain_slice.get_value();
        let chunk_indexes = terrain.get_chunks_in_y(terrain_slice.get_value());

        let mv = (s) % terrain.chunk_size;
        println!("index at {} = {}", s, mv);

        non_dirty_chunk_query.iter().for_each(|(entity, chunk)| {
            if chunk_indexes.contains(&chunk.chunk_idx) {
                commands.entity(entity).insert(DirtyChunk);
            }
        });
    };
}

pub fn update_chunk_mesh(
    chunk: &Chunk,
    block_buffer: &BlockBuffer,
    terrain_slice: Res<TerrainSlice>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh_data = build_chunk_mesh(&block_buffer, terrain_slice.get_value());
    if let Some(mesh) = meshes.get_mut(chunk.mesh_handle.clone()) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
        mesh.insert_attribute(ATTRIBUTE_PACKED_BLOCK, mesh_data.packed);
        mesh.insert_indices(Indices::U32(mesh_data.indicies));
    }
}

#[derive(Default)]
struct ChunkMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
    pub packed: Vec<u32>,
}

fn build_chunk_mesh(block_buffer: &BlockBuffer, slice: u32) -> ChunkMeshData {
    let mut data = ChunkMeshData::default();
    data.positions = vec![];
    data.normals = vec![];
    data.indicies = vec![];
    data.packed = vec![];
    let mut idx = 0;

    for block_idx in 0..block_buffer.block_count {
        let block = block_buffer.get_block(block_idx);

        if !block.is_filled() {
            continue;
        }

        let [x, y, z] = block_buffer.get_block_xyz(block_idx);

        let fx = x as f32;
        let fy = y as f32;
        let fz = z as f32;

        if (block_buffer.world_y + y) > slice {
            continue;
        }

        let neighbors = block_buffer.get_immediate_neighbors(x, y, z);
        let is_slice_y = block_buffer.world_y + y == slice;
        let has_neighbor_y = neighbors[0].is_filled();

        if !has_neighbor_y || is_slice_y {
            // add face above
            data.positions.push([fx, fy + 1., fz]);
            data.positions.push([fx + 1., fy + 1., fz]);
            data.positions.push([fx + 1., fy + 1., fz + 1.]);
            data.positions.push([fx, fy + 1., fz + 1.]);

            let face_dir = if is_slice_y && has_neighbor_y {
                FaceDir::Slice
            } else {
                FaceDir::PosY
            };

            data.packed.push(pack_block(block, face_dir));
            data.packed.push(pack_block(block, face_dir));
            data.packed.push(pack_block(block, face_dir));
            data.packed.push(pack_block(block, face_dir));

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

        if !neighbors[1].is_filled() {
            // add face in front
            data.positions.push([fx, fy, fz]);
            data.positions.push([fx, fy + 1., fz]);
            data.positions.push([fx + 1., fy + 1., fz]);
            data.positions.push([fx + 1., fy, fz]);

            data.packed.push(pack_block(block, FaceDir::NegZ));
            data.packed.push(pack_block(block, FaceDir::NegZ));
            data.packed.push(pack_block(block, FaceDir::NegZ));
            data.packed.push(pack_block(block, FaceDir::NegZ));

            data.normals.push([0., 0., -1.]);
            data.normals.push([0., 0., -1.]);
            data.normals.push([0., 0., -1.]);
            data.normals.push([0., 0., -1.]);

            data.indicies.push(idx + 0);
            data.indicies.push(idx + 1);
            data.indicies.push(idx + 2);
            data.indicies.push(idx + 2);
            data.indicies.push(idx + 3);
            data.indicies.push(idx + 0);

            idx = idx + 4;
        }

        if !neighbors[2].is_filled() {
            // add face right
            data.positions.push([fx + 1., fy, fz]);
            data.positions.push([fx + 1., fy, fz + 1.]);
            data.positions.push([fx + 1., fy + 1., fz + 1.]);
            data.positions.push([fx + 1., fy + 1., fz]);

            data.packed.push(pack_block(block, FaceDir::PosX));
            data.packed.push(pack_block(block, FaceDir::PosX));
            data.packed.push(pack_block(block, FaceDir::PosX));
            data.packed.push(pack_block(block, FaceDir::PosX));

            data.normals.push([1., 0., 0.]);
            data.normals.push([1., 0., 0.]);
            data.normals.push([1., 0., 0.]);
            data.normals.push([1., 0., 0.]);

            data.indicies.push(idx + 2);
            data.indicies.push(idx + 1);
            data.indicies.push(idx + 0);
            data.indicies.push(idx + 0);
            data.indicies.push(idx + 3);
            data.indicies.push(idx + 2);

            idx = idx + 4;
        }

        if !neighbors[3].is_filled() {
            // add face behind
            data.positions.push([fx, fy, fz + 1.]);
            data.positions.push([fx, fy + 1., fz + 1.]);
            data.positions.push([fx + 1., fy + 1., fz + 1.]);
            data.positions.push([fx + 1., fy, fz + 1.]);

            data.packed.push(pack_block(block, FaceDir::PosZ));
            data.packed.push(pack_block(block, FaceDir::PosZ));
            data.packed.push(pack_block(block, FaceDir::PosZ));
            data.packed.push(pack_block(block, FaceDir::PosZ));

            data.normals.push([0., 0., 1.]);
            data.normals.push([0., 0., 1.]);
            data.normals.push([0., 0., 1.]);
            data.normals.push([0., 0., 1.]);

            data.indicies.push(idx + 2);
            data.indicies.push(idx + 1);
            data.indicies.push(idx + 0);
            data.indicies.push(idx + 0);
            data.indicies.push(idx + 3);
            data.indicies.push(idx + 2);

            idx = idx + 4;
        }

        if !neighbors[4].is_filled() {
            // add face left
            data.positions.push([fx, fy, fz]);
            data.positions.push([fx, fy, fz + 1.]);
            data.positions.push([fx, fy + 1., fz + 1.]);
            data.positions.push([fx, fy + 1., fz]);

            data.packed.push(pack_block(block, FaceDir::NegX));
            data.packed.push(pack_block(block, FaceDir::NegX));
            data.packed.push(pack_block(block, FaceDir::NegX));
            data.packed.push(pack_block(block, FaceDir::NegX));

            data.normals.push([-1., 0., 0.]);
            data.normals.push([-1., 0., 0.]);
            data.normals.push([-1., 0., 0.]);
            data.normals.push([-1., 0., 0.]);

            data.indicies.push(idx + 0);
            data.indicies.push(idx + 1);
            data.indicies.push(idx + 2);
            data.indicies.push(idx + 2);
            data.indicies.push(idx + 3);
            data.indicies.push(idx + 0);

            idx = idx + 4;
        }

        if !neighbors[5].is_filled() {
            // add face below
            data.positions.push([fx, fy, fz]);
            data.positions.push([fx + 1., fy, fz]);
            data.positions.push([fx + 1., fy, fz + 1.]);
            data.positions.push([fx, fy, fz + 1.]);

            data.packed.push(pack_block(block, FaceDir::NegY));
            data.packed.push(pack_block(block, FaceDir::NegY));
            data.packed.push(pack_block(block, FaceDir::NegY));
            data.packed.push(pack_block(block, FaceDir::NegY));

            data.normals.push([0., -1., 0.]);
            data.normals.push([0., -1., 0.]);
            data.normals.push([0., -1., 0.]);
            data.normals.push([0., -1., 0.]);

            data.indicies.push(idx + 0);
            data.indicies.push(idx + 1);
            data.indicies.push(idx + 2);
            data.indicies.push(idx + 2);
            data.indicies.push(idx + 3);
            data.indicies.push(idx + 0);

            idx = idx + 4;
        }
    }

    return data;
}

fn pack_block(block: Block, dir: FaceDir) -> u32 {
    let t_id = block.texture_idx(); // 0-15
    let f_id = dir.bit(); // 0-7

    return (t_id & 15) | ((f_id & 7) << 4);
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FaceDir {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
    Slice,
}

impl FaceDir {
    pub fn bit(&self) -> u32 {
        match self {
            FaceDir::PosX => 0,
            FaceDir::NegX => 1,
            FaceDir::PosY => 2,
            FaceDir::NegY => 3,
            FaceDir::PosZ => 4,
            FaceDir::NegZ => 5,
            FaceDir::Slice => 6,
        }
    }
}
