// Bevy 0.14 Material2d (group 2), in the EXISTING native HUD pass.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BattleCompositeUniform {
    rows: array<vec4<f32>, 96>,
    background: vec4<f32>,
    screen_offset: vec4<f32>,
    source_to_view: array<vec4<f32>, 3>,
    view_to_source: array<vec4<f32>, 3>,
    battler_rows: array<vec4<f32>, 2>,
    actor_view_rects: array<vec4<f32>, 2>,
    actor_source_rects: array<vec4<f32>, 2>,
}
@group(2) @binding(0) var<uniform> source: BattleCompositeUniform;
@group(2) @binding(1) var background_texture: texture_2d<f32>;
@group(2) @binding(2) var background_sampler: sampler;
@group(2) @binding(3) var player_rows_texture: texture_2d<f32>;
@group(2) @binding(4) var player_rows_sampler: sampler;
@group(2) @binding(5) var enemy_rows_texture: texture_2d<f32>;
@group(2) @binding(6) var enemy_rows_sampler: sampler;

fn source_point(uv: vec2<f32>) -> vec3<f32> {
    return source.view_to_source[0].xyz * uv.x
        + source.view_to_source[1].xyz * uv.y + source.view_to_source[2].xyz;
}

fn viewport_point(pixel: vec2<f32>) -> vec3<f32> {
    return source.source_to_view[0].xyz * pixel.x
        + source.source_to_view[1].xyz * pixel.y + source.source_to_view[2].xyz;
}

fn background_uv(view_uv: vec2<f32>) -> vec3<f32> {
    // Holding the same zero-scroll source frame leaves the full arena exact.
    if source.screen_offset.z == 0.0 {
        return vec3<f32>(view_uv, 1.0);
    }
    // Undo the SAME perspective mapping used for source OAM anchors. Row
    // identity and register offsets remain source pixels before reprojection.
    let source_homogeneous = source_point(view_uv);
    // Extremely thin windows can put the attack plane horizon inside the
    // viewport. Keep the artistic arena there instead of dividing at a pole
    // or converting a nonfinite LCD row to an integer.
    if source_homogeneous.z <= 0.00001 {
        return vec3<f32>(view_uv, 1.0);
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
            return vec3<f32>(0.0);
        }
    }
    let viewport_homogeneous = viewport_point(pixel);
    if viewport_homogeneous.z <= 0.00001 {
        return vec3<f32>(0.0);
    }
    let uv = viewport_homogeneous.xy / viewport_homogeneous.z;
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return vec3<f32>(0.0);
    }
    // Never clamp an out-of-view sample onto a battler silhouette.
    return vec3<f32>(uv, 1.0);
}

fn actor_source_point(uv: vec2<f32>, actor: u32) -> vec2<f32> {
    let view = source.actor_view_rects[actor];
    let original = source.actor_source_rects[actor];
    return original.xy + (uv - view.xy) / (view.zw - view.xy) * (original.zw - original.xy);
}

fn in_row_band(uv: vec2<f32>, rows: vec4<f32>, actor: u32) -> bool {
    let y = actor_source_point(uv, actor).y;
    return y >= rows.x && y < rows.y;
}

// Row identity is registered to each complete artistic silhouette. The actual
// source-pixel displacement still uses the SAME perspective attack plane as
// native OAM and the arena sampler, preserving source signs and units.
fn actor_background_uv(uv: vec2<f32>, actor: u32) -> vec3<f32> {
    let identity = actor_source_point(uv, actor) - source.screen_offset.xy;
    let row = i32(floor(identity.y));
    var offset = -source.screen_offset.xy;
    if row >= 0 && row < 95 { offset += source.rows[u32(row)].xy; }
    if all(offset == vec2<f32>(0.0)) { return vec3<f32>(uv, 1.0); }
    let original = source_point(uv);
    if original.z <= 0.00001 { return vec3<f32>(uv, 1.0); }
    let sampled = viewport_point(original.xy / original.z + offset);
    if sampled.z <= 0.00001 { return vec3<f32>(0.0); }
    let point = sampled.xy / sampled.z;
    if any(point < vec2<f32>(0.0)) || any(point > vec2<f32>(1.0)) { return vec3<f32>(0.0); }
    return vec3<f32>(point, 1.0);
}

// Transparent camera targets use coverage-premultiplied RGB. Layer the BG
// remainder first; a second native quad carries object-1 strip priority.
fn over(front: vec4<f32>, back: vec4<f32>) -> vec4<f32> {
    return front + back * (1.0 - front.a);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let player = source.battler_rows[0];
    let enemy = source.battler_rows[1];
    if mesh.color.r > 0.5 {
        // Object-1 priority: this native quad sits above later source OAM.
        // No BG sampling, screen shake or SCX/SCY applies to the OBJ copy.
        var strip = vec4<f32>(0.0);
        if player.w != 0.0 && in_row_band(mesh.uv, player, 0u) {
            strip = over(textureSampleLevel(player_rows_texture, player_rows_sampler, mesh.uv, 0.0), strip);
        }
        if enemy.w != 0.0 && in_row_band(mesh.uv, enemy, 1u) {
            strip = over(textureSampleLevel(enemy_rows_texture, enemy_rows_sampler, mesh.uv, 0.0), strip);
        }
        // Material2d uses ordinary alpha blending. Convert the camera's
        // coverage-premultiplied edge colors back to straight alpha once.
        if strip.a > 0.0 { return vec4<f32>(strip.rgb / strip.a, strip.a); }
        return strip;
    }
    // These two source programs scroll Pokémon drawn in the original blank
    // BG. Their registered actor images carry those exact LCD bands below.
    // The added arena has no source pixels to scroll: warping it too invents
    // moving terrain cuts and exposes sky at the edge of the stage.
    let isolated_actors = player.w != 0.0 || enemy.w != 0.0;
    var sampled = vec3<f32>(mesh.uv, 1.0);
    if !isolated_actors { sampled = background_uv(mesh.uv); }
    var color = source.background;
    if sampled.z > 0.0 {
        color = textureSampleLevel(background_texture, background_sampler, sampled.xy, 0.0);
    }
    // Keep ordinary Surf/Psychic/neutral frames on exactly one texture sample.
    if !isolated_actors { return color; }
    // Both actor views are isolated for this pilot; otherwise a sculpture's
    // feet outside the original LCD footprint could enter another side's band.
    let player_sample = actor_background_uv(mesh.uv, 0u);
    let enemy_sample = actor_background_uv(mesh.uv, 1u);
    if player_sample.z > 0.0 {
        let same = all(abs(player_sample.xy - mesh.uv) < vec2<f32>(0.000001));
        if player.w == 0.0 || !in_row_band(player_sample.xy, player, 0u) || (player.z == 0.0 && !same) {
            color = over(textureSampleLevel(player_rows_texture, player_rows_sampler, player_sample.xy, 0.0), color);
        }
    }
    if enemy_sample.z > 0.0 {
        let same = all(abs(enemy_sample.xy - mesh.uv) < vec2<f32>(0.000001));
        if enemy.w == 0.0 || !in_row_band(enemy_sample.xy, enemy, 1u) || (enemy.z == 0.0 && !same) {
            color = over(textureSampleLevel(enemy_rows_texture, enemy_rows_sampler, enemy_sample.xy, 0.0), color);
        }
    }
    return color;
}
