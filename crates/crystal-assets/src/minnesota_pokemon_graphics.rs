//! Verify reviewed native sprites and package their exact runtime assets.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::minnesota_pokemon_profiles::{Family, Profile};
use crate::minnesota_sprite_refinement::{self, PixelEdit};
use crate::minnesota_sprite_sheet::{
    ExteriorContour, MinnesotaSheet, SpriteConversionOptions, SpriteCrop, SpriteSampling,
    SpriteView,
};

#[derive(Deserialize)]
struct SpriteManifest {
    family_id: String,
    visual_review: VisualReview,
    sprites: Vec<SpriteImport>,
}

#[derive(Deserialize)]
struct Registry {
    families: Vec<FamilyManifest>,
}

#[derive(Deserialize)]
struct FamilyManifest {
    id: String,
    manifest: String,
}

#[derive(Deserialize)]
struct VisualReview {
    status: String,
    outline_profile: String,
    sheet_sha256: String,
}

#[derive(Deserialize)]
struct SpriteImport {
    id: String,
    species_id: String,
    view: String,
    source: String,
    crop: [usize; 4],
    fit: [usize; 2],
    normal_palette_rgb555: [u16; 4],
    #[serde(default)]
    source_palette_rgb555: Option<[u16; 4]>,
    white_detail_min_samples: Option<usize>,
    interior_black_min_samples: Option<usize>,
    #[serde(default)]
    mirror_horizontal: bool,
    #[serde(default)]
    exterior_contour: ExteriorContour,
    #[serde(default)]
    sampling: SpriteSampling,
    #[serde(default)]
    pixel_edits: Vec<PixelEdit>,
}

#[derive(Deserialize)]
struct NativeFrame {
    view: String,
    size: [usize; 2],
    rows: Vec<String>,
    palette_rgb555: [u16; 4],
    source_sha256: String,
    outline_profile: String,
    import_crop: [usize; 4],
    import_fit: [usize; 2],
    #[serde(default = "default_white_detail_min_samples")]
    white_detail_min_samples: usize,
    #[serde(default = "default_interior_black_min_samples")]
    interior_black_min_samples: usize,
    #[serde(default)]
    mirror_horizontal: bool,
    #[serde(default)]
    exterior_contour: ExteriorContour,
    #[serde(default)]
    sampling: SpriteSampling,
    #[serde(default)]
    pixel_edits: Vec<PixelEdit>,
    #[serde(default)]
    quantization_palette_rgb555: Option<[u16; 4]>,
}

fn default_white_detail_min_samples() -> usize {
    16
}

fn default_interior_black_min_samples() -> usize {
    32
}

#[derive(Deserialize)]
struct Palettes {
    pokemon: BTreeMap<String, Palette>,
}

#[derive(Deserialize)]
struct Palette {
    gbc_rgb555_words: [u16; 4],
}

