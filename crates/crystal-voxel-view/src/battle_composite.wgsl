// Bevy 0.14 Material2d (group 2), in the EXISTING native HUD pass.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BattleCompositeUniform {
    rows: array<vec4<f32>, 96>,
    background: vec4<f32>,
    screen_offset: vec4<f32>,
    source_to_view: array<vec4<f32>, 3>,
    view_to_source: array<vec4<f32>, 3>,
}
@group(2) @binding(0) var<uniform> source: BattleCompositeUniform;
@group(2) @binding(1) var background_texture: texture_2d<f32>;
@group(2) @binding(2) var background_sampler: sampler;

fn source_point(uv: vec2<f32>) -> vec3<f32> {
    return source.view_to_source[0].xyz * uv.x
        + source.view_to_source[1].xyz * uv.y + source.view_to_source[2].xyz;
}

fn viewport_point(pixel: vec2<f32>) -> vec3<f32> {
    return source.source_to_view[0].xyz * pixel.x
        + source.source_to_view[1].xyz * pixel.y + source.source_to_view[2].xyz;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Holding the same zero-scroll source frame leaves the full arena exact.
    if source.screen_offset.z == 0.0 {
        return textureSampleLevel(background_texture, background_sampler, mesh.uv, 0.0);
    }
    // Undo the SAME perspective mapping used for source OAM anchors. Row
    // identity and register offsets remain source pixels before reprojection.
    let source_homogeneous = source_point(mesh.uv);
    // Extremely thin windows can put the attack plane horizon inside the
    // viewport. Keep the artistic arena there instead of dividing at a pole
    // or converting a nonfinite LCD row to an integer.
    if source_homogeneous.z <= 0.00001 {
        return textureSampleLevel(background_texture, background_sampler, mesh.uv, 0.0);
    }
    let original = source_homogeneous.xy / source_homogeneous.z - source.screen_offset.xy;
    var pixel = original;
    let row = i32(floor(original.y));
    if row >= 0 && row < 95 {
        pixel += source.rows[u32(row)].xy;
    }
    // Inside the original LCD footprint, preserve 256x256 BG tilemap wrap and
    // blank palette regions. The immersive arena also extends beyond that
    // footprint; those added background pixels follow the same projected band
    // without clipping the entire scene to the original LCD rectangle.
    if all(original >= vec2<f32>(0.0)) && original.x < 160.0 && original.y < 144.0 {
        pixel = pixel - floor(pixel / vec2<f32>(256.0)) * vec2<f32>(256.0);
        if pixel.x >= 160.0 || pixel.y >= 144.0 {
            return source.background;
        }
    }
    let viewport_homogeneous = viewport_point(pixel);
    if viewport_homogeneous.z <= 0.00001 {
        return source.background;
    }
    let uv = viewport_homogeneous.xy / viewport_homogeneous.z;
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return source.background;
    }
    // Never clamp an out-of-view sample onto a battler silhouette.
    return textureSampleLevel(background_texture, background_sampler, uv, 0.0);
}
