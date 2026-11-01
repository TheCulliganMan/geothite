//! Source-clocked presentation heightfield, inspired by Evan Wallace's water
//! demo. Heights never move gameplay geometry. The retained shore image keeps
//! its RG distances and carries current/previous optical heights in BA.
use bevy::{prelude::*, render::render_resource::TextureFormat};
use crystal_render_api::VisualWorldFrame;

const SAMPLES: usize = 4;
const MAX_CELLS: usize = 512 * 512;
pub(super) const HEIGHT_RANGE: f32 = 0.12;
const HEIGHT_LIMIT: f32 = HEIGHT_RANGE * 0.45;
const PHYSICS_PHASE: f32 = std::f32::consts::TAU / 6.0;

#[derive(Default)]
pub(super) struct WaterRipples {
    field: Option<Field>,
}

#[derive(Clone, Debug, PartialEq)]
struct Field {
    map: std::sync::Arc<str>,
    origin: IVec2,
    width: usize,
    height: usize,
    wet: Vec<bool>,
    stencil: Vec<(usize, [usize; 4])>,
    heights: Vec<f32>,
    previous: Vec<f32>,
    velocity: Vec<f32>,
    scratch: Vec<f32>,
    contact: Option<Vec3>,
    tick: u32,
    phase_remainder: f32,
}