pub(crate) fn load_family_graphics(
    pokemon_root: &Path,
    native_root: &Path,
    family: &Family,
    profiles: &BTreeMap<String, Profile>,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let registry: Registry =
        serde_json::from_slice(&fs::read(pokemon_root.join("sprite-generation.json"))?)?;
    let references: Vec<_> = registry
        .families
        .iter()
        .filter(|entry| entry.id == family.id)
        .collect();
    ensure!(
        references.len() == 1,
        "family must have exactly one registered sprite manifest"
    );
    crate::validate_compiled_runtime_file_key(&references[0].manifest)?;
    let manifest_path = pokemon_root.join(&references[0].manifest);
    let manifest: SpriteManifest = serde_json::from_slice(&fs::read(&manifest_path)?)
        .with_context(|| format!("read reviewed manifest {}", manifest_path.display()))?;
    ensure!(manifest.family_id == family.id, "sprite family mismatch");
    let has_source_backed = manifest
        .sprites
        .iter()
        .any(|sprite| sprite.exterior_contour == ExteriorContour::SourceBacked);
    let has_solid = manifest
        .sprites
        .iter()
        .any(|sprite| sprite.exterior_contour == ExteriorContour::Solid);
    let review_profile = match (has_source_backed, has_solid) {
        (true, true) => "mixed-source-solid-v1",
        (false, true) => ExteriorContour::Solid.outline_profile(),
        _ => ExteriorContour::SourceBacked.outline_profile(),
    };
    ensure!(
        manifest.visual_review.status == "reviewed"
            && manifest.visual_review.outline_profile == review_profile,
        "family {} lacks accepted {review_profile} review",
        family.id
    );
    let native_directory = native_root.join(&family.id);
    ensure!(
        sha256(&fs::read(native_directory.join("line.review.png"))?)
            == manifest.visual_review.sheet_sha256,
        "stale visual review for {}",
        family.id
    );
    let palettes: Palettes =
        serde_json::from_slice(&fs::read(pokemon_root.join("palettes.json"))?)?;
    let mut files = BTreeMap::new();
    for member in &family.members {
        let token = member.name.to_ascii_uppercase();
        let root = format!("gfx/pokemon/{}", token.to_ascii_lowercase());
        let palette = palettes
            .pokemon
            .get(&member.id)
            .with_context(|| format!("missing palette {}", member.id))?
            .gbc_rgb555_words;
        validate_palette(palette)?;
        for (view, size) in [("front", 56_usize), ("back", 48)] {
            let imports: Vec<_> = manifest
                .sprites
                .iter()
                .filter(|entry| entry.species_id == member.id && entry.view == view)
                .collect();
            ensure!(
                imports.len() == 1,
                "{} must have exactly one {view} import",
                member.id
            );
            let import = imports[0];
            ensure!(
                !import.id.is_empty()
                    && import
                        .id
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
                "invalid native sprite basename"
            );
            crate::validate_compiled_runtime_file_key(&import.source)?;
            let stem = native_directory.join(format!("{}.{view}", import.id));
            let metadata: NativeFrame =
                serde_json::from_slice(&fs::read(stem.with_extension(format!("{view}.json")))?)?;
            ensure!(
                metadata.view == view && metadata.size == [size, size],
                "native view geometry mismatch"
            );
            ensure!(
                metadata.outline_profile == import.exterior_contour.outline_profile()
                    && metadata.exterior_contour == import.exterior_contour
                    && metadata.sampling == import.sampling
                    && metadata.pixel_edits == import.pixel_edits
                    && metadata.import_crop == import.crop
                    && metadata.import_fit == import.fit
                    && metadata.white_detail_min_samples
                        == import.white_detail_min_samples.unwrap_or(16)
                    && metadata.interior_black_min_samples
                        == import.interior_black_min_samples.unwrap_or(32)
                    && metadata.mirror_horizontal == import.mirror_horizontal,
                "stale native import for {} {view}",
                member.id
            );
            ensure!(
                metadata.palette_rgb555 == palette && import.normal_palette_rgb555 == palette,
                "native palette mismatch for {}",
                member.id
            );
            let source = fs::read(pokemon_root.join(&import.source))?;
            ensure!(
                metadata.source_sha256 == sha256(&source),
                "stale native source {} {view}",
                member.id
            );
            if import.sampling == SpriteSampling::SourceCenter || !import.pixel_edits.is_empty() {
                verify_source_conversion(&source, import, &metadata)?;
            }
            let indices = parse_rows(&metadata.rows, size)?;
            let png = fs::read(stem.with_extension(format!("{view}.png")))?;
            verify_png(&png, &indices, size, palette)?;
            let bitplanes = fs::read(stem.with_extension(format!("{view}.2bpp")))?;
            ensure!(
                bitplanes == encode_bitplanes(&indices, size, view == "back"),
                "native bitplanes disagree with pixels for {} {view}",
                member.id
            );
            files.insert(format!("{root}/{view}.png"), png);
            files.insert(format!("{root}/{view}.2bpp"), bitplanes);
        }
        files.insert(format!("{root}/front.dimensions"), vec![0x77]);
        files.insert(
            format!("{root}/normal.gbcpal"),
            palette
                .iter()
                .flat_map(|color| color.to_le_bytes())
                .collect(),
        );
        let shiny = profiles
            .get(&member.id)
            .context("missing profile")?
            .shiny_palette()?;
        let shiny_text: String = shiny[1..3]
            .iter()
            .map(|value| {
                format!(
                    "RGB {}, {}, {}\n",
                    value & 31,
                    (value >> 5) & 31,
                    (value >> 10) & 31
                )
            })
            .collect();
        files.insert(format!("{root}/shiny.pal"), shiny_text.into_bytes());
    }
    Ok(files)
}

