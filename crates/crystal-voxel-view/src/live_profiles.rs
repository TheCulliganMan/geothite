//! Runtime-authored object folds. Editing JSON invalidates geometry, not code.
use bevy::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
    pub objects: Vec<Object>,
    pub atmosphere: Option<Atmosphere>,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Object {
    pub name: String,
    pub tileset: String,
    pub map: Option<String>,
    /// Optional exact map allowlist; mutually exclusive with `map`.
    pub maps: Option<Vec<String>>,
    pub metatile: u16,
    /// Exact block grid when the drawing crosses its anchor block.
    pub metatiles: Option<Vec<Vec<u16>>>,
    pub origin: [u8; 2],
    pub tiles: Vec<Vec<u16>>,
    pub ground: u16,
    /// Native pixels assigned to the top; the remaining pixels fold upright.
    pub top_pixels: usize,
    pub depth_pixels: f32,
    /// Explicit actor support height over this drawing, in source pixels.
    pub footing_pixels: Option<f32>,
    #[serde(default)]
    pub mask: Mask,
    #[serde(default)]
    pub parts: Vec<Part>,
}

#[derive(Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mask {
    #[default]
    None,
    Ground,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Part {
    pub rect: [usize; 4],
    pub top_pixels: usize,
    pub depth_pixels: f32,
    #[serde(default)]
    pub base_pixels: f32,
    pub height_pixels: Option<f32>,
    /// Slope a complete top drawing down to its perimeter instead of folding a facade.
    pub bevel_pixels: Option<f32>,
    #[serde(default)]
    pub offset_pixels: [f32; 2],
    #[serde(default)]
    pub mask: Mask,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Atmosphere {
    pub maps: Vec<String>,
    pub start_tiles: f32,
    pub end_tiles: f32,
    pub opacity: f32,
}

impl Document {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let document: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if let Some(atmosphere) = &document.atmosphere {
            if atmosphere.maps.is_empty()
                || atmosphere.maps.iter().any(|map| map.is_empty())
                || !atmosphere.start_tiles.is_finite()
                || !atmosphere.end_tiles.is_finite()
                || !atmosphere.opacity.is_finite()
                || atmosphere.start_tiles < 0.0
                || atmosphere.end_tiles <= atmosphere.start_tiles
                || atmosphere.end_tiles > 128.0
                || !(0.0..=1.0).contains(&atmosphere.opacity)
            {
                return Err("invalid atmosphere maps, distances or opacity".into());
            }
        }
        if document.objects.len() > 256 {
            return Err("at most 256 object profiles are supported".into());
        }
        for object in &document.objects {
            if object.maps.as_ref().is_some_and(|maps| {
                object.map.is_some() || maps.is_empty() || maps.iter().any(String::is_empty)
            }) {
                return Err(format!("invalid map selectors in {}", object.name));
            }
            let width = object.tiles.first().map_or(0, Vec::len);
            let height = object.tiles.len();
            if width == 0
                || height == 0
                || object.origin[0] >= 4
                || object.origin[1] >= 4
                || width > 8
                || height > 8
                || usize::from(object.origin[0]) + width > 8
                || usize::from(object.origin[1]) + height > 8
                || object.tiles.iter().any(|row| row.len() != width)
                || object.top_pixels >= height * 8
                || object.footing_pixels.is_some_and(|h| !h.is_finite() || !(0.0..=64.0).contains(&h))
                || !object.depth_pixels.is_finite()
                || !(0.0..=32.0).contains(&object.depth_pixels)
                || (object.top_pixels > 0 && object.depth_pixels == 0.0)
                || object.name.is_empty()
                || object.tileset.is_empty()
            {
                return Err(format!("invalid drawing, fold or depth in {}", object.name));
            }
            if object.parts.len() > 32 {
                return Err(format!("too many parts in {}", object.name));
            }
            let mut occupied = vec![false; width * height * 64];
            for part in &object.parts {
                let [x, y, w, h] = part.rect;
                if w == 0
                    || h == 0
                    || x > width * 8
                    || y > height * 8
                    || w > width * 8 - x
                    || h > height * 8 - y
                    || part.top_pixels > h
                    || !part.depth_pixels.is_finite()
                    || !(0.0..=64.0).contains(&part.depth_pixels)
                    || (part.top_pixels > 0 && part.depth_pixels == 0.0)
                    || !part.base_pixels.is_finite()
                    || !(0.0..=64.0).contains(&part.base_pixels)
                    || part
                        .height_pixels
                        .is_some_and(|v| !v.is_finite() || !(0.0..=64.0).contains(&v))
                    || part.bevel_pixels.is_some_and(|v| {
                        !v.is_finite()
                            || v <= 0.0
                            || v > w.min(h) as f32 / 2.0
                            || part.top_pixels != h
                            || part.base_pixels != 0.0
                            || !matches!(part.mask, Mask::None)
                            || !part.height_pixels.is_some_and(|height| height > 0.0)
                    })
                    || part
                        .offset_pixels
                        .iter()
                        .any(|v| !v.is_finite() || !(-64.0..=64.0).contains(v))
                {
                    return Err(format!("invalid part in {}", object.name));
                }
                for py in y..y + h {
                    for px in x..x + w {
                        let index = py * width * 8 + px;
                        if occupied[index] {
                            return Err(format!("overlapping source parts in {}", object.name));
                        }
                        occupied[index] = true;
                    }
                }
            }
            let blocks_w = (usize::from(object.origin[0]) + width).div_ceil(4);
            let blocks_h = (usize::from(object.origin[1]) + height).div_ceil(4);
            match &object.metatiles {
                Some(blocks)
                    if blocks.len() == blocks_h
                        && blocks.iter().all(|row| row.len() == blocks_w)
                        && blocks[0][0] == object.metatile => {}
                None if blocks_w == 1 && blocks_h == 1 => {}
                _ => return Err(format!("missing or invalid block grid in {}", object.name)),
            }
        }
        Ok(document)
    }
}

