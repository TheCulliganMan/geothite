// One global inverse projection for all source OAM slots in the native HUD.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SourceUniform {
    view_to_source: array<vec4<f32>, 3>,
    source_rect: vec4<f32>,
    uv_rect: vec4<f32>,
    viewport: vec4<f32>,
}
@group(2) @binding(0) var<uniform> source: SourceUniform;
@group(2) @binding(1) var source_texture: texture_2d<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Fragment position uses physical top-left viewport pixels. Reconstructing
    // it from each coverage quad's UV would allow floating-point tile seams.
    let screen = mesh.position.xy / source.viewport.xy;
    let point = source.view_to_source[0].xyz * screen.x
        + source.view_to_source[1].xyz * screen.y + source.view_to_source[2].xyz;
    if point.z <= 0.00001 { discard; }
    let original = point.xy / point.z;
    if any(original < source.source_rect.xy) || any(original >= source.source_rect.zw) { discard; }
    // All objects agree on LCD-pixel ownership first. A clipped UV crop then
    // locates that pixel's exact center in the unchanged source texture.
    let pixel_center = floor(original) + vec2<f32>(0.5);
    let uv = source.uv_rect.xy + (pixel_center - source.source_rect.xy)
        / (source.source_rect.zw - source.source_rect.xy)
        * (source.uv_rect.zw - source.uv_rect.xy);
    let dimensions = vec2<i32>(textureDimensions(source_texture));
    let texel = vec2<i32>(floor(uv * vec2<f32>(dimensions)));
    if any(texel < vec2<i32>(0)) || any(texel >= dimensions) { discard; }
    // textureLoad explicitly selects one texel: no linear sampling, fourfold
    // source scale, hardware OAM offsets, BG scroll or screen shake here.
    return textureLoad(source_texture, texel, 0);
}
