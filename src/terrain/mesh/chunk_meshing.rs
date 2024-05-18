use bevy::{
    math::Vec3A,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::VertexFormat,
    },
};
use ndshape::AbstractShape;

use crate::{
    items::image_loader_settings, pack_block, Block, BlockFace, ChunkLayerMesh, ChunkMaterial,
    Neighbor, Terrain, TerrainSlice, TerrainSliceChangeEvent, VertexCornerCount,
};

pub const ATTRIBUTE_BLOCK_PACKED: MeshVertexAttribute =
    MeshVertexAttribute::new("BlockPacked", 9985136798, VertexFormat::Uint32);
pub const ATTRIBUTE_BLOCK_LIGHT: MeshVertexAttribute =
    MeshVertexAttribute::new("BlockLight", 98218357661, VertexFormat::Uint32);

pub fn setup_chunk_meshes(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    terrain: Res<Terrain>,
) {
    let terrain_texture: Handle<Image> =
        asset_server.load_with_settings("textures/comfy.png", image_loader_settings);

    let chunk_material = materials.add(ChunkMaterial {
        color: Color::YELLOW_GREEN,
        texture: terrain_texture,
        texture_count: 8,
    });

    let chunk_container_entity = cmd
        .spawn((Name::new("Chunks"), SpatialBundle::default()))
        .id();

    for chunk_idx in 0..terrain.chunk_count {
        let chunk_pos = terrain.shape.delinearize(chunk_idx);

        let chunk_x = chunk_pos[0] * terrain.chunk_size;
        let chunk_y = chunk_pos[1] * terrain.chunk_size;
        let chunk_z = chunk_pos[2] * terrain.chunk_size;

        for local_y in 0..terrain.chunk_size {
            let global_y = chunk_y + local_y;
            let mesh_data = ChunkLayerMeshData::default();
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

            let x_f32 = chunk_x as f32;
            let y_f32 = global_y as f32;
            let z_f32 = chunk_z as f32;

            let size = terrain.chunk_size as f32 / 2.;

            let mut chunk_cmds = cmd.spawn((
                Name::new(format!("Chunk {} Layer {}", chunk_idx, global_y)),
                ChunkLayerMesh {
                    chunk_idx,
                    mesh_handle: mesh_handle.clone(),
                    global_y,
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
            ));

            chunk_cmds.set_parent(chunk_container_entity);
        }
    }
}

pub fn chunk_meshing(
    mut terrain: ResMut<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    layers: Query<&ChunkLayerMesh>,
    mut ev_terrain_slice: EventWriter<TerrainSliceChangeEvent>,
) {
    let maximum = 16;
    let mut cur = 0;
    let mut update_slice = false;

    for layer in layers.iter() {
        let is_mesh_dirty = terrain.is_layer_dirty(layer.chunk_idx, layer.global_y);

        if !is_mesh_dirty {
            continue;
        }

        cur += 1;
        if cur > maximum {
            break;
        }

        if let Some(mesh) = meshes.get_mut(layer.mesh_handle.clone()) {
            let mesh_data =
                build_chunk_layer_mesh(terrain.as_ref(), layer.chunk_idx, layer.global_y);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
            mesh.insert_attribute(ATTRIBUTE_BLOCK_PACKED, mesh_data.packed);
            mesh.insert_attribute(ATTRIBUTE_BLOCK_LIGHT, mesh_data.light);
            mesh.insert_indices(Indices::U32(mesh_data.indicies));
        }

        terrain.set_layer_mesh_dirty(layer.chunk_idx, layer.global_y, false);

        update_slice = true;
    }

    if update_slice {
        ev_terrain_slice.send(TerrainSliceChangeEvent);
    }
}

pub fn on_slice_changed(
    mut cmd: Commands,
    terrain_slice: Res<TerrainSlice>,
    layers: Query<(Entity, &ChunkLayerMesh)>,
    mut ev_slice_changed: EventReader<TerrainSliceChangeEvent>,
) {
    if ev_slice_changed.is_empty() {
        return;
    }

    ev_slice_changed.clear();

    let y_slice = terrain_slice.get_value();

    for (entity, layer) in layers.iter() {
        if layer.global_y >= y_slice {
            cmd.entity(entity).insert(Visibility::Hidden);
        } else {
            cmd.entity(entity).insert(Visibility::Visible);
        }
    }
}

#[derive(Default)]
struct ChunkLayerMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
    pub packed: Vec<u32>,
    pub light: Vec<u32>,
}

