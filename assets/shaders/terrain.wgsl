#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> color: vec4<f32>;
@group(2) @binding(3) var<uniform> texture_count: u32;
@group(2) @binding(4) var<uniform> terrain_slice_y: u32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) packed_block: u32,
    @location(3) light: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) packed_block: u32,
    @location(1) position: vec3<f32>,
    @location(2) position_world: vec4<f32>,
    @location(3) ao: f32,
    @location(4) light: f32,
};

@vertex 
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    out.position = vertex.position;
    out.packed_block = vertex.packed_block;
    out.position_world = mesh_position_local_to_world(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0)
    );

    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );

    let vertex_ao = vertex.packed_block >> 11u & 3u;

    switch vertex_ao {
        case 0u: {
            out.ao = 1.0;
        }
        case 1u: {
            out.ao = 0.6;
        }
        case 2u: {
            out.ao = 0.4;
        }
        case 3u: {
            out.ao = 0.2;
        }
        default: {
            out.ao = 0.0;
        }
    }

    let torch = vertex.light & 0xf;
    let sun = (vertex.light >> 4) & 0xf;
    out.light = f32(max(sun, torch)) / 15.0 + 0.1;

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let block_type = mesh.packed_block & 255u;
    let block_face = mesh.packed_block >> 8u & 7u;
    let vertex_ao = mesh.packed_block >> 11u & 3u;
    let vertex_mine = (mesh.packed_block >> 13u & 1u) == 1u;
    let vertex_chop = (mesh.packed_block >> 14u & 1u) == 1u;
    let vertex_blue = (mesh.packed_block >> 15u & 1u) == 1u;

    var uv: vec2<f32>;
    var uv_px_offset: vec2<f32>;

    let ox = f32(block_type % texture_count);
    let oy = f32(block_type / texture_count);

    let ceil_mp_y = ceil(mesh.position_world.y);

    var light: f32;
    let position_local = mesh.position_world % 1.0;

    switch block_face {
        case 0u: { // PosX
            uv_px_offset = vec2(position_local.z, position_local.y);
            uv = vec2(ox + position_local.z, oy + position_local.y);
            light = 0.5;
        }
        case 1u: { // NegX
            uv_px_offset = vec2(position_local.z, position_local.y);
            uv = vec2(ox + position_local.z, oy + position_local.y);
            light = 0.5;
        }
        case 2u: { // PosY
            uv_px_offset = vec2(position_local.x, position_local.z);
            uv = vec2(ox + position_local.x, oy + position_local.z);
            light = 1.0;
        }
        case 3u: { // NegY
            uv_px_offset = vec2(position_local.x, position_local.z);
            uv = vec2(ox + position_local.x, oy + position_local.z);
            light = 1.0;
        }
        case 4u: { // PosZ
            uv_px_offset = vec2(position_local.x, position_local.y);
            uv = vec2(ox + position_local.x, oy + position_local.y);
            light = 0.2;
        }
        case 5u: { // NegZ
            uv_px_offset = vec2(position_local.x, position_local.y);
            uv = vec2(ox + position_local.x, oy + position_local.y);
            light = 0.2;
        }
        default: { // ?
            uv_px_offset = vec2(position_local.x, position_local.z);
            uv = vec2(position_local.x, position_local.z);
            light = 1.0;
        }
    }

    uv = uv / f32(texture_count);
    let tex = textureSample(texture, texture_sampler, uv);
    var outc = light * tex * mesh.ao * mesh.light;
    // var outc = vec4(.5, .5, .5, 1.) * light * mesh.ao * mesh.light;

    outc[3] = 1.0;

    // let gray = 0.299 * outc[0] + 0.587 * outc[1] + 0.114 * outc[2];
    // outc = vec4(gray, gray, gray, 1.);
    
    if (vertex_blue) {
        outc[2] = 0.25;
    }

    if (vertex_mine) {
        outc[0] = outc[0] + .1;
        outc[1] = outc[1] + .1;
        let axe_texture_idx = 60u;
        let axe_ox = f32(axe_texture_idx % texture_count);
        let axe_oy = f32(axe_texture_idx / texture_count);
        var uv2 = (vec2(axe_ox, axe_oy) + uv_px_offset)/ f32(texture_count);
        var axe_c = textureSample(texture, texture_sampler, uv2);

        if (axe_c[3] != 0) {
            outc = outc - vec4(.1, .1, .2, 1.);
        }
    }

    if (vertex_chop) {
        outc[0] = outc[0] + .1;
        outc[1] = outc[1] + .1;
        let axe_texture_idx = 61u;
        let axe_ox = f32(axe_texture_idx % texture_count);
        let axe_oy = f32(axe_texture_idx / texture_count);
        var uv2 = (vec2(axe_ox, axe_oy) + uv_px_offset)/ f32(texture_count);
        var axe_c = textureSample(texture, texture_sampler, uv2);

        if (axe_c[3] != 0) {
            outc = outc - vec4(.1, .1, .2, 1.);
        }
    }

    return outc;
}