impl WaterRipples {
    /// Advance only when an authenticated water frame advances. A zero or
    /// invalid step can register/reproject a new grid, but cannot create motion.
    pub(super) fn update(
        &mut self,
        frame: &VisualWorldFrame,
        contact: Vec4,
        steps: Vec2,
        shore: &mut Image,
    ) -> bool {
        let width = shore.texture_descriptor.size.width as usize;
        let height = shore.texture_descriptor.size.height as usize;
        let count = width.checked_mul(height);
        let valid = crate::cave_water::outdoor(&frame.map_id)
            && crate::water_surface::style(frame).w > 0.25
            && shore.texture_descriptor.format == TextureFormat::Rgba8Unorm
            && width > 0
            && height > 0
            && Some(width) == (frame.grid_size.x as usize).checked_mul(SAMPLES)
            && Some(height) == (frame.grid_size.y as usize).checked_mul(SAMPLES)
            && count.is_some_and(|n| n <= MAX_CELLS)
            && shore
                .data
                .as_ref()
                .is_some_and(|data| Some(data.len()) == count.and_then(|n| n.checked_mul(4)));
        if !valid {
            self.field = None;
            return false;
        }
        let _update = crate::loading_trace::span("water.ripples.update", || {
            serde_json::json!({"map":frame.map_id.as_ref(),"grid":[width,height],
                "source_step":steps.x,"contact":contact.to_array()})
        });
        let count = count.unwrap();
        let mut known = vec![false; count / (SAMPLES * SAMPLES)];
        for tile in &frame.tiles {
            if tile.column < frame.grid_size.x && tile.row < frame.grid_size.y {
                known[tile.row as usize * frame.grid_size.x as usize + tile.column as usize] = true;
            }
        }
        let data = shore.data.as_mut().unwrap();
        let wet = shore_wet_mask(width, frame.grid_size.x as usize, &known, data);
        let needs_grid = self.field.as_ref().is_none_or(|field| {
            field.map != frame.map_id
                || field.origin != frame.grid_origin
                || field.width != width
                || field.height != height
        });
        if needs_grid {
            let old = self.field.take();
            self.field = Some(Field::new(frame, width, height, wet, old.as_ref()));
        } else {
            let field = self.field.as_mut().unwrap();
            if field.wet != wet {
                field.stencil = wet_stencil(width, height, &wet);
                field.wet = wet;
            }
            for i in 0..count {
                if !field.wet[i] {
                    field.heights[i] = 0.0;
                    field.previous[i] = 0.0;
                    field.velocity[i] = 0.0;
                    field.scratch[i] = 0.0;
                }
            }
        }
        let field = self.field.as_mut().unwrap();
        if steps.x.is_finite() && steps.x > 0.0 {
            // Native water drawings have different frame counts and encode
            // their phases in bytes. Carry the fractional phase instead of
            // rounding every callback to a supposed three-frame source cycle.
            let budget = field.phase_remainder + steps.x.min(std::f32::consts::TAU);
            let advances = (budget / PHYSICS_PHASE + 0.00001).floor() as usize;
            field.phase_remainder = (budget - advances as f32 * PHYSICS_PHASE).max(0.0);
            field.previous.copy_from_slice(&field.heights);
            field.move_contact(contact);
            for _ in 0..advances {
                if field.tick & 1 == 0 {
                    field.ambient_drop();
                }
                // The CFL coefficient and damping are fixed per source step,
                // independent of rendering speed and callback batching.
                field.step();
                field.tick = field.tick.wrapping_add(1);
            }
        }
        let _publish = crate::loading_trace::span("water.ripples.publish", || {
            serde_json::json!({"tick":field.tick,"grid":[width,height],
                "height_peak":field.heights.iter().fold(0.0_f32, |peak,h| peak.max(h.abs()))})
        });
        for (i, pixel) in data.chunks_exact_mut(4).enumerate() {
            let encode =
                |value: f32| ((value / HEIGHT_RANGE + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
            pixel[2] = encode(field.heights[i]);
            pixel[3] = encode(field.previous[i]);
        }
        true
    }
}

impl Field {
    fn new(
        frame: &VisualWorldFrame,
        width: usize,
        height: usize,
        wet: Vec<bool>,
        old: Option<&Self>,
    ) -> Self {
        let count = width * height;
        let mut field = Self {
            map: frame.map_id.clone(),
            origin: frame.grid_origin,
            width,
            height,
            stencil: wet_stencil(width, height, &wet),
            wet,
            heights: vec![0.0; count],
            previous: vec![0.0; count],
            velocity: vec![0.0; count],
            scratch: vec![0.0; count],
            contact: None,
            tick: 0,
            phase_remainder: 0.0,
        };
        if let Some(old) = old.filter(|old| old.map == frame.map_id) {
            let shift = (frame.grid_origin - old.origin) * SAMPLES as i32;
            for y in 0..height {
                for x in 0..width {
                    let (ox, oy) = (x as i32 + shift.x, y as i32 + shift.y);
                    if ox < 0 || oy < 0 || ox >= old.width as i32 || oy >= old.height as i32 {
                        continue;
                    }
                    let i = y * width + x;
                    let previous = oy as usize * old.width + ox as usize;
                    if field.wet[i] && old.wet[previous] {
                        field.heights[i] = old.heights[previous];
                        field.previous[i] = old.previous[previous];
                        field.velocity[i] = old.velocity[previous];
                    }
                }
            }
            field.tick = old.tick;
            field.phase_remainder = old.phase_remainder;
            field.contact = old.contact;
        }
        field
    }

    fn add_drop(&mut self, point: Vec2, radius: f32, strength: f32) {
        let p = (point - self.origin.as_vec2()) * SAMPLES as f32 - Vec2::splat(0.5);
        let r = radius * SAMPLES as f32;
        let low = (p - Vec2::splat(r)).floor().max(Vec2::ZERO).as_uvec2();
        let high = (p + Vec2::splat(r))
            .ceil()
            .min(Vec2::new(self.width as f32 - 1.0, self.height as f32 - 1.0));
        if high.min_element() < 0.0 {
            return;
        }
        for y in low.y as usize..=high.y as usize {
            for x in low.x as usize..=high.x as usize {
                let i = y * self.width + x;
                let distance = (Vec2::new(x as f32, y as f32) - p).length() / r;
                if self.wet[i] && distance < 1.0 {
                    let bump = 0.5 + 0.5 * (distance * std::f32::consts::PI).cos();
                    self.heights[i] =
                        (self.heights[i] + bump * strength).clamp(-HEIGHT_LIMIT, HEIGHT_LIMIT);
                }
            }
        }
    }

    fn move_contact(&mut self, contact: Vec4) {
        let next =
            (contact.is_finite() && contact.w > 0.0 && contact.z > 0.0).then_some(contact.xyz());
        if self
            .contact
            .zip(next)
            .is_some_and(|(old, new)| old.distance(new) < 0.01)
            || (self.contact.is_none() && next.is_none())
        {
            return;
        }
        // Moving a body restores the old displaced volume and removes the
        // new one. A stationary swimmer cannot emit an endless painted ring.
        if let Some(old) = self.contact {
            self.add_drop(old.xy(), old.z * 0.9, 0.025);
        }
        if let Some(new) = next {
            self.add_drop(new.xy(), new.z * 0.9, -0.025);
        }
        self.contact = next;
    }

    fn ambient_drop(&mut self) {
        // Small, independently placed disturbances keep sheltered water alive.
        // This sequence advances with source animation, never display frames.
        for attempt in 0..4 {
            let seed = self
                .tick
                .wrapping_add(1)
                .wrapping_mul(17)
                .wrapping_add(attempt * 7919);
            let x = hash(seed) as usize % self.width;
            let y = hash(seed ^ 0x9e3779b9) as usize % self.height;
            if !self.wet[y * self.width + x] {
                continue;
            }
            let point =
                self.origin.as_vec2() + Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / SAMPLES as f32;
            let sign = if hash(seed ^ 0xa511e9b3) & 1 == 0 {
                1.0
            } else {
                -1.0
            };
            self.add_drop(point, 0.45, sign * 0.008);
            break;
        }
    }

    fn step(&mut self) {
        // Dry samples stay zero. Resolve the no-flux bank neighbors only
        // when the wet mask changes, retaining the original addition order.
        for &(i, [left, right, above, below]) in &self.stencil {
            let height = self.heights[i];
            let laplacian = self.heights[left]
                + self.heights[right]
                + self.heights[above]
                + self.heights[below]
                - 4.0 * height;
            self.velocity[i] = (self.velocity[i] + laplacian * 0.32) * 0.985;
            self.scratch[i] = (height + self.velocity[i]).clamp(-HEIGHT_LIMIT, HEIGHT_LIMIT);
        }
        std::mem::swap(&mut self.heights, &mut self.scratch);
    }
}

// Expand each known source tile into its four-by-four sample block. This
// avoids dividing every sample index by the dynamic texture width.
fn shore_wet_mask(width: usize, grid_width: usize, known: &[bool], data: &[u8]) -> Vec<bool> {
    let mut wet = vec![false; data.len() / 4];
    for (row, tiles) in known.chunks_exact(grid_width).enumerate() {
        for (column, &available) in tiles.iter().enumerate() {
            if !available { continue; }
            for sample_row in 0..SAMPLES {
                let start = (row * SAMPLES + sample_row) * width + column * SAMPLES;
                for sample in start..start + SAMPLES {
                    // Signed distances include rock feet and raised piers.
                    wet[sample] = data[sample * 4] > 127;
                }
            }
        }
    }
    wet
}

/// A dry neighbor reflects the current sample; an outer edge clamps to it.
fn wet_stencil(width: usize, height: usize, wet: &[bool]) -> Vec<(usize, [usize; 4])> {
    let mut stencil = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            if !wet[i] { continue; }
            let neighbors = [
                y * width + x.saturating_sub(1),
                y * width + (x + 1).min(width - 1),
                y.saturating_sub(1) * width + x,
                (y + 1).min(height - 1) * width + x,
            ].map(|j| if wet[j] { j } else { i });
            stencil.push((i, neighbors));
        }
    }
    stencil
}

