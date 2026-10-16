//! Texture descriptors stay CPU-only until the main renderer applies a build.
use super::*;
use crate::terrain_worker::blobs::{BlobStore, Bulk};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextureWire {
    blend: bool,
    width: u32,
    height: u32,
    pixels: Bulk<u8>,
    cutoff: Option<u32>,
    double_sided: bool,
    wrap: [u8; 2],
}
impl TextureWire {
    pub(crate) fn capture(texture: &SceneryTexture) -> Self {
        fn wrap(v: TextureWrap) -> u8 {
            match v {
                TextureWrap::Clamp => 0,
                TextureWrap::Mirror => 1,
                TextureWrap::Repeat => 2,
            }
        }
        Self {
            blend: texture.blend,
            width: texture.width,
            height: texture.height,
            pixels: texture.pixels.clone().into(),
            cutoff: texture.alpha_cutoff.map(f32::to_bits),
            double_sided: texture.double_sided,
            wrap: [wrap(texture.wrap_u), wrap(texture.wrap_v)],
        }
    }
    pub(crate) fn resolve(&mut self, blobs: &mut BlobStore) -> Result<bool, String> {
        self.pixels.resolve(blobs)
    }
    pub(crate) fn restore(self) -> Result<Arc<SceneryTexture>, String> {
        fn wrap(v: u8) -> Result<TextureWrap, String> {
            match v {
                0 => Ok(TextureWrap::Clamp),
                1 => Ok(TextureWrap::Mirror),
                2 => Ok(TextureWrap::Repeat),
                _ => Err("invalid texture wrap".into()),
            }
        }
        let cutoff = self.cutoff.map(f32::from_bits);
        if self.width == 0
            || self.height == 0
            || self.width > 1024
            || self.height > 1024
            || self.pixels.len() != self.width as usize * self.height as usize * 4
            || cutoff.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err("invalid terrain scenery texture".into());
        }
        Ok(Arc::new(SceneryTexture {
            blend: self.blend,
            width: self.width,
            height: self.height,
            pixels: self.pixels.into_vec()?,
            alpha_cutoff: cutoff,
            double_sided: self.double_sided,
            wrap_u: wrap(self.wrap[0])?,
            wrap_v: wrap(self.wrap[1])?,
        }))
    }
}