#[derive(Resource)]
pub(crate) struct LiveProfiles {
    pub revision: u64,
    pub document: Arc<Document>,
    #[cfg(not(target_arch = "wasm32"))]
    path: Option<std::path::PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    last_poll: std::time::Instant,
    #[cfg(not(target_arch = "wasm32"))]
    observed: Option<Vec<u8>>,
    #[cfg(not(target_arch = "wasm32"))]
    last_error: Option<String>,
}

impl Default for LiveProfiles {
    fn default() -> Self {
        Self {
            revision: 0,
            document: Arc::new(
                Document::parse(include_bytes!("../../../modpacks/voxel-view/profiles.json"))
                    .expect("bundled geometry profiles must be valid"),
            ),
            #[cfg(not(target_arch = "wasm32"))]
            path: std::env::var_os("CRYSTAL_VOXEL_PROFILES").map(Into::into),
            #[cfg(not(target_arch = "wasm32"))]
            last_poll: std::time::Instant::now() - std::time::Duration::from_secs(1),
            #[cfg(not(target_arch = "wasm32"))]
            observed: None,
            #[cfg(not(target_arch = "wasm32"))]
            last_error: None,
        }
    }
}

pub(crate) fn reload(mut profiles: ResMut<LiveProfiles>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(path) = profiles.path.clone() else {
            return;
        };
        if profiles.last_poll.elapsed().as_millis() < 150 {
            return;
        }
        // Bypass change detection for polling bookkeeping. Only a valid new
        // document should wake the renderer and invalidate its geometry cache.
        profiles.bypass_change_detection().last_poll = std::time::Instant::now();
        let result = std::fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                if profiles.observed.as_ref() == Some(&bytes) {
                    return Ok(None);
                }
                let document = Document::parse(&bytes)?;
                profiles.bypass_change_detection().observed = Some(bytes);
                Ok(Some(document))
            });
        match result {
            Ok(Some(document)) => {
                if profiles.document.objects != document.objects {
                    profiles.revision = profiles.revision.saturating_add(1);
                }
                profiles.document = Arc::new(document);
                profiles.last_error = None;
                println!(
                    "geometry profiles loaded: revision {} from {} ({} objects)",
                    profiles.revision,
                    path.display(),
                    profiles.document.objects.len()
                );
            }
            Ok(None) => {}
            Err(error) => {
                if profiles.last_error.as_ref() != Some(&error) {
                    eprintln!("geometry profile edit rejected; keeping previous geometry: {error}");
                    profiles.bypass_change_detection().last_error = Some(error);
                }
            }
        }
    }
}