fn verify_source_conversion(
    source: &[u8],
    import: &SpriteImport,
    metadata: &NativeFrame,
) -> Result<()> {
    ensure!(
        metadata.sampling == import.sampling,
        "source conversion sampling policy mismatch"
    );
    ensure!(
        metadata.pixel_edits == import.pixel_edits,
        "native pixel authoring differs from manifest"
    );
    let quantization_palette = import
        .source_palette_rgb555
        .unwrap_or(import.normal_palette_rgb555);
    ensure!(
        metadata.quantization_palette_rgb555 == Some(quantization_palette),
        "stale source conversion quantization palette"
    );
    let [x, y, width, height] = import.crop;
    let view = match import.view.as_str() {
        "front" => SpriteView::Front,
        "back" => SpriteView::Back,
        _ => anyhow::bail!("invalid source conversion view"),
    };
    let mut converted = MinnesotaSheet::from_png(source)?.candidate_with_conversion(
        SpriteCrop {
            x,
            y,
            width,
            height,
            fit_width: import.fit[0],
            fit_height: import.fit[1],
        },
        quantization_palette,
        view,
        SpriteConversionOptions {
            minimum_white_samples: u16::try_from(import.white_detail_min_samples.unwrap_or(16))?,
            minimum_black_samples: u16::try_from(import.interior_black_min_samples.unwrap_or(32))?,
            exterior_contour: import.exterior_contour,
            sampling: import.sampling,
        },
    )?;
    if import.mirror_horizontal {
        converted.mirror_horizontal();
    }
    if !import.pixel_edits.is_empty() {
        let mut rows = converted.rows.clone();
        minnesota_sprite_refinement::apply(&mut rows, &import.pixel_edits)?;
        converted.set_rows(rows)?;
    }
    ensure!(
        converted.rows == metadata.rows,
        "native pixels differ from deterministic source conversion"
    );
    Ok(())
}

fn validate_palette(palette: [u16; 4]) -> Result<()> {
    ensure!(
        palette[0] == 0x7fff
            && palette[3] == 0
            && palette.iter().all(|color| *color < 0x8000)
            && palette
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == 4,
        "normal palette must contain four distinct RGB555 colors with white/black endpoints"
    );
    Ok(())
}

fn parse_rows(rows: &[String], size: usize) -> Result<Vec<u8>> {
    ensure!(rows.len() == size, "wrong native row count");
    let mut pixels = Vec::with_capacity(size * size);
    for row in rows {
        ensure!(row.len() == size, "wrong native row width");
        for byte in row.bytes() {
            pixels.push(match byte {
                b'.' => 0,
                b'1'..=b'3' => byte - b'0',
                _ => anyhow::bail!("unknown native palette index"),
            });
        }
    }
    ensure!(
        pixels.iter().any(|pixel| *pixel != 0),
        "empty native sprite"
    );
    Ok(pixels)
}

fn verify_png(bytes: &[u8], pixels: &[u8], size: usize, palette: [u16; 4]) -> Result<()> {
    let png = MinnesotaSheet::from_png(bytes)?;
    ensure!(
        png.width == size && png.height == size,
        "native PNG dimensions mismatch"
    );
    for (offset, index) in pixels.iter().enumerate() {
        let color = palette[usize::from(*index)];
        let channels = [color & 31, (color >> 5) & 31, (color >> 10) & 31];
        let expected = channels.map(|channel| ((channel << 3) | (channel >> 2)) as u8);
        ensure!(
            png.pixel(offset % size, offset / size) == Some(expected),
            "native PNG pixel disagrees with indexed source"
        );
    }
    Ok(())
}