fn build_chunk_layer_mesh(terrain: &Terrain, chunk_idx: u32, global_y: u32) -> ChunkLayerMeshData {
    let mut data = ChunkLayerMeshData::default();
    let mut idx = 0;
    let chunk_offset = terrain.get_chunk_offset(chunk_idx);

    let y = global_y % terrain.chunk_size;

    for x in 0..terrain.chunk_size {
        for z in 0..terrain.chunk_size {
            let wx = chunk_offset[0] + x;
            let wy = chunk_offset[1] + y;
            let wz = chunk_offset[2] + z;
            let block = terrain.get_block(wx, wy, wz);

            if !block.is_rendered() {
                continue;
            }

            let fx = x as f32;
            let fy = 0.;
            let fz = z as f32;

            let neighbors = terrain.get_neighbors_detail(wx, wy, wz);

            if !neighbors[Neighbor::ABOVE.idx()].is_rendered() {
                // add face above
                data.positions.push([fx, fy + 1., fz + 1.]); // behind left
                let f1_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_LEFT.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND_LEFT.idx()],
                );

                data.positions.push([fx, fy + 1., fz]); // forward left
                let f2_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_FORWARD.idx()],
                    neighbors[Neighbor::ABOVE_LEFT.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD_LEFT.idx()],
                );

                data.positions.push([fx + 1., fy + 1., fz]); // forward right
                let f3_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_FORWARD.idx()],
                    neighbors[Neighbor::ABOVE_RIGHT.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy + 1., fz + 1.]); // behind right
                let f4_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_RIGHT.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND_RIGHT.idx()],
                );

                if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                } else {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                }

                data.packed.push(pack_block(block, BlockFace::PosY, f1_ao));
                data.packed.push(pack_block(block, BlockFace::PosY, f2_ao));
                data.packed.push(pack_block(block, BlockFace::PosY, f3_ao));
                data.packed.push(pack_block(block, BlockFace::PosY, f4_ao));

                let n = neighbors[Neighbor::ABOVE.idx()];
                let light = ((n.light & 0xf) | ((n.sunlight << 4) & 0xf0)) as u32;

                data.light.push(light);
                data.light.push(light);
                data.light.push(light);
                data.light.push(light);

                data.normals.push([0., 1., 0.]);
                data.normals.push([0., 1., 0.]);
                data.normals.push([0., 1., 0.]);
                data.normals.push([0., 1., 0.]);

                idx += 4;
            }

            if !neighbors[Neighbor::FORWARD.idx()].is_rendered() {
                // add face in front
                data.positions.push([fx + 1., fy, fz]); // bottom right
                let f1_ao = vert_ao(
                    neighbors[Neighbor::FORWARD_RIGHT.idx()],
                    neighbors[Neighbor::BELOW_FORWARD.idx()],
                    neighbors[Neighbor::BELOW_FORWARD_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy + 1., fz]); // above right
                let f2_ao = vert_ao(
                    neighbors[Neighbor::FORWARD_RIGHT.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD_RIGHT.idx()],
                );

                data.positions.push([fx, fy + 1., fz]); // above left
                let f3_ao = vert_ao(
                    neighbors[Neighbor::FORWARD_LEFT.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD_LEFT.idx()],
                );

                data.positions.push([fx, fy, fz]); // bottom left
                let f4_ao = vert_ao(
                    neighbors[Neighbor::FORWARD_LEFT.idx()],
                    neighbors[Neighbor::BELOW_FORWARD.idx()],
                    neighbors[Neighbor::BELOW_FORWARD_LEFT.idx()],
                );

                if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                } else {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                }

                data.packed.push(pack_block(block, BlockFace::NegZ, f1_ao));
                data.packed.push(pack_block(block, BlockFace::NegZ, f2_ao));
                data.packed.push(pack_block(block, BlockFace::NegZ, f3_ao));
                data.packed.push(pack_block(block, BlockFace::NegZ, f4_ao));

                let n = neighbors[Neighbor::FORWARD.idx()];
                let light = ((n.light & 0xf) | ((n.sunlight << 4) & 0xf0)) as u32;

                data.light.push(light);
                data.light.push(light);
                data.light.push(light);
                data.light.push(light);

                data.normals.push([0., 0., -1.]);
                data.normals.push([0., 0., -1.]);
                data.normals.push([0., 0., -1.]);
                data.normals.push([0., 0., -1.]);

                idx += 4;
            }

            if !neighbors[Neighbor::RIGHT.idx()].is_rendered() {
                // add face right
                data.positions.push([fx + 1., fy, fz + 1.]); // bottom back
                let f1_ao = vert_ao(
                    neighbors[Neighbor::BELOW_RIGHT.idx()],
                    neighbors[Neighbor::BEHIND_RIGHT.idx()],
                    neighbors[Neighbor::BELOW_BEHIND_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy + 1., fz + 1.]); // above back
                let f2_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_RIGHT.idx()],
                    neighbors[Neighbor::BEHIND_RIGHT.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy + 1., fz]); // above forward
                let f3_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_RIGHT.idx()],
                    neighbors[Neighbor::FORWARD_RIGHT.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy, fz]); // bottom forward
                let f4_ao = vert_ao(
                    neighbors[Neighbor::BELOW_RIGHT.idx()],
                    neighbors[Neighbor::FORWARD_RIGHT.idx()],
                    neighbors[Neighbor::BELOW_FORWARD_RIGHT.idx()],
                );

                if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                } else {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                }

                data.packed.push(pack_block(block, BlockFace::PosX, f1_ao));
                data.packed.push(pack_block(block, BlockFace::PosX, f2_ao));
                data.packed.push(pack_block(block, BlockFace::PosX, f3_ao));
                data.packed.push(pack_block(block, BlockFace::PosX, f4_ao));

                let n = neighbors[Neighbor::RIGHT.idx()];
                let light = ((n.light & 0xf) | ((n.sunlight << 4) & 0xf0)) as u32;

                data.light.push(light);
                data.light.push(light);
                data.light.push(light);
                data.light.push(light);

                data.normals.push([1., 0., 0.]);
                data.normals.push([1., 0., 0.]);
                data.normals.push([1., 0., 0.]);
                data.normals.push([1., 0., 0.]);

                idx += 4;
            }

            if !neighbors[Neighbor::BEHIND.idx()].is_rendered() {
                // add face behind
                data.positions.push([fx, fy, fz + 1.]); // bottom left
                let f1_ao = vert_ao(
                    neighbors[Neighbor::BEHIND_LEFT.idx()],
                    neighbors[Neighbor::BELOW_BEHIND.idx()],
                    neighbors[Neighbor::BELOW_BEHIND_LEFT.idx()],
                );

                data.positions.push([fx, fy + 1., fz + 1.]); // above left
                let f2_ao = vert_ao(
                    neighbors[Neighbor::BEHIND_LEFT.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND_LEFT.idx()],
                );

                data.positions.push([fx + 1., fy + 1., fz + 1.]); // above right
                let f3_ao = vert_ao(
                    neighbors[Neighbor::BEHIND_RIGHT.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy, fz + 1.]); // bottom right
                let f4_ao = vert_ao(
                    neighbors[Neighbor::BEHIND_RIGHT.idx()],
                    neighbors[Neighbor::BELOW_BEHIND.idx()],
                    neighbors[Neighbor::BELOW_BEHIND_RIGHT.idx()],
                );

                if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                } else {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                }

                data.packed.push(pack_block(block, BlockFace::PosZ, f1_ao));
                data.packed.push(pack_block(block, BlockFace::PosZ, f2_ao));
                data.packed.push(pack_block(block, BlockFace::PosZ, f3_ao));
                data.packed.push(pack_block(block, BlockFace::PosZ, f4_ao));

                let n = neighbors[Neighbor::BEHIND.idx()];
                let light = ((n.light & 0xf) | ((n.sunlight << 4) & 0xf0)) as u32;

                data.light.push(light);
                data.light.push(light);
                data.light.push(light);
                data.light.push(light);

                data.normals.push([0., 0., 1.]);
                data.normals.push([0., 0., 1.]);
                data.normals.push([0., 0., 1.]);
                data.normals.push([0., 0., 1.]);

                idx += 4;
            }

            if !neighbors[Neighbor::LEFT.idx()].is_rendered() {
                // add face left
                data.positions.push([fx, fy, fz]); // below forward
                let f1_ao = vert_ao(
                    neighbors[Neighbor::BELOW_LEFT.idx()],
                    neighbors[Neighbor::FORWARD_LEFT.idx()],
                    neighbors[Neighbor::BELOW_FORWARD_LEFT.idx()],
                );

                data.positions.push([fx, fy + 1., fz]); // above forward
                let f2_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_LEFT.idx()],
                    neighbors[Neighbor::FORWARD_LEFT.idx()],
                    neighbors[Neighbor::ABOVE_FORWARD_LEFT.idx()],
                );

                data.positions.push([fx, fy + 1., fz + 1.]); // above behind
                let f3_ao = vert_ao(
                    neighbors[Neighbor::ABOVE_LEFT.idx()],
                    neighbors[Neighbor::BEHIND_LEFT.idx()],
                    neighbors[Neighbor::ABOVE_BEHIND_LEFT.idx()],
                );

                data.positions.push([fx, fy, fz + 1.]); // below behind
                let f4_ao = vert_ao(
                    neighbors[Neighbor::BELOW_LEFT.idx()],
                    neighbors[Neighbor::BEHIND_LEFT.idx()],
                    neighbors[Neighbor::BELOW_BEHIND_LEFT.idx()],
                );

                if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                } else {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                }

                data.packed.push(pack_block(block, BlockFace::NegX, f1_ao));
                data.packed.push(pack_block(block, BlockFace::NegX, f2_ao));
                data.packed.push(pack_block(block, BlockFace::NegX, f3_ao));
                data.packed.push(pack_block(block, BlockFace::NegX, f4_ao));

                let n = neighbors[Neighbor::LEFT.idx()];
                let light = ((n.light & 0xf) | ((n.sunlight << 4) & 0xf0)) as u32;

                data.light.push(light);
                data.light.push(light);
                data.light.push(light);
                data.light.push(light);

                data.normals.push([-1., 0., 0.]);
                data.normals.push([-1., 0., 0.]);
                data.normals.push([-1., 0., 0.]);
                data.normals.push([-1., 0., 0.]);

                idx += 4;
            }

            if !neighbors[Neighbor::BELOW.idx()].is_rendered() {
                // add face below
                data.positions.push([fx + 1., fy, fz + 1.]); // behind right
                let f1_ao = vert_ao(
                    neighbors[Neighbor::BELOW_RIGHT.idx()],
                    neighbors[Neighbor::BELOW_BEHIND.idx()],
                    neighbors[Neighbor::BELOW_BEHIND_RIGHT.idx()],
                );

                data.positions.push([fx + 1., fy, fz]); // forward right
                let f2_ao = vert_ao(
                    neighbors[Neighbor::BELOW_FORWARD.idx()],
                    neighbors[Neighbor::BELOW_RIGHT.idx()],
                    neighbors[Neighbor::BELOW_FORWARD_RIGHT.idx()],
                );

                data.positions.push([fx, fy, fz]); // forward left
                let f3_ao = vert_ao(
                    neighbors[Neighbor::BELOW_FORWARD.idx()],
                    neighbors[Neighbor::BELOW_LEFT.idx()],
                    neighbors[Neighbor::BELOW_FORWARD_LEFT.idx()],
                );

                data.positions.push([fx, fy, fz + 1.]); // behind left
                let f4_ao = vert_ao(
                    neighbors[Neighbor::BELOW_LEFT.idx()],
                    neighbors[Neighbor::BELOW_BEHIND.idx()],
                    neighbors[Neighbor::BELOW_BEHIND_LEFT.idx()],
                );

                if f1_ao.bit() + f3_ao.bit() > f2_ao.bit() + f4_ao.bit() {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                } else {
                    data.indicies.push(idx);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);
                }

                data.packed.push(pack_block(block, BlockFace::NegY, f1_ao));
                data.packed.push(pack_block(block, BlockFace::NegY, f2_ao));
                data.packed.push(pack_block(block, BlockFace::NegY, f3_ao));
                data.packed.push(pack_block(block, BlockFace::NegY, f4_ao));

                let n = neighbors[Neighbor::BELOW.idx()];
                let light = ((n.light & 0xf) | ((n.sunlight << 4) & 0xf0)) as u32;

                data.light.push(light);
                data.light.push(light);
                data.light.push(light);
                data.light.push(light);

                data.normals.push([0., -1., 0.]);
                data.normals.push([0., -1., 0.]);
                data.normals.push([0., -1., 0.]);
                data.normals.push([0., -1., 0.]);

                idx += 4;
            }
        }
    }

    data
}

fn vert_ao(side1: Block, side2: Block, corner: Block) -> VertexCornerCount {
    let s1f = side1.is_rendered();
    let s2f = side2.is_rendered();
    let cf = corner.is_rendered();

    if s1f && s2f {
        return VertexCornerCount::Three;
    }

    let mut vao = 0;
    if s1f {
        vao += 1;
    }
    if s2f {
        vao += 1;
    }
    if cf {
        vao += 1;
    }

    VertexCornerCount::from_bit(vao)
}
