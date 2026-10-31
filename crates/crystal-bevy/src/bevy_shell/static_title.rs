//! Load one complete native title card without a sprite or text overlay.
use anyhow::{Context, Result};
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::{Assets, Image, Vec2};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crystal_assets::AssetRoot;

use super::SpriteFrame;

pub(super) fn load(
    root: &AssetRoot,
    path: &str,
    images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    let source = crate::open_runtime_image(root.runtime_assets().join(path))
        .context("decode static regional title")?
        .to_rgba8();
    anyhow::ensure!(
        source.dimensions() == (160, 144),
        "static title must be 160x144"
    );
    let mut image = Image::new(
        Extent3d {
            width: 160,
            height: 144,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        source.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    Ok(SpriteFrame {
        handle: images.add(image),
        size: Vec2::new(160.0, 144.0),
    })
}
