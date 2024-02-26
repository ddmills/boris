use bevy::{
    math::Vec3A,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::VertexFormat,
        texture::{ImageLoaderSettings, ImageSampler},
    },
};
use ndshape::AbstractShape;

use crate::block::{
    self,
    block_face::BlockFace,
    slice::slice::{TerrainSlice, TerrainSliceChanged},
    world::{
        block::Block,
        block_buffer::Neighbor,
        chunk::{Chunk, DirtyChunk},
        terrain::Terrain,
    },
};

use super::chunk_material::{pack_block, ChunkMaterial, ChunkMaterialRes, VertexAo};

pub const ATTRIBUTE_BLOCK_PACKED: MeshVertexAttribute =
    MeshVertexAttribute::new("BlockPacked", 9985136798, VertexFormat::Uint32);
pub const ATTRIBUTE_BLOCK_LIGHT: MeshVertexAttribute =
    MeshVertexAttribute::new("BlockLight", 98218357661, VertexFormat::Uint32);

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

    commands.insert_resource(ChunkMaterialRes {
        handle: chunk_material.clone(),
    });

    for chunk_idx in 0..terrain.chunk_count {
        let chunk_pos = terrain.shape.delinearize(chunk_idx);
        let x = chunk_pos[0] * terrain.chunk_size;
        let y = chunk_pos[1] * terrain.chunk_size;
        let z = chunk_pos[2] * terrain.chunk_size;
        let mesh_data = build_chunk_mesh(&terrain, chunk_idx);
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
        .with_inserted_attribute(ATTRIBUTE_BLOCK_PACKED, mesh_data.packed)
        .with_inserted_attribute(ATTRIBUTE_BLOCK_LIGHT, mesh_data.light)
        .with_inserted_indices(Indices::U32(mesh_data.indicies));

        let mesh_handle = meshes.add(mesh);
        let x_f32 = x as f32;
        let y_f32 = y as f32;
        let z_f32 = z as f32;
        let size = terrain.chunk_size as f32 / 2.;

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
                transform: Transform::from_xyz(x_f32, y_f32, z_f32),
                ..default()
            },
            Aabb {
                center: Vec3A::new(size, size, size),
                half_extents: Vec3A::new(size, size, size),
            },
            // Wireframe,
        ));
    }
}

pub fn process_dirty_chunks(
    mut commands: Commands,
    mut terrain: ResMut<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunks: Query<(&Chunk)>,
) {
    let maximum = 1;
    let mut cur = 0;

    chunks.iter().for_each(|chunk| {
        let is_dirty = terrain.get_chunk_dirty(chunk.chunk_idx);

        if !is_dirty {
            return;
        }

        cur = cur + 1;
        if cur > maximum {
            return;
        }

        if let Some(mesh) = meshes.get_mut(chunk.mesh_handle.clone()) {
            let mesh_data = build_chunk_mesh(terrain.as_ref(), chunk.chunk_idx);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
            mesh.insert_attribute(ATTRIBUTE_BLOCK_PACKED, mesh_data.packed);
            mesh.insert_attribute(ATTRIBUTE_BLOCK_LIGHT, mesh_data.light);
            mesh.insert_indices(Indices::U32(mesh_data.indicies));
        }

        terrain.set_chunk_dirty(chunk.chunk_idx, false);
    });

    // dirty_chunk_query.iter().for_each(|(entity, chunk)| {
    //     cur = cur + 1;
    //     if cur > maximum {
    //         return;
    //     }

    //     if let Some(mesh) = meshes.get_mut(chunk.mesh_handle.clone()) {
    //         let mesh_data = build_chunk_mesh(&terrain, chunk.chunk_idx);
    //         mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    //         mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    //         mesh.insert_attribute(ATTRIBUTE_BLOCK_PACKED, mesh_data.packed);
    //         mesh.insert_attribute(ATTRIBUTE_BLOCK_LIGHT, mesh_data.light);
    //         mesh.insert_indices(Indices::U32(mesh_data.indicies));
    //     }

    //     commands.entity(entity).remove::<DirtyChunk>();
    // });
}

