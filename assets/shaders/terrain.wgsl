#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> color: vec4<f32>;
@group(2) @binding(3) var<uniform> texture_count: u32;
@group(2) @binding(4) var<uniform> terrain_slice_y: u32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) packed_block: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) packed_block: u32,
    @location(1) position: vec3<f32>,
    @location(2) position_world: vec4<f32>,
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

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let block_type = mesh.packed_block & 15u;
    let block_face = mesh.packed_block >> 4u & 7u;

    var uv: vec2<f32>;

    let ox = f32(block_type % texture_count);
    let oy = f32(block_type / texture_count);

    // let is_sliced_out = ceil(mesh.position_world.y) > f32(terrain_slice_y);
    let is_sliced_out = floor(mesh.position_world.y) >= f32(terrain_slice_y);

    if (is_sliced_out) {
        discard;
    }

    var shade: f32;

    switch block_face {
        case 0u: { // PosX
            uv = vec2(ox + mesh.position.y % 1.0, oy + mesh.position.z % 1.0);
            shade = 0.5;
        }
        case 1u: { // NegX
            uv = vec2(ox + mesh.position.y % 1.0, oy + mesh.position.z % 1.0);
            shade = 0.5;
        }
        case 2u: { // PosY
            uv = vec2(ox + mesh.position.x % 1.0, oy + mesh.position.z % 1.0);
            shade = 0.0;
        }
        case 3u: { // NegY
            uv = vec2(ox + mesh.position.x % 1.0, oy + mesh.position.z % 1.0);
            shade = 0.9;
        }
        case 4u: { // PosZ
            uv = vec2(ox + mesh.position.x % 1.0, oy + mesh.position.y % 1.0);
            shade = 0.8;
        }
        case 5u: { // NegZ
            uv = vec2(ox + mesh.position.x % 1.0, oy + mesh.position.y % 1.0);
            shade = 0.25;
        }
        default: { // ?
            uv = vec2(mesh.position.x % 1.0, mesh.position.z % 1.0);
            shade = 1.0;
        }
    }

    uv = uv / f32(texture_count);
    var outc = vec4(1.0 - shade) * textureSample(texture, texture_sampler, uv);

    outc[3] = 1.0;


    return outc;
}
