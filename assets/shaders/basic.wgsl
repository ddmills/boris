#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> color: vec4<f32>;
@group(2) @binding(3) var<uniform> sunlight: u32;
@group(2) @binding(4) var<uniform> torchlight: u32;

@group(2) @binding(5) var slots_texture: texture_2d<f32>;
@group(2) @binding(6) var slots_texture_sampler: sampler;

@group(2) @binding(7) var<uniform> slot_0_color: vec4<f32>;
@group(2) @binding(8) var<uniform> slot_1_color: vec4<f32>;
@group(2) @binding(9) var<uniform> slot_2_color: vec4<f32>;

@group(2) @binding(11) var<uniform> slot_indexes: u32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif

#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif

#ifdef VERTEX_SLOTS
    @location(3) slots: vec4<f32>,
#endif
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,

#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif

#ifdef VERTEX_SLOTS
    @location(3) slots: vec4<f32>,
#endif

#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
}

@vertex 
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    out.position = vertex.position;

    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_SLOTS
    out.slots = vertex.slots;
#endif

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var out = color;

#ifdef IS_LIT
    let light = f32(max(sunlight, torchlight)) / 15.0 + 0.05;
    out = out * light;
#endif

#ifdef VERTEX_COLORS
    let vertex_colors = mesh.color;
    out = out * mesh.color;
#endif

#ifdef VERTEX_UVS
    let texture_color = textureSample(texture, texture_sampler, mesh.uv);
    out = out * texture_color;
#endif

#ifdef VERTEX_SLOTS
#ifdef VERTEX_UVS
    let texture_count = 8u;
    let texture_count_f32 = f32(texture_count);

    let slot_0_idx = slot_indexes & 255u;
    let slot_1_idx = (slot_indexes >> 8u) & 255u;
    let slot_2_idx = (slot_indexes >> 16u) & 255u;
    // let slot_0_idx = 532995u & 255u;
    // let slot_1_idx = (532995u >> 8u) & 255u;
    // let slot_2_idx = (532995u >> 16u) & 255u;

    let slot_0_offset = vec2(f32(slot_0_idx % texture_count), f32(slot_0_idx / texture_count)) / texture_count_f32;
    let slot_1_offset = vec2(f32(slot_1_idx % texture_count), f32(slot_1_idx / texture_count)) / texture_count_f32;
    let slot_2_offset = vec2(f32(slot_2_idx % texture_count), f32(slot_2_idx / texture_count)) / texture_count_f32;

    let uv_clamped = vec2(abs(mesh.uv[0] % 1), abs(mesh.uv[1] % 1)) / texture_count_f32;

    let slot_0_uv = slot_0_offset + uv_clamped;
    let slot_1_uv = slot_1_offset + uv_clamped;
    let slot_2_uv = slot_2_offset + uv_clamped;

    let slot_0_texture_color = textureSample(slots_texture, slots_texture_sampler, slot_0_uv) * slot_0_color;
    let slot_1_texture_color = textureSample(slots_texture, slots_texture_sampler, slot_1_uv) * slot_1_color;
    let slot_2_texture_color = textureSample(slots_texture, slots_texture_sampler, slot_2_uv) * slot_2_color;

    let slot_0_weighted = slot_0_texture_color * mesh.slots[0];
    let slot_1_weighted = slot_1_texture_color * mesh.slots[1];
    let slot_2_weighted = slot_2_texture_color * mesh.slots[2];

    out = out * (slot_0_weighted + slot_1_weighted + slot_2_weighted);
#else
    let slot_0_weighted = slot_0_color * mesh.slots[0];
    let slot_1_weighted = slot_1_color * mesh.slots[1];
    let slot_2_weighted = slot_2_color * mesh.slots[2];

    out = out * (slot_0_weighted + slot_1_weighted + slot_2_weighted);
#endif

    // let uv = mesh.uv / f32(texture_count);
    // out = mesh.slots;
    // out = slot_texture_color;

#endif

    return out;
}
