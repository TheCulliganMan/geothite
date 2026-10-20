//! External static scenery keeps its UVs and cutout materials separate from
//! the pack atlas and the opaque authored geometry.
#[cfg(feature = "terrain-worker")]
#[path = "terrain_worker/scenery_wire.rs"]
pub(super) mod worker_wire;
use bevy::asset::RenderAssetUsages;
use crate::mesh::SurfaceMeshData;
use base64::Engine as _;
use bevy::{
    prelude::*,
    render::{

        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TextureWrap {
    Clamp,
    Mirror,
    #[default]
    Repeat,
}
impl TextureWrap {
    fn address(self) -> bevy::image::ImageAddressMode {
        use bevy::image::ImageAddressMode;
        match self {
            Self::Clamp => ImageAddressMode::ClampToEdge,
            Self::Mirror => ImageAddressMode::MirrorRepeat,
            Self::Repeat => ImageAddressMode::Repeat,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EncodedTexture {
    pub width: u32,
    pub height: u32,
    pub rgba_base64: String,
    pub alpha_cutoff: Option<f32>,
    #[serde(default)]
    pub double_sided: bool,
    #[serde(default)]
    pub wrap_u: TextureWrap,
    #[serde(default)]
    pub wrap_v: TextureWrap,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneryTexture {
    blend: bool,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: bool,
    wrap_u: TextureWrap,
    wrap_v: TextureWrap,
}
impl SceneryTexture {
    pub fn decode(encoded: EncodedTexture) -> Result<Arc<Self>, String> {
        if encoded.width == 0
            || encoded.height == 0
            || encoded.width > 1024
            || encoded.height > 1024
            || encoded
                .alpha_cutoff
                .is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err("invalid external scenery texture dimensions or cutoff".into());
        }
        let expected = encoded.width as usize * encoded.height as usize * 4;
        if encoded.rgba_base64.len() > expected.div_ceil(3) * 4 {
            return Err("external scenery texture payload exceeds dimensions".into());
        }
        let pixels = base64::engine::general_purpose::STANDARD
            .decode(encoded.rgba_base64)
            .map_err(|e| e.to_string())?;
        if pixels.len() != expected {
            return Err("external scenery texture byte count mismatch".into());
        }
        Ok(Arc::new(Self {
            blend: false,
            width: encoded.width,
            height: encoded.height,
            pixels,
            alpha_cutoff: encoded.alpha_cutoff,
            double_sided: encoded.double_sided,
            wrap_u: encoded.wrap_u,
            wrap_v: encoded.wrap_v,
        }))
    }
    pub fn image(&self) -> Image {
        let _trace = crate::loading_trace::span("region.prepare_scenery_image", || serde_json::json!({"width":self.width,"height":self.height,"bytes":self.pixels.len()}));
        let mut image = Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.pixels.clone(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        );
        image.sampler = bevy::image::ImageSampler::Descriptor(
            bevy::image::ImageSamplerDescriptor {
                address_mode_u: self.wrap_u.address(),
                address_mode_v: self.wrap_v.address(),
                ..bevy::image::ImageSamplerDescriptor::linear()
            },
        );
        if crate::render_quality::requested().materials() {
            crate::texture_filtering::mipmap(&mut image, self.alpha_cutoff);
        }
        image
    }
    pub fn alpha_mode(&self) -> AlphaMode {
        if self.blend {return AlphaMode::Blend;}
        let mode = self.alpha_cutoff.map_or(AlphaMode::Opaque, AlphaMode::Mask);
        if std::env::var_os("CRYSTAL_VOXEL_PIPELINE_TRACE").is_some() {
            eprintln!("scenery alpha: mode={mode:?} size={}x{}", self.width, self.height);
        }
        mode
    }
    /// A shared neutral texture lets authored glazing supply its own tint and
    /// alpha without changing the opaque/cutout policy of imported scenery.
    /// Already paletted live map pixels; glass opacity belongs to the named
    /// pane vertices, leaving the source atlas and opaque frames untouched.
    pub(crate) fn live_glazing(width: u32, height: u32, pixels: Vec<u8>) -> Arc<Self> {
        assert_eq!(pixels.len(), width as usize * height as usize * 4);
        Arc::new(Self { blend:true,width,height,pixels,alpha_cutoff:None,double_sided:true,
            wrap_u:TextureWrap::Clamp,wrap_v:TextureWrap::Clamp })
    }

    pub fn glass_panel() -> Arc<Self> {
        static GLASS: std::sync::OnceLock<Arc<SceneryTexture>>=std::sync::OnceLock::new();
        GLASS.get_or_init(||Arc::new(Self{blend:true,width:1,height:1,pixels:vec![255;4],
            alpha_cutoff:None,double_sided:true,wrap_u:TextureWrap::Clamp,wrap_v:TextureWrap::Clamp})).clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScenerySurface {
    pub mesh: SurfaceMeshData,
    pub texture: Arc<SceneryTexture>,
}

/// One draw group per source texture, not one material/image per tree.
pub(crate) fn append_group(groups: &mut Vec<ScenerySurface>, part: ScenerySurface) {
    let group = if let Some(index) = groups
        .iter()
        .position(|g| Arc::ptr_eq(&g.texture, &part.texture))
    {
        &mut groups[index]
    } else {
        groups.push(ScenerySurface {
            mesh: SurfaceMeshData::default(),
            texture: part.texture.clone(),
        });
        groups.last_mut().unwrap()
    };
    let base = group.mesh.positions.len() as u32;
    group.mesh.positions.extend(part.mesh.positions);
    group.mesh.normals.extend(part.mesh.normals);
    group.mesh.uvs.extend(part.mesh.uvs);
    group.mesh.colors.extend(part.mesh.colors);
    group
        .mesh
        .indices
        .extend(part.mesh.indices.into_iter().map(|i| base + i));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn encoded() -> EncodedTexture {
        EncodedTexture {
            width: 2,
            height: 1,
            rgba_base64: base64::engine::general_purpose::STANDARD
                .encode([80, 120, 70, 0, 80, 120, 70, 255]),
            alpha_cutoff: Some(0.5),
            double_sided: true,
            wrap_u: TextureWrap::Repeat,
            wrap_v: TextureWrap::Clamp,
        }
    }
    #[test]
    fn authored_glazing_blends_without_changing_imported_scenery_modes() {
        let glass=SceneryTexture::glass_panel();
        assert_eq!(glass.alpha_mode(),AlphaMode::Blend);
        assert!(glass.double_sided);
        assert!(Arc::ptr_eq(&glass,&SceneryTexture::glass_panel()));
        assert_eq!(glass.image().data.as_deref(),Some(&[255;4][..]));
        let mut opaque=encoded();opaque.alpha_cutoff=None;
        assert_eq!(SceneryTexture::decode(opaque).unwrap().alpha_mode(),AlphaMode::Opaque);
        assert_eq!(SceneryTexture::decode(encoded()).unwrap().alpha_mode(),AlphaMode::Mask(0.5));
    }
    #[test]
    fn cutouts_keep_alpha_srgb_and_the_source_wrap_mode() {
        let texture = SceneryTexture::decode(encoded()).unwrap();
        let image = texture.image();
        assert_eq!(&image.data.as_ref().expect("CPU image pixels")[..8], [80, 120, 70, 0, 80, 120, 70, 255]);
        assert_eq!(
            image.texture_descriptor.format,
            TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(texture.alpha_mode(), AlphaMode::Mask(0.5));
        let bevy::image::ImageSampler::Descriptor(sampler) = image.sampler else {
            panic!("explicit sampler required")
        };
        assert!(matches!(
            sampler.address_mode_u,
            bevy::image::ImageAddressMode::Repeat
        ));
        assert!(matches!(
            sampler.address_mode_v,
            bevy::image::ImageAddressMode::ClampToEdge
        ));
        assert!(texture.double_sided);
    }
    #[test]
    fn texture_payload_limits_reject_bad_metadata_before_image_creation() {
        let mut value = encoded();
        value.width = 1025;
        assert!(SceneryTexture::decode(value).is_err());
        let mut value = encoded();
        value.height = 0;
        assert!(SceneryTexture::decode(value).is_err());
        let mut value = encoded();
        value.rgba_base64 = "AA==".into();
        assert!(SceneryTexture::decode(value).is_err());
        let mut value = encoded();
        value.alpha_cutoff = Some(f32::NAN);
        assert!(SceneryTexture::decode(value).is_err());
    }
}
