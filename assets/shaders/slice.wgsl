#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> color: vec4<f32>;

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

    var uv: vec2<f32>;

    uv = vec2(mesh.position.x % 1.0, mesh.position.z % 1.0);
    
    var outc = color * textureSample(texture, texture_sampler, uv);

    if (vertex_blue) {
        outc[2] = 0.25;
    }

    if (vertex_mine) {
        outc[0] = 0.05;
    }

    return outc;
}
