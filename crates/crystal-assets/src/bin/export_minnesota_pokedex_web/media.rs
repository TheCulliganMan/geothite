//! Browser controls and previews of the reviewed native art; audio stays external.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crystal_assets::minnesota_overworld_sprites::walking_thumbnail;

#[derive(Default)]
pub(super) struct MediaCatalog {
    thumbnails: BTreeMap<String, String>,
    cry_base: Option<String>,
}

impl MediaCatalog {
    pub(super) fn export(
        overworld: &Path,
        output: &Path,
        base: &str,
        cry_base: &str,
        active: &BTreeSet<String>,
    ) -> Result<Self> {
        ensure!(
            cry_base.starts_with("/pokedex/")
                && !cry_base.contains(['"', '<', '>', '\\'])
                && !cry_base.contains(".."),
            "invalid external cry URL base"
        );
        let registry: Value = serde_json::from_slice(&fs::read(overworld.join("accepted.json"))?)?;
        let mut catalog = Self {
            thumbnails: BTreeMap::new(),
            cry_base: Some(cry_base.trim_end_matches('/').into()),
        };
        fs::create_dir_all(output.join("thumbnails"))?;
        for entry in super::array(&registry["species"]) {
            let id = super::string(&entry["species_id"]);
            ensure!(
                active.contains(id),
                "thumbnail references inactive species {id}"
            );
            let token = super::string(&entry["name"]).to_ascii_lowercase();
            ensure!(
                !token.is_empty() && token.bytes().all(|b| b.is_ascii_lowercase()),
                "unsafe thumbnail species token"
            );
            let folder = overworld.join("native/gfx/pokemon").join(&token);
            let bank = checked_file(&folder, "overworld.2bpp", entry)?;
            let stem = id.to_ascii_lowercase().replace('_', "-");
            for (suffix, file) in [
                ("normal", "overworld.gbcpal"),
                ("shiny", "overworld-shiny.gbcpal"),
            ] {
                let bytes = checked_file(&folder, file, entry)?;
                ensure!(bytes.len() == 8, "invalid thumbnail palette {id}");
                let palette =
                    std::array::from_fn(|i| u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]));
                fs::write(
                    output
                        .join("thumbnails")
                        .join(format!("{stem}-{suffix}.png")),
                    walking_thumbnail(&bank, palette)?,
                )?;
            }
            ensure!(
                catalog
                    .thumbnails
                    .insert(id.into(), format!("{base}/thumbnails/{stem}"))
                    .is_none(),
                "duplicate thumbnail {id}"
            );
        }
        Ok(catalog)
    }

    pub(super) fn controls(&self, member: &Value, active: bool, detail: bool) -> String {
        if !active {
            return String::new();
        }
        let id = super::string(&member["id"]);
        let name = super::esc(super::string(&member["name"]));
        let thumbnail = self.thumbnails.get(id).map(|base| format!(
            "<span class=\"walking-thumbnail\" data-walker title=\"{name} walking sprite\"><img src=\"{base}-normal.png\" data-walk-normal=\"{base}-normal.png\" data-walk-shiny=\"{base}-shiny.png\" alt=\"{name} walking sprite\" width=\"192\" height=\"48\" loading=\"lazy\" decoding=\"async\"></span>"
        )).unwrap_or_default();
        let cry = self.cry_base.as_ref().map(|base| format!(
            "<button type=\"button\" class=\"cry-button\" data-cry=\"{base}/{id}.wav\" data-cry-name=\"{name}\" aria-label=\"Play {name} cry\" aria-pressed=\"false\">▶ Cry</button>"
        )).unwrap_or_default();
        if thumbnail.is_empty() && cry.is_empty() {
            String::new()
        } else {
            let motion = if detail && self.thumbnails.contains_key(id) {
                self.motion_control()
            } else {
                String::new()
            };
            format!("<div class=\"species-media\">{thumbnail}{cry}{motion}</div>")
        }
    }

    pub(super) fn motion_control(&self) -> String {
        if self.thumbnails.is_empty() {
            return String::new();
        }
        "<button type=\"button\" class=\"motion-button\" data-walking-toggle aria-pressed=\"true\">Pause sprites</button>".into()
    }

    pub(super) fn thumbnail_count(&self) -> usize {
        self.thumbnails.len()
    }
}

fn checked_file(folder: &Path, name: &str, entry: &Value) -> Result<Vec<u8>> {
    let bytes = fs::read(folder.join(name))?;
    let expected = super::array(&entry["runtime_files"])
        .iter()
        .find(|file| super::string(&file["path"]).ends_with(&format!("/{name}")))
        .context("accepted runtime file hash missing")?;
    ensure!(
        format!("{:x}", Sha256::digest(&bytes)) == super::string(&expected["sha256"]),
        "thumbnail source changed since native review: {}",
        folder.join(name).display()
    );
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::walking_thumbnail;
    use crate::minnesota_pokemon_pixel_art::bitplanes;
    use crate::sprite::MinnesotaSheet;

    const PIXELS: usize = 16 * 16;
    const FRAMES: usize = 6;

    #[test]
    fn walking_thumbnail_preserves_native_colors_holes_and_alternating_feet() {
        let mut pixels = vec![0; PIXELS * FRAMES];
        // An asymmetric footprint distinguishes a real mirrored step from a repeat.
        for (frame, x, ink) in [(0, 4, 1), (3, 5, 2)] {
            pixels[frame * PIXELS + 8 * 16 + x] = ink;
            pixels[frame * PIXELS + 8 * 16 + 7] = 3;
        }
        let palette = [32767, 31, 992, 0];
        let bank = bitplanes(&pixels, 16, 96);
        let png = walking_thumbnail(&bank, palette).expect("native walking strip");
        let sheet = MinnesotaSheet::from_png(&png).expect("decode browser image independently");
        assert_eq!((sheet.width, sheet.height), (64, 16));
        for (x, color) in [
            (4, [255, 0, 0]),
            (21, [0, 255, 0]),
            (36, [255, 0, 0]),
            (58, [0, 255, 0]),
        ] {
            assert_eq!(sheet.pixel(x, 8), Some(color));
            assert_eq!(sheet.alpha(x, 8), Some(255));
        }
        for x in [7, 23, 39, 56] {
            assert_eq!(sheet.pixel(x, 8), Some([0, 0, 0]));
            assert_eq!(sheet.alpha(x, 8), Some(255));
        }
        for x in [0, 6, 16, 22, 32, 38, 48, 57, 63] {
            assert_eq!(
                sheet.alpha(x, 8),
                Some(0),
                "preserve empty pixels and holes"
            );
        }
        assert!(walking_thumbnail(&bank[..383], palette).is_err());
        pixels[0] = 1;
        assert!(walking_thumbnail(&bitplanes(&pixels, 16, 96), palette).is_err());
        assert!(walking_thumbnail(&bank, [32767, 31, 31, 0]).is_err());
        assert!(walking_thumbnail(&vec![0; 384], palette).is_err());
    }
}
