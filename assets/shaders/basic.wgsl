#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    view_transformations::position_world_to_clip,
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> color: vec4<f32>;
@group(2) @binding(3) var<uniform> sunlight: u32;
@group(2) @binding(4) var<uniform> torchlight: u32;

@group(2) @binding(5) var slots_texture: texture_2d<f32>;
@group(2) @binding(6) var slots_texture_sampler: sampler;
@group(2) @binding(7) var<uniform> slots_uv_scale: f32;

@group(2) @binding(8) var<uniform> slot_0_color: vec4<f32>;
@group(2) @binding(9) var<uniform> slot_1_color: vec4<f32>;
@group(2) @binding(10) var<uniform> slot_2_color: vec4<f32>;

@group(2) @binding(11) var<uniform> slot_indexes: u32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
#endif
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
#ifdef VERTEX_SLOTS
    @location(8) slots: vec4<f32>,
#endif
};

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
#ifdef VERTEX_SLOTS
    @location(8) slots: vec4<f32>,
#endif
}


@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var model = mesh_functions::get_model_matrix(vertex_no_morph.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_SLOTS
    out.slots = vertex.slots;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, model[3]);
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

    let slot_0_offset = vec2(f32(slot_0_idx % texture_count), f32(slot_0_idx / texture_count)) / texture_count_f32;
    let slot_1_offset = vec2(f32(slot_1_idx % texture_count), f32(slot_1_idx / texture_count)) / texture_count_f32;
    let slot_2_offset = vec2(f32(slot_2_idx % texture_count), f32(slot_2_idx / texture_count)) / texture_count_f32;

    let uv_x = abs((mesh.uv[0] * slots_uv_scale) % 1);
    let uv_y = abs((mesh.uv[1] * slots_uv_scale) % 1);

    let uv_clamped = vec2(uv_x, uv_y) / texture_count_f32;

    let slot_0_uv = slot_0_offset + uv_clamped;
    let slot_1_uv = slot_1_offset + uv_clamped;
    let slot_2_uv = slot_2_offset + uv_clamped;

    let slot_0_texture_color = textureSample(slots_texture, slots_texture_sampler, slot_0_uv) * slot_0_color;
    let slot_1_texture_color = textureSample(slots_texture, slots_texture_sampler, slot_1_uv) * slot_1_color;
    let slot_2_texture_color = textureSample(slots_texture, slots_texture_sampler, slot_2_uv) * slot_2_color;

    let slot_0_weighted = slot_0_texture_color * mesh.slots[0];
    let slot_1_weighted = slot_1_texture_color * mesh.slots[1];
    let slot_2_weighted = slot_2_texture_color * mesh.slots[2];

    if (mesh.slots[0] > 0.1 || mesh.slots[1] > 0.1 || mesh.slots[2] > 0.1) {
        out = out * (slot_0_weighted + slot_1_weighted + slot_2_weighted);
    }

#else
    let slot_0_weighted = slot_0_color * mesh.slots[0];
    let slot_1_weighted = slot_1_color * mesh.slots[1];
    let slot_2_weighted = slot_2_color * mesh.slots[2];

    if (mesh.slots[0] > 0.1 || mesh.slots[1] > 0.1 || mesh.slots[2] > 0.1) {
        out = out * (slot_0_weighted + slot_1_weighted + slot_2_weighted);
    }
#endif
#endif

    return out;
}
