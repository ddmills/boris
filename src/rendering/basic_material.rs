use bevy::{
    asset::{Asset, Assets, Handle},
    ecs::{
        query::Changed,
        system::{Query, Res, ResMut},
    },
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        color::Color,
        mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderDefVal, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
        texture::Image,
    },
};

use crate::{colonists::ChildMaterials, Position, Terrain};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(BasicMaterialKey)]
pub struct BasicMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[uniform[2]]
    pub color: Color,
    pub enable_vertex_colors: bool,

    pub is_lit: bool,
    #[uniform[3]]
    pub sunlight: u32,
    #[uniform[4]]
    pub torchlight: u32,

    pub enable_slots: bool,
    #[texture(5)]
    #[sampler(6)]
    pub slots_texture: Option<Handle<Image>>,

    #[uniform[7]]
    pub slot_0_color: Color,
    #[uniform[8]]
    pub slot_1_color: Color,
    #[uniform[9]]
    pub slot_2_color: Color,

    #[uniform[11]]
    pub slot_indexes: u32,
}

impl Default for BasicMaterial {
    fn default() -> Self {
        Self {
            texture: None,
            is_lit: true,
            enable_vertex_colors: true,
            enable_slots: true,
            sunlight: 15,
            torchlight: 15,
            color: Color::WHITE,
            slots_texture: None,
            slot_indexes: 0,
            slot_0_color: Color::WHITE,
            slot_1_color: Color::WHITE,
            slot_2_color: Color::WHITE,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct BasicMaterialKey {
    is_lit: bool,
    sunlight: u32,
    torchlight: u32,
    enable_vertex_colors: bool,
    enable_slots: bool,
}

impl From<&BasicMaterial> for BasicMaterialKey {
    fn from(material: &BasicMaterial) -> Self {
        Self {
            is_lit: material.is_lit,
            sunlight: material.sunlight,
            torchlight: material.torchlight,
            enable_vertex_colors: material.enable_vertex_colors,
            enable_slots: material.enable_slots,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotIndex {
    Slot0,
    Slot1,
    Slot2,
}

impl SlotIndex {
    pub fn to_idx(&self) -> usize {
        match self {
            SlotIndex::Slot0 => 0,
            SlotIndex::Slot1 => 1,
            SlotIndex::Slot2 => 2,
        }
    }
}

impl BasicMaterial {
    pub fn from_color(color: Color) -> Self {
        Self {
            texture: None,
            is_lit: true,
            enable_vertex_colors: true,
            enable_slots: true,
            sunlight: 15,
            torchlight: 15,
            color,
            slots_texture: None,
            slot_0_color: Color::WHITE,
            slot_1_color: Color::WHITE,
            slot_2_color: Color::WHITE,
            slot_indexes: 0,
        }
    }

    pub fn with_slot(&mut self, idx: SlotIndex, texture_idx: u8, slot_color: Color) {
        match idx {
            SlotIndex::Slot0 => {
                self.slot_indexes |= texture_idx as u32 & 255;
                self.slot_0_color = slot_color;
            }
            SlotIndex::Slot1 => {
                self.slot_indexes |= (texture_idx as u32 & 255) << 8;
                self.slot_1_color = slot_color;
            }
            SlotIndex::Slot2 => {
                self.slot_indexes |= (texture_idx as u32 & 255) << 16;
                self.slot_2_color = slot_color;
            }
        };

        println!("slot_indexes={}", self.slot_indexes);
    }
}

pub const ATTRIBUTE_SLOTS: MeshVertexAttribute =
    MeshVertexAttribute::new("SlotData", 9911128712, VertexFormat::Float32x4);

impl Material for BasicMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/basic.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/basic.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let mut vertex_attributes = vec![];
        let fragment = descriptor.fragment.as_mut().unwrap();

        let mut defs: Vec<ShaderDefVal> = vec![];

        if key.bind_group_data.is_lit {
            defs.push("IS_LIT".into());
        }

        if layout.contains(Mesh::ATTRIBUTE_POSITION) {
            vertex_attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        }

        if key.bind_group_data.enable_vertex_colors && layout.contains(Mesh::ATTRIBUTE_COLOR) {
            defs.push("VERTEX_COLORS".into());
            vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(5));
        }

        if layout.contains(Mesh::ATTRIBUTE_UV_0) {
            defs.push("VERTEX_UVS".into());
            vertex_attributes.push(Mesh::ATTRIBUTE_UV_0.at_shader_location(2));
        }

        if key.bind_group_data.enable_slots && layout.contains(ATTRIBUTE_SLOTS) {
            defs.push("VERTEX_SLOTS".into());
            vertex_attributes.push(ATTRIBUTE_SLOTS.at_shader_location(3));
        }

        let vertex_buffer_layout = layout.get_layout(&vertex_attributes)?;

        fragment.shader_defs = defs.clone();

        descriptor.vertex.buffers = vec![vertex_buffer_layout];
        descriptor.vertex.shader_defs = defs;
        // ENABLE SKINNING https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.wgsl#L5
        Ok(())
    }
}

pub fn update_basic_material_lighting(
    terrain: Res<Terrain>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    q_moved: Query<(&Position, &Handle<BasicMaterial>), Changed<Position>>,
) {
    for (pos, mat_handle) in q_moved.iter() {
        let Some(material) = basic_materials.get_mut(mat_handle) else {
            continue;
        };

        let block = terrain.get_block_by_idx(pos.chunk_idx, pos.block_idx);

        material.torchlight = block.light as u32;
        material.sunlight = block.sunlight as u32;
    }
}

pub fn update_basic_material_children_lighting(
    terrain: Res<Terrain>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
    q_moved: Query<(&Position, &ChildMaterials), Changed<Position>>,
) {
    for (pos, child) in q_moved.iter() {
        let Some(material) = basic_materials.get_mut(child.0.clone()) else {
            continue;
        };

        let block = terrain.get_block_by_idx(pos.chunk_idx, pos.block_idx);

        material.torchlight = block.light as u32;
        material.sunlight = block.sunlight as u32;
    }
}
