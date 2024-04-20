#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> color: vec4<f32>;
@group(2) @binding(3) var<uniform> texture_count: u32;
@group(2) @binding(4) var<uniform> texture_idx: u32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) packed_block: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) packed_block: u32,
};

@vertex 
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    out.position = vertex.position;
    out.packed_block = vertex.packed_block;
    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let vertex_mine = (mesh.packed_block >> 9u & 1u) == 1u;
    let vertex_blue = (mesh.packed_block >> 10u & 1u) == 1u;

    let ox = f32(texture_idx % texture_count);
    let oy = f32(texture_idx / texture_count);

    var uv: vec2<f32>;

    uv = vec2(ox + mesh.position.x % 1.0, oy + mesh.position.z % 1.0) / f32(texture_count);

    var outc = color * textureSample(texture, texture_sampler, uv);

    if (vertex_blue) {
        outc[2] = 0.25;
    }

    if (vertex_mine) {
        outc[0] = 0.05;
        let axe_texture_idx = 60u;
        let axe_ox = f32(axe_texture_idx % texture_count);
        let axe_oy = f32(axe_texture_idx / texture_count);
        var uv2 = vec2(axe_ox + mesh.position.x % 1.0, axe_oy + mesh.position.z % 1.0) / f32(texture_count);
        var axe_c = textureSample(texture, texture_sampler, uv2);

        if (axe_c[3] != 0) {
            // outc = axe_c;
            outc = vec4(.7, .7, .7, 1.);
        }
    }

    return outc;
}
