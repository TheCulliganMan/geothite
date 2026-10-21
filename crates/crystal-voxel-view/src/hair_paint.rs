//! Reuse the existing painted hair as quiet longitudinal vertex albedo.
//! The body material samples a constant neutral portrait texel, so it does
//! not multiply the hair pigment by a second copy of the painting.

use crate::npc_textures::Texture;

pub(crate) struct HairPaint<'a> {
    texture: &'a Texture,
    pub(crate) uv: [f32; 2],
    pub(crate) neutral: [f32; 3],
    pub(crate) base: [f32; 3],
}

fn linear(value: f32) -> f32 {
    let value = value / 255.;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

impl<'a> HairPaint<'a> {
    pub(crate) fn new(texture: &'a Texture) -> Self {
        let (width, height) = texture.size;
        // Uniform recoloring preserves this portrait region. Pick the centre
        // of its brightest neutral texel, rather than a clothing swatch whose
        // pigment could change. Constant UV derivatives retain mip level zero.
        let mut best = (0, 0, [0_u8; 3]);
        for y in height * 11 / 100..height * 26 / 100 {
            for x in width * 15 / 100..width * 48 / 100 {
                let at = ((y * width + x) * 4) as usize;
                let rgb = [texture.rgba[at], texture.rgba[at + 1], texture.rgba[at + 2]];
                if rgb.into_iter().min() > best.2.into_iter().min() {
                    best = (x, y, rgb);
                }
            }
        }
        let mut paint = Self {
            texture,
            uv: [
                (best.0 as f32 + 0.5) / width as f32,
                (best.1 as f32 + 0.5) / height as f32,
            ],
            neutral: best.2.map(|v| linear(f32::from(v)).max(0.001)),
            base: [0.; 3],
        };
        paint.base = paint.sample([0.06, 0.048]);
        paint
    }

    pub(crate) fn sample(&self, uv: [f32; 2]) -> [f32; 3] {
        let (width, height) = self.texture.size;
        let x = (uv[0] * width as f32) as i32;
        let y = (uv[1] * height as f32) as i32;
        let mut sum = [0_u32; 3];
        for dy in -2..=2 {
            for dx in -2..=2 {
                let px = (x + dx).clamp(0, width as i32 - 1) as u32;
                let py = (y + dy).clamp(0, height as i32 - 1) as u32;
                let at = ((py * width + px) * 4) as usize;
                for (channel, total) in sum.iter_mut().enumerate() {
                    *total += u32::from(self.texture.rgba[at + channel]);
                }
            }
        }
        sum.map(|v| linear(v as f32 / 25.))
    }

    pub(crate) fn hair(&self, uv: [f32; 2]) -> [f32; 3] {
        assert!(
            (0.008..=0.14).contains(&uv[0]) && (0.04..=0.235).contains(&uv[1]),
            "hair albedo must stay in the inspected hair-only atlas strip"
        );
        self.sample(uv)
    }

    pub(crate) fn color(&self, rgb: [f32; 3]) -> [f32; 4] {
        [
            (rgb[0] / self.neutral[0]).clamp(0., 1.),
            (rgb[1] / self.neutral[1]).clamp(0., 1.),
            (rgb[2] / self.neutral[2]).clamp(0., 1.),
            1.,
        ]
    }
}

#[cfg(test)]
mod tests {
    use bevy::color::Color;

    use super::HairPaint;
    use crate::npc_textures::Texture;

    #[test]
    fn neutral_texel_preserves_sampled_hair_pigment_in_the_body_material() {
        let mut rgba = [0, 255, 0, 255].repeat(128 * 128);
        for y in 0..31 {
            for x in 0..19 {
                let at = (y * 128 + x) * 4;
                rgba[at..at + 4].copy_from_slice(&[120, 70, 40, 255]);
            }
        }
        let at = (23 * 128 + 40) * 4;
        rgba[at..at + 4].copy_from_slice(&[250, 251, 252, 255]);
        let texture = Texture {
            size: (128, 128),
            rgba,
        };
        let paint = HairPaint::new(&texture);
        assert_eq!(paint.uv, [40.5 / 128., 23.5 / 128.]);
        let source = paint.hair([0.06, 0.12]);
        let vertex = paint.color(source);
        let expected = Color::srgb_u8(120, 70, 40).to_linear();
        for (channel, expected) in [expected.red, expected.green, expected.blue]
            .into_iter()
            .enumerate()
        {
            assert!(
                (vertex[channel] * paint.neutral[channel] - expected).abs() < 1e-6,
                "the body material must not tint the sampled hair pigment twice"
            );
        }
        assert_eq!(vertex[3], 1.);
    }
}
