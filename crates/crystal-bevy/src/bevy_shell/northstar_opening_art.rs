//! A staged reveal and native pixel animation of Northstar's installed artwork.
use anyhow::Result;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::{Assets, Image, Vec2};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crystal_assets::AssetRoot;

use super::SpriteFrame;
use super::northstar_opening::Opening;
use super::northstar_title_scene::Scene;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

pub(super) struct Art {
    scene: Scene,
    surface: Option<SpriteFrame>,
    previous_frame: Option<u32>,
}

impl Art {
    pub(super) fn load(root: &AssetRoot) -> Result<Self> {
        let title =
            crate::open_runtime_image(root.runtime_assets().join("gfx/title/northstar.png"))?
                .to_rgba8();
        Ok(Self {
            scene: Scene::new(title)?,
            surface: None,
            previous_frame: None,
        })
    }

    pub(super) fn render(&mut self, opening: &Opening, images: &mut Assets<Image>) -> SpriteFrame {
        let frame = opening.image_frame();
        if self.previous_frame == Some(frame)
            && let Some(surface) = &self.surface
        {
            return surface.clone();
        }
        let pixels = self.scene.compose(frame).into_raw();
        if let Some(surface) = &self.surface {
            images
                .get_mut(&surface.handle)
                .expect("owned Northstar LCD")
                .data = Some(pixels);
        } else {
            let mut image = Image::new(
                Extent3d {
                    width: WIDTH,
                    height: HEIGHT,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            image.sampler = ImageSampler::nearest();
            self.surface = Some(SpriteFrame {
                handle: images.add(image),
                size: Vec2::new(WIDTH as f32, HEIGHT as f32),
            });
        }
        self.previous_frame = Some(frame);
        self.surface
            .as_ref()
            .expect("composed Northstar LCD")
            .clone()
    }
}