pub fn on_slice_changed(
    terrain_slice: Res<TerrainSlice>,
    chunk_material_res: Res<ChunkMaterialRes>,
    mut ev_slice_changed: EventReader<TerrainSliceChanged>,
    mut terrain_material: ResMut<Assets<ChunkMaterial>>,
) {
    if ev_slice_changed.is_empty() {
        return;
    }

    ev_slice_changed.clear();

    if let Some(material) = terrain_material.get_mut(chunk_material_res.handle.clone()) {
        material.terrain_slice_y = terrain_slice.get_value();
    }
}

#[derive(Default)]
struct ChunkMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
    pub packed: Vec<u32>,
    pub light: Vec<u32>,
}

fn build_chunk_mesh(terrain: &Terrain, chunk_idx: u32) -> ChunkMeshData {
    let mut data = ChunkMeshData::default();
    data.positions = vec![];
    data.normals = vec![];
    data.indicies = vec![];
    data.packed = vec![];
    data.light = vec![];
    let mut idx = 0;
    let chunk_offset = terrain.get_chunk_offset(chunk_idx);

    for x in 0..terrain.chunk_size {
        for y in 0..terrain.chunk_size {
            for z in 0..terrain.chunk_size {
                let wx = chunk_offset[0] + x;
                let wy = chunk_offset[1] + y;
                let wz = chunk_offset[2] + z;
                let block = terrain.get_block(wx, wy, wz);

                if !block.is_filled() {
                    continue;
                }

                let fx = x as f32;
                let fy = y as f32;
                let fz = z as f32;

                let neighbors = terrain.get_neighbors_detail(wx, wy, wz);

                if !neighbors[Neighbor::ABOVE.idx()].block.is_filled() {
                    // add face above
                    data.positions.push([fx, fy + 1., fz + 1.]); // behind left
                    let f1_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_LEFT.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy + 1., fz]); // forward left
                    let f2_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_FORWARD.idx()].block,
                        neighbors[Neighbor::ABOVE_LEFT.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD_LEFT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy + 1., fz]); // forward right
                    let f3_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_FORWARD.idx()].block,
                        neighbors[Neighbor::ABOVE_RIGHT.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy + 1., fz + 1.]); // behind right
                    let f4_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_RIGHT.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND_RIGHT.idx()].block,
                    );

                    if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    } else {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 2);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    }

                    data.packed.push(pack_block(block, BlockFace::PosY, f1_ao));
                    data.packed.push(pack_block(block, BlockFace::PosY, f2_ao));
                    data.packed.push(pack_block(block, BlockFace::PosY, f3_ao));
                    data.packed.push(pack_block(block, BlockFace::PosY, f4_ao));

                    data.light
                        .push(neighbors[Neighbor::ABOVE.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::ABOVE.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::ABOVE.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::ABOVE.idx()].light as u32);

                    data.normals.push([0., 1., 0.]);
                    data.normals.push([0., 1., 0.]);
                    data.normals.push([0., 1., 0.]);
                    data.normals.push([0., 1., 0.]);

                    idx = idx + 4;
                }

                if !neighbors[Neighbor::FORWARD.idx()].block.is_filled() {
                    // add face in front
                    data.positions.push([fx + 1., fy, fz]); // bottom right
                    let f1_ao = vert_ao(
                        neighbors[Neighbor::FORWARD_RIGHT.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy + 1., fz]); // above right
                    let f2_ao = vert_ao(
                        neighbors[Neighbor::FORWARD_RIGHT.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD_RIGHT.idx()].block,
                    );

                    data.positions.push([fx, fy + 1., fz]); // above left
                    let f3_ao = vert_ao(
                        neighbors[Neighbor::FORWARD_LEFT.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy, fz]); // bottom left
                    let f4_ao = vert_ao(
                        neighbors[Neighbor::FORWARD_LEFT.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD_LEFT.idx()].block,
                    );

                    if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    } else {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 2);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    }

                    data.packed.push(pack_block(block, BlockFace::NegZ, f1_ao));
                    data.packed.push(pack_block(block, BlockFace::NegZ, f2_ao));
                    data.packed.push(pack_block(block, BlockFace::NegZ, f3_ao));
                    data.packed.push(pack_block(block, BlockFace::NegZ, f4_ao));

                    data.light
                        .push(neighbors[Neighbor::FORWARD.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::FORWARD.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::FORWARD.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::FORWARD.idx()].light as u32);

                    data.normals.push([0., 0., -1.]);
                    data.normals.push([0., 0., -1.]);
                    data.normals.push([0., 0., -1.]);
                    data.normals.push([0., 0., -1.]);

                    idx = idx + 4;
                }

                if !neighbors[Neighbor::RIGHT.idx()].block.is_filled() {
                    // add face right
                    data.positions.push([fx + 1., fy, fz + 1.]); // bottom back
                    let f1_ao = vert_ao(
                        neighbors[Neighbor::BELOW_RIGHT.idx()].block,
                        neighbors[Neighbor::BEHIND_RIGHT.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy + 1., fz + 1.]); // above back
                    let f2_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_RIGHT.idx()].block,
                        neighbors[Neighbor::BEHIND_RIGHT.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy + 1., fz]); // above forward
                    let f3_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_RIGHT.idx()].block,
                        neighbors[Neighbor::FORWARD_RIGHT.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy, fz]); // bottom forward
                    let f4_ao = vert_ao(
                        neighbors[Neighbor::BELOW_RIGHT.idx()].block,
                        neighbors[Neighbor::FORWARD_RIGHT.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD_RIGHT.idx()].block,
                    );

                    if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    } else {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 2);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    }

                    data.packed.push(pack_block(block, BlockFace::PosX, f1_ao));
                    data.packed.push(pack_block(block, BlockFace::PosX, f2_ao));
                    data.packed.push(pack_block(block, BlockFace::PosX, f3_ao));
                    data.packed.push(pack_block(block, BlockFace::PosX, f4_ao));

                    data.light
                        .push(neighbors[Neighbor::RIGHT.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::RIGHT.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::RIGHT.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::RIGHT.idx()].light as u32);

                    data.normals.push([1., 0., 0.]);
                    data.normals.push([1., 0., 0.]);
                    data.normals.push([1., 0., 0.]);
                    data.normals.push([1., 0., 0.]);

                    idx = idx + 4;
                }

                if !neighbors[Neighbor::BEHIND.idx()].block.is_filled() {
                    // add face behind
                    data.positions.push([fx, fy, fz + 1.]); // bottom left
                    let f1_ao = vert_ao(
                        neighbors[Neighbor::BEHIND_LEFT.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy + 1., fz + 1.]); // above left
                    let f2_ao = vert_ao(
                        neighbors[Neighbor::BEHIND_LEFT.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND_LEFT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy + 1., fz + 1.]); // above right
                    let f3_ao = vert_ao(
                        neighbors[Neighbor::BEHIND_RIGHT.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy, fz + 1.]); // bottom right
                    let f4_ao = vert_ao(
                        neighbors[Neighbor::BEHIND_RIGHT.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND_RIGHT.idx()].block,
                    );

                    if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    } else {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 2);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    }

                    data.packed.push(pack_block(block, BlockFace::PosZ, f1_ao));
                    data.packed.push(pack_block(block, BlockFace::PosZ, f2_ao));
                    data.packed.push(pack_block(block, BlockFace::PosZ, f3_ao));
                    data.packed.push(pack_block(block, BlockFace::PosZ, f4_ao));

                    data.light
                        .push(neighbors[Neighbor::BEHIND.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::BEHIND.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::BEHIND.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::BEHIND.idx()].light as u32);

                    data.normals.push([0., 0., 1.]);
                    data.normals.push([0., 0., 1.]);
                    data.normals.push([0., 0., 1.]);
                    data.normals.push([0., 0., 1.]);

                    idx = idx + 4;
                }

                if !neighbors[Neighbor::LEFT.idx()].block.is_filled() {
                    // add face left
                    data.positions.push([fx, fy, fz]); // below forward
                    let f1_ao = vert_ao(
                        neighbors[Neighbor::BELOW_LEFT.idx()].block,
                        neighbors[Neighbor::FORWARD_LEFT.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy + 1., fz]); // above forward
                    let f2_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_LEFT.idx()].block,
                        neighbors[Neighbor::FORWARD_LEFT.idx()].block,
                        neighbors[Neighbor::ABOVE_FORWARD_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy + 1., fz + 1.]); // above behind
                    let f3_ao = vert_ao(
                        neighbors[Neighbor::ABOVE_LEFT.idx()].block,
                        neighbors[Neighbor::BEHIND_LEFT.idx()].block,
                        neighbors[Neighbor::ABOVE_BEHIND_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy, fz + 1.]); // below behind
                    let f4_ao = vert_ao(
                        neighbors[Neighbor::BELOW_LEFT.idx()].block,
                        neighbors[Neighbor::BEHIND_LEFT.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND_LEFT.idx()].block,
                    );

                    if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    } else {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 2);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    }

                    data.packed.push(pack_block(block, BlockFace::NegX, f1_ao));
                    data.packed.push(pack_block(block, BlockFace::NegX, f2_ao));
                    data.packed.push(pack_block(block, BlockFace::NegX, f3_ao));
                    data.packed.push(pack_block(block, BlockFace::NegX, f4_ao));

                    data.light
                        .push(neighbors[Neighbor::LEFT.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::LEFT.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::LEFT.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::LEFT.idx()].light as u32);

                    data.normals.push([-1., 0., 0.]);
                    data.normals.push([-1., 0., 0.]);
                    data.normals.push([-1., 0., 0.]);
                    data.normals.push([-1., 0., 0.]);

                    idx = idx + 4;
                }

                if !neighbors[Neighbor::BELOW.idx()].block.is_filled() {
                    // add face below
                    data.positions.push([fx + 1., fy, fz + 1.]); // behind right
                    let f1_ao = vert_ao(
                        neighbors[Neighbor::BELOW_RIGHT.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND_RIGHT.idx()].block,
                    );

                    data.positions.push([fx + 1., fy, fz]); // forward right
                    let f2_ao = vert_ao(
                        neighbors[Neighbor::BELOW_FORWARD.idx()].block,
                        neighbors[Neighbor::BELOW_RIGHT.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD_RIGHT.idx()].block,
                    );

                    data.positions.push([fx, fy, fz]); // forward left
                    let f3_ao = vert_ao(
                        neighbors[Neighbor::BELOW_FORWARD.idx()].block,
                        neighbors[Neighbor::BELOW_LEFT.idx()].block,
                        neighbors[Neighbor::BELOW_FORWARD_LEFT.idx()].block,
                    );

                    data.positions.push([fx, fy, fz + 1.]); // behind left
                    let f4_ao = vert_ao(
                        neighbors[Neighbor::BELOW_LEFT.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND.idx()].block,
                        neighbors[Neighbor::BELOW_BEHIND_LEFT.idx()].block,
                    );

                    if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    } else {
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 2);
                        data.indicies.push(idx + 1);
                        data.indicies.push(idx + 0);
                        data.indicies.push(idx + 3);
                        data.indicies.push(idx + 2);
                    }

                    data.packed.push(pack_block(block, BlockFace::NegY, f1_ao));
                    data.packed.push(pack_block(block, BlockFace::NegY, f2_ao));
                    data.packed.push(pack_block(block, BlockFace::NegY, f3_ao));
                    data.packed.push(pack_block(block, BlockFace::NegY, f4_ao));

                    data.light
                        .push(neighbors[Neighbor::BELOW.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::BELOW.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::BELOW.idx()].light as u32);
                    data.light
                        .push(neighbors[Neighbor::BELOW.idx()].light as u32);

                    data.normals.push([0., -1., 0.]);
                    data.normals.push([0., -1., 0.]);
                    data.normals.push([0., -1., 0.]);
                    data.normals.push([0., -1., 0.]);

                    idx = idx + 4;
                }
            }
        }
    }

    return data;
}

fn vert_ao(side1: Block, side2: Block, corner: Block) -> VertexAo {
    let s1f = side1.is_filled();
    let s2f = side2.is_filled();
    let cf = corner.is_filled();

    if s1f && s2f {
        return VertexAo::TouchThree;
    }

    let mut vao = 0;
    if s1f {
        vao = vao + 1;
    }
    if s2f {
        vao = vao + 1;
    }
    if cf {
        vao = vao + 1;
    }

    return VertexAo::from_bit(vao);
}