fn encode_bitplanes(pixels: &[u8], size: usize, columns: bool) -> Vec<u8> {
    let tiles = size / 8;
    let mut bytes = Vec::with_capacity(size * size / 4);
    for major in 0..tiles {
        for minor in 0..tiles {
            let (tile_x, tile_y) = if columns {
                (major, minor)
            } else {
                (minor, major)
            };
            for y in 0..8 {
                let mut low = 0;
                let mut high = 0;
                for x in 0..8 {
                    let pixel = pixels[(tile_y * 8 + y) * size + tile_x * 8 + x];
                    low |= (pixel & 1) << (7 - x);
                    high |= ((pixel >> 1) & 1) << (7 - x);
                }
                bytes.extend([low, high]);
            }
        }
    }
    bytes
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::{
        NativeFrame, SpriteImport, encode_bitplanes, parse_rows, verify_source_conversion,
    };
    use crate::minnesota_pokemon_pixel_art::png_rgb;
    use crate::minnesota_sprite_sheet::{
        ExteriorContour, MinnesotaSheet, SpriteConversionOptions, SpriteCrop, SpriteSampling,
        SpriteView,
    };

    #[test]
    fn native_loader_replays_guarded_edits_for_both_sampling_modes() -> anyhow::Result<()> {
        let palette = [0x7fff, 0x2b78, 0x3caa, 0];
        let mut pixels = vec![[0; 3]; 20 * 20];
        for y in 1..19 {
            for x in 1..19 {
                pixels[y * 20 + x] = if x < 10 { [255; 3] } else { [100, 60, 80] };
            }
        }
        let source = png_rgb(20, 20, &pixels)?;
        let crop = SpriteCrop {
            x: 0,
            y: 0,
            width: 20,
            height: 20,
            fit_width: 10,
            fit_height: 10,
        };
        for sampling in [SpriteSampling::AreaVote, SpriteSampling::SourceCenter] {
            let mut frame = MinnesotaSheet::from_png(&source)?.candidate_with_conversion(
                crop,
                palette,
                SpriteView::Front,
                SpriteConversionOptions {
                    minimum_white_samples: 16,
                    minimum_black_samples: 32,
                    exterior_contour: ExteriorContour::SourceBacked,
                    sampling,
                },
            )?;
            frame.mirror_horizontal();
            let mut value = serde_json::to_value(&frame)?;
            value["source_sha256"] = "fixture".into();
            value["import_crop"] = serde_json::json!([0, 0, 20, 20]);
            value["import_fit"] = serde_json::json!([10, 10]);
            value["quantization_palette_rgb555"] = serde_json::json!(palette);
            value["mirror_horizontal"] = true.into();
            let mut metadata: NativeFrame = serde_json::from_value(value)?;
            let mut import: SpriteImport = serde_json::from_value(serde_json::json!({
                "id": "fixture", "species_id": "MN_011", "view": "front",
                "source": "sprites/fixture.png", "crop": [0, 0, 20, 20], "fit": [10, 10],
                "normal_palette_rgb555": palette, "mirror_horizontal": true,
                "sampling": sampling
            }))?;
            verify_source_conversion(&source, &import, &metadata)?;
            import.mirror_horizontal = false;
            assert!(verify_source_conversion(&source, &import, &metadata).is_err());
            import.mirror_horizontal = true;
            metadata.rows[0].replace_range(0..1, "3");
            assert!(verify_source_conversion(&source, &import, &metadata).is_err());
            metadata.rows = frame.rows;
            metadata.quantization_palette_rgb555 = None;
            assert!(verify_source_conversion(&source, &import, &metadata).is_err());
            metadata.quantization_palette_rgb555 = Some(palette);
            metadata.sampling = match sampling {
                SpriteSampling::AreaVote => SpriteSampling::SourceCenter,
                SpriteSampling::SourceCenter => SpriteSampling::AreaVote,
            };
            assert!(verify_source_conversion(&source, &import, &metadata).is_err());
            metadata.sampling = sampling;
            let edit = crate::minnesota_sprite_refinement::PixelEdit {
                x: 0,
                y: 0,
                from: 0,
                to: 3,
            };
            import.pixel_edits = vec![edit];
            metadata.pixel_edits = vec![edit];
            assert!(
                verify_source_conversion(&source, &import, &metadata).is_err(),
                "declared edits must be replayed"
            );
            crate::minnesota_sprite_refinement::apply(&mut metadata.rows, &[edit])?;
            verify_source_conversion(&source, &import, &metadata)?;
            import.pixel_edits.clear();
            assert!(
                verify_source_conversion(&source, &import, &metadata).is_err(),
                "undeclared painting rejected"
            );
        }
        Ok(())
    }

    #[test]
    fn backs_use_column_major_tiles_and_reject_bad_indices() {
        let mut pixels = vec![0; 48 * 48];
        pixels[8] = 1;
        let front_order = encode_bitplanes(&pixels, 48, false);
        let back_order = encode_bitplanes(&pixels, 48, true);
        assert_eq!(front_order[16], 128);
        assert_eq!(back_order[6 * 16], 128);
        assert_ne!(front_order, back_order);
        assert!(parse_rows(&["x".to_string()], 1).is_err());
        assert!(parse_rows(&[".".to_string()], 1).is_err());
    }
}