fn hash(mut seed: u32) -> u32 {
    seed ^= seed >> 16;
    seed = seed.wrapping_mul(0x7feb352d);
    seed ^= seed >> 15;
    seed = seed.wrapping_mul(0x846ca68b);
    seed ^ (seed >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileSource};

    fn reference_step(field: &mut Field) {
        for y in 0..field.height {
            for x in 0..field.width {
                let i = y * field.width + x;
                if !field.wet[i] {
                    field.scratch[i] = 0.0;
                    continue;
                }
                let height = field.heights[i];
                let neighbor = |nx: usize, ny: usize| {
                    let j = ny * field.width + nx;
                    if field.wet[j] { field.heights[j] } else { height }
                };
                // No-flux banks reflect incoming waves; a dry strip cannot
                // transmit ripples between disconnected ponds.
                let laplacian = neighbor(x.saturating_sub(1), y)
                    + neighbor((x + 1).min(field.width - 1), y)
                    + neighbor(x, y.saturating_sub(1))
                    + neighbor(x, (y + 1).min(field.height - 1))
                    - 4.0 * height;
                field.velocity[i] = (field.velocity[i] + laplacian * 0.32) * 0.985;
                field.scratch[i] = (height + field.velocity[i]).clamp(-HEIGHT_LIMIT, HEIGHT_LIMIT);
            }
        }
        std::mem::swap(&mut field.heights, &mut field.scratch);
    }

    #[test]
    fn tile_block_mask_matches_sample_scan_with_missing_source() {
        for (columns, rows) in [(1, 1), (3, 7), (128, 128)] {
            let width = columns * SAMPLES;
            let known = (0..columns * rows).map(|i|hash(i as u32) % 3 != 0).collect::<Vec<_>>();
            let data = (0..width * rows * SAMPLES * 4)
                .map(|i|hash(i as u32) as u8).collect::<Vec<_>>();
            let expected = (0..data.len()/4).map(|i| {
                let tile = (i / width / SAMPLES) * columns + i % width / SAMPLES;
                known[tile] && data[i*4] > 127
            }).collect::<Vec<_>>();
            assert_eq!(shore_wet_mask(width, columns, &known, &data), expected);
        }
    }

    #[test]
    fn wet_stencil_matches_full_grid_physics_bit_for_bit() {
        let frame = frame();
        for (width, height) in [(1, 1), (1, 17), (19, 1), (48, 48), (127, 93)] {
            for pattern in 0..4 {
                let wet = (0..width * height).map(|i| match pattern {
                    0 => true,
                    1 => false,
                    2 => hash(i as u32) % 7 < 2,
                    _ => i % width < width / 3,
                }).collect();
                let mut fast = Field::new(&frame, width, height, wet, None);
                for i in 0..width * height {
                    if fast.wet[i] {
                        fast.heights[i] = (hash(i as u32) % 101) as f32 * 0.0001 - 0.005;
                        fast.velocity[i] = (hash(i as u32 ^ 17) % 31) as f32 * 0.00001;
                    }
                }
                let mut original = fast.clone();
                for _ in 0..80 {
                    fast.step();
                    reference_step(&mut original);
                    assert_eq!(fast.heights.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),
                        original.heights.iter().map(|v|v.to_bits()).collect::<Vec<_>>());
                    assert_eq!(fast.velocity, original.velocity);
                    assert_eq!(fast.scratch, original.scratch);
                }
            }
        }
    }

    #[test]
    fn changing_shore_mask_rebuilds_stencil_and_clears_retired_water() {
        let frame = frame();
        let mut image = crate::water_surface::shoreline(&frame).unwrap();
        let mut ripples = WaterRipples::default();
        for pixel in image.data.as_mut().unwrap().chunks_exact_mut(4) { pixel[0] = 255; }
        ripples.update(&frame, Vec4::new(5.,5.,1.,1.), steps(), &mut image);
        for (i,pixel) in image.data.as_mut().unwrap().chunks_exact_mut(4).enumerate() {
            if i % 48 < 24 { pixel[0] = 0; }
        }
        ripples.update(&frame, Vec4::ZERO, Vec2::ZERO, &mut image);
        let field = ripples.field.as_ref().unwrap();
        assert_eq!(field.stencil.len(),48*24);
        for i in 0..48*48 {
            if i % 48 < 24 {
                assert_eq!([field.heights[i],field.previous[i],field.velocity[i],field.scratch[i]], [0.;4]);
            }
        }
        let mut original = field.clone();
        ripples.update(&frame, Vec4::ZERO, Vec2::new(PHYSICS_PHASE,0.), &mut image);
        original.previous.copy_from_slice(&original.heights);
        original.move_contact(Vec4::ZERO);
        if original.tick & 1 == 0 { original.ambient_drop(); }
        reference_step(&mut original);
        let field = ripples.field.as_ref().unwrap();
        assert_eq!(field.heights,original.heights);
        assert_eq!(field.velocity,original.velocity);
    }

    fn frame() -> VisualWorldFrame {
        let mut frame = VisualWorldFrame {
            map_id: "Route6".into(),
            grid_size: UVec2::splat(12),
            tile_size: Vec2::splat(8.0),
            ..default()
        };
        frame.tiles = (0..12)
            .flat_map(|row| {
                (0..12).map(move |column| VisualTile {
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: "johto".into(),
                        metatile_id: 0x35,
                        subtile_column: 0,
                        subtile_row: 0,
                        tile_index: 0x14,
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                })
            })
            .collect();
        frame
    }

    fn steps() -> Vec2 {
        Vec2::new(std::f32::consts::TAU / 3.0, 0.0)
    }

    #[test]
    fn source_clock_holds_exactly_and_shore_distances_stay_byte_identical() {
        let frame = frame();
        let mut image = crate::water_surface::shoreline(&frame).unwrap();
        let distances = image
            .data
            .as_ref()
            .unwrap()
            .chunks_exact(4)
            .map(|p| [p[0], p[1]])
            .collect::<Vec<_>>();
        let mut ripples = WaterRipples::default();
        let contact = Vec4::new(5.0, 5.0, 0.8, 1.0);
        assert!(ripples.update(&frame, contact, steps(), &mut image));
        let held = ripples.field.clone();
        let bytes = image.data.clone();
        for step in [Vec2::ZERO, Vec2::new(f32::NAN, 0.0), Vec2::new(-1.0, 0.0)] {
            assert!(ripples.update(&frame, contact + Vec4::X, step, &mut image));
            assert_eq!(ripples.field, held);
            assert_eq!(image.data, bytes);
        }
        assert_eq!(
            image
                .data
                .as_ref()
                .unwrap()
                .chunks_exact(4)
                .map(|p| [p[0], p[1]])
                .collect::<Vec<_>>(),
            distances
        );
    }

    #[test]
    fn a_displaced_body_launches_waves_that_propagate_and_decay() {
        let frame = frame();
        let mut field = Field::new(&frame, 48, 48, vec![true; 48 * 48], None);
        field.move_contact(Vec4::new(6.0, 6.0, 0.7, 1.0));
        let initial = field.heights.clone();
        let initial_energy = initial.iter().map(|h| h * h).sum::<f32>();
        for _ in 0..12 {
            field.step();
        }
        assert!(
            initial
                .iter()
                .zip(&field.heights)
                .any(|(a, b)| *a == 0.0 && b.abs() > 0.0001)
        );
        let unchanged = field.heights.clone();
        field.move_contact(Vec4::new(6.0, 6.0, 0.7, 1.0));
        assert_eq!(
            field.heights, unchanged,
            "a stationary body adds no further energy"
        );
        for _ in 0..1200 {
            field.step();
        }
        assert!(
            field
                .heights
                .iter()
                .all(|v| v.is_finite() && v.abs() <= HEIGHT_LIMIT)
        );
        // A displaced volume has a persistent mean; only its wave energy decays.
        let mean = field.heights.iter().sum::<f32>() / field.heights.len() as f32;
        let energy = field
            .heights
            .iter()
            .map(|h| (h - mean).powi(2))
            .sum::<f32>();
        assert!(
            energy < initial_energy * 0.02,
            "damped wave energy: {energy}"
        );
    }

    #[test]
    fn banks_block_waves_and_streaming_preserves_world_registered_height_and_velocity() {
        let mut frame = frame();
        let mut wet = vec![true; 48 * 48];
        for y in 0..48 {
            wet[y * 48 + 24] = false;
        }
        let mut field = Field::new(&frame, 48, 48, wet, None);
        field.add_drop(Vec2::new(4.0, 6.0), 0.7, 0.025);
        for _ in 0..80 {
            field.step();
        }
        field.previous = field.heights.clone();
        assert!((0..48).all(|y| (24..48).all(|x| field.heights[y * 48 + x] == 0.0)));
        frame.grid_origin = IVec2::new(1, -1);
        let shifted = Field::new(&frame, 48, 48, vec![true; 48 * 48], Some(&field));
        for y in 4..48 {
            for x in 0..44 {
                let (new, old) = (y * 48 + x, (y - 4) * 48 + x + 4);
                assert_eq!(shifted.heights[new], field.heights[old]);
                assert_eq!(shifted.previous[new], field.previous[old]);
                assert_eq!(shifted.velocity[new], field.velocity[old]);
            }
        }
        frame.map_id = "OlivinePort".into();
        let different_map = Field::new(&frame, 48, 48, vec![true; 48 * 48], Some(&field));
        assert!(different_map.heights.iter().all(|h| *h == 0.0));
    }

    #[test]
    fn moving_and_leaving_water_restore_displaced_volume_and_batching_keeps_physics_identical() {
        let frame = frame();
        let mut field = Field::new(&frame, 48, 48, vec![true; 48 * 48], None);
        field.move_contact(Vec4::new(5.0, 6.0, 0.7, 1.0));
        let volume = field.heights.iter().sum::<f32>();
        field.move_contact(Vec4::new(7.0, 6.0, 0.7, 1.0));
        assert_eq!(field.heights[24 * 48 + 20], 0.0, "old foot volume restored");
        assert!(
            field.heights[24 * 48 + 28] < 0.0,
            "new foot displaces water"
        );
        assert!((field.heights.iter().sum::<f32>() - volume).abs() < 0.00001);
        field.move_contact(Vec4::ZERO);
        assert!(field.heights.iter().all(|h| h.abs() < 0.000001));

        let mut image0 = crate::water_surface::shoreline(&frame).unwrap();
        let mut image1 = image0.clone();
        let mut single = WaterRipples::default();
        let mut batched = WaterRipples::default();
        single.update(&frame, Vec4::ZERO, steps(), &mut image0);
        single.update(&frame, Vec4::ZERO, steps(), &mut image0);
        batched.update(&frame, Vec4::ZERO, steps() * 2.0, &mut image1);
        let (a, b) = (single.field.unwrap(), batched.field.unwrap());
        assert_eq!(a.heights, b.heights);
        assert_eq!(a.velocity, b.velocity);
        assert_eq!(a.tick, b.tick);
    }

    #[test]
    fn native_three_four_and_eight_frame_cycles_keep_the_same_fixed_physics_when_batched() {
        let frame = frame();
        for frames in [3, 4, 8] {
            let mut image0 = crate::water_surface::shoreline(&frame).unwrap();
            let mut image1 = image0.clone();
            let mut single = WaterRipples::default();
            let mut batch = WaterRipples::default();
            let mut previous = 0.0;
            for phase in 1..=frames {
                let encoded = (phase as f32 / frames as f32 * 255.0).round();
                let current = encoded / 255.0 * std::f32::consts::TAU;
                single.update(
                    &frame,
                    Vec4::ZERO,
                    Vec2::new(current - previous, 0.0),
                    &mut image0,
                );
                previous = current;
            }
            batch.update(
                &frame,
                Vec4::ZERO,
                Vec2::new(std::f32::consts::TAU, 0.0),
                &mut image1,
            );
            let (single, batch) = (single.field.unwrap(), batch.field.unwrap());
            assert_eq!(single.tick, 6, "{frames}-frame cycle");
            assert_eq!(
                single.heights, batch.heights,
                "{frames}-frame cycle heights"
            );
            assert_eq!(
                single.velocity, batch.velocity,
                "{frames}-frame cycle velocity"
            );
        }
    }

    #[test]
    fn ambient_disturbances_displace_water_in_both_directions_at_the_actual_step_cadence() {
        let frame = frame();
        let mut field = Field::new(&frame, 48, 48, vec![true; 48 * 48], None);
        let mut positive = false;
        let mut negative = false;
        for tick in (0..128).step_by(2) {
            field.tick = tick;
            let before = field.heights.iter().sum::<f32>();
            field.ambient_drop();
            let change = field.heights.iter().sum::<f32>() - before;
            positive |= change > 0.00001;
            negative |= change < -0.00001;
        }
        assert!(
            positive && negative,
            "ambient forcing must not pump water in only one direction"
        );
    }

    #[test]
    fn missing_source_and_ineligible_water_cannot_receive_disturbances() {
        let mut frame = frame();
        let mut image = crate::water_surface::shoreline(&frame).unwrap();
        frame.tiles.retain(|tile| tile.column >= 6);
        let mut ripples = WaterRipples::default();
        assert!(ripples.update(&frame, Vec4::new(3.0, 6.0, 1.0, 1.0), steps(), &mut image));
        assert!(
            (0..48).all(
                |y| (0..24).all(|x| ripples.field.as_ref().unwrap().heights[y * 48 + x] == 0.0)
            )
        );
        frame.map_id = "CeruleanGym".into();
        assert!(!ripples.update(&frame, Vec4::ZERO, steps(), &mut image));
        assert!(ripples.field.is_none());
    }
}
