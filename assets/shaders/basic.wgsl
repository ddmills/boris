#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world}
#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> sunlight: u32;
@group(2) @binding(3) var<uniform> torchlight: u32;
@group(2) @binding(4) var<uniform> color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let light = f32(max(sunlight, torchlight)) / 15.0 + 0.05;

    return mesh.color * color * textureSample(texture, texture_sampler, mesh.uv) * light;
}
