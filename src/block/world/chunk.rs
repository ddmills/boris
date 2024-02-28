use bevy::{asset::Handle, ecs::component::Component, render::mesh::Mesh};

#[derive(Component)]
pub struct Chunk {
    pub chunk_idx: u32,
    pub world_x: u32,
    pub world_y: u32,
    pub world_z: u32,
    pub mesh_handle: Handle<Mesh>,
}
