//! Filter authored 3D detail in linear light. Source tile/sprite atlases never
//! enter this path: their nearest-neighbour sampling and CPU evidence stay exact.
use bevy::{
    image::{Image, ImageSampler, ImageSamplerDescriptor},
    render::render_resource::TextureFormat,
};

fn linear(v: u8) -> f32 {
    let v = v as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let v = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round() as u8
}

/// Preserve mip zero byte-for-byte; premultiply while filtering to prevent
/// transparent padding bleeding into leaves, then preserve alpha-test coverage.
pub(super) fn mipmap(image: &mut Image, alpha_cutoff: Option<f32>) {
    if image.texture_descriptor.format != TextureFormat::Rgba8UnormSrgb
        || image.texture_descriptor.size.depth_or_array_layers != 1
        || image.texture_descriptor.mip_level_count != 1
    {
        return;
    }
    let (mut width, mut height) = (image.width(), image.height());
    let Some(data) = image.data.as_mut() else {
        return;
    };
    if width == 0 || height == 0 || data.len() != width as usize * height as usize * 4 {
        return;
    }
    let cutoff = alpha_cutoff.filter(|v| *v > 0.0 && *v < 1.0);
    let target_coverage = cutoff.map(|c| coverage(data, c, 1.0));
    // A lookup makes large painted textures cheap to prepare off-thread.
    let lut: [f32; 256] = std::array::from_fn(|i| linear(i as u8));
    let mut offset = 0;
    while width > 1 || height > 1 {
        let (next_width, next_height) = ((width / 2).max(1), (height / 2).max(1));
        let mut next = Vec::with_capacity(next_width as usize * next_height as usize * 4);
        for y in 0..next_height {
            for x in 0..next_width {
                // Partition the whole source, including the odd last row/column.
                let (x0, x1) = (x * width / next_width, (x + 1) * width / next_width);
                let (y0, y1) = (y * height / next_height, (y + 1) * height / next_height);
                let mut color = [0.0; 3];
                let mut alpha = 0.0;
                for sy in y0..y1 {
                    for sx in x0..x1 {
                        let i = offset + ((sy * width + sx) * 4) as usize;
                        let a = data[i + 3] as f32 / 255.0;
                        alpha += a;
                        for c in 0..3 {
                            color[c] += lut[data[i + c] as usize] * a;
                        }
                    }
                }
                for c in color {
                    next.push(srgb(c / alpha.max(0.000001)));
                }
                next.push((alpha / ((x1 - x0) * (y1 - y0)) as f32 * 255.0).round() as u8);
            }
        }
        if let (Some(cutoff), Some(target)) = (cutoff, target_coverage) {
            preserve_coverage(&mut next, cutoff, target);
        }
        offset = data.len();
        data.extend(next);
        width = next_width;
        height = next_height;
        image.texture_descriptor.mip_level_count += 1;
    }
    let mut sampler = match &image.sampler {
        ImageSampler::Descriptor(sampler) => sampler.clone(),
        _ => ImageSamplerDescriptor::linear(),
    };
    // wgpu requires all three filters to be linear for anisotropic sampling.
    sampler.mag_filter = bevy::image::ImageFilterMode::Linear;
    sampler.min_filter = bevy::image::ImageFilterMode::Linear;
    sampler.mipmap_filter = bevy::image::ImageFilterMode::Linear;
    sampler.anisotropy_clamp = 4;
    image.sampler = ImageSampler::Descriptor(sampler);
}

fn coverage(data: &[u8], cutoff: f32, scale: f32) -> f32 {
    data.chunks_exact(4)
        .filter(|p| p[3] as f32 / 255.0 * scale >= cutoff)
        .count() as f32
        / (data.len() / 4) as f32
}
fn preserve_coverage(data: &mut [u8], cutoff: f32, target: f32) {
    let (mut lo, mut hi) = (0.0, 4.0);
    let mut best = 1.0;
    let mut error = (coverage(data, cutoff, best) - target).abs();
    for _ in 0..10 {
        let scale = (lo + hi) * 0.5;
        let current = coverage(data, cutoff, scale);
        if (current - target).abs() < error {
            best = scale;
            error = (current - target).abs();
        }
        if current < target {
            lo = scale;
        } else {
            hi = scale;
        }
    }
    for p in data.chunks_exact_mut(4) {
        p[3] = (p[3] as f32 * best).round().min(255.0) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::RenderAssetUsages,
        render::render_resource::{Extent3d, TextureDimension},
    };
    fn image(w: u32, h: u32, data: Vec<u8>) -> Image {
        Image::new(
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        )
    }
    #[test]
    fn mip_zero_is_exact_and_black_white_average_is_linear_light() {
        let pixels = vec![
            0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
        ];
        let mut image = image(2, 2, pixels.clone());
        mipmap(&mut image, None);
        assert_eq!(&image.data.as_ref().unwrap()[..16], pixels);
        assert_eq!(&image.data.as_ref().unwrap()[16..], &[188, 188, 188, 255]);
        assert_eq!(image.texture_descriptor.mip_level_count, 2);
        let ImageSampler::Descriptor(sampler) = image.sampler else {
            panic!()
        };
        assert_eq!(sampler.anisotropy_clamp, 4);
    }
    #[test]
    fn transparent_padding_does_not_stain_cutouts() {
        let mut image = image(
            2,
            2,
            vec![0, 255, 0, 255, 255, 0, 0, 0, 0, 255, 0, 255, 255, 0, 0, 0],
        );
        mipmap(&mut image, Some(0.5));
        assert_eq!(&image.data.as_ref().unwrap()[16..19], &[0, 255, 0]);
    }
    #[test]
    fn non_square_odd_textures_include_edges_and_have_a_complete_chain() {
        let mut pixels = vec![0; 3 * 5 * 4];
        for p in pixels.chunks_exact_mut(4) {
            p[3] = 255;
        }
        pixels[56..59].fill(255);
        let mut image = image(3, 5, pixels.clone());
        mipmap(&mut image, None);
        assert_eq!(image.texture_descriptor.mip_level_count, 3);
        assert_eq!(image.data.as_ref().unwrap().len(), 60 + 8 + 4);
        assert_eq!(&image.data.as_ref().unwrap()[..60], pixels);
        assert!(image.data.as_ref().unwrap()[68] > 0);
        let before = image.data.clone();
        mipmap(&mut image, None);
        assert_eq!(image.data, before);
    }
}
