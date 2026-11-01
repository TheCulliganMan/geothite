//! Fixed-grid ImageGen imports for original, placeable Minnesota map props.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::minnesota_pokemon_pixel_art;
use crate::minnesota_sprite_sheet::MinnesotaSheet;
use crate::{CompiledGamePack, CompiledMapExtension, CompiledTilesetExtension};

/// Original source-sheet recipe. Paths are resolved by the publisher.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinnesotaTileCatalog {
    pub schema_version: u32,
    pub tileset_id: String,
    pub source_sha256: String,
    pub items: Vec<MinnesotaTileItem>,
}

/// One 32-pixel map block, with collision ordered NW, NE, SW, SE.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MinnesotaTileItem {
    pub id: String,
    pub name: String,
    pub crop: [usize; 4],
    /// Contiguous source-crop neighbors keyed by N/E/S/W. Reciprocal entries
    /// permit ink across that internal seam; exterior borders stay transparent.
    #[serde(default)]
    pub neighbors: BTreeMap<String, String>,
    /// Vote over an aligned 8x8 sample grid per native pixel. Requires alpha
    /// and an unsplit crop; source-backed dark contours need 12 of 64 samples.
    #[serde(default)]
    pub area_sampling: bool,
    /// Integer native placement adjustment, without silhouette rescaling.
    #[serde(default)]
    pub offset: [i8; 2],
    pub palette: u8,
    /// RGB colors used to classify the generated source before native recoloring.
    pub source_colors: [[u8; 3]; 4],
    /// Preserve source alpha instead of treating white paint as background.
    #[serde(default)]
    pub transparent_background: bool,
    /// Optional palettes for the sixteen 8px glyphs, in row-major order.
    #[serde(default)]
    pub tile_palettes: Option<[u8; 16]>,
    /// Source classification colors for glyphs using a different palette.
    #[serde(default)]
    pub source_colors_by_palette: BTreeMap<u8, [[u8; 3]; 4]>,
    /// Map near-neutral opaque ink to this index in the primary palette.
    /// Useful for charcoal around colored fire; chroma must be below 24.
    #[serde(default)]
    pub neutral_ink_index: Option<u8>,
    /// Reviewed source offset separating flame/stone halves, each mapped to 16 rows.
    #[serde(default)]
    pub source_row_split: Option<usize>,
    /// Optional stable RGB foreground colors by native palette. Background zero
    /// continues to follow the ordinary ground color at the current time.
    #[serde(default)]
    pub palette_foregrounds: BTreeMap<u8, [[u8; 3]; 3]>,
    pub collision: [String; 4],
    pub placement: String,
}

/// Native indexed pixels for a catalog entry; no rescaling of its silhouette.
#[derive(Debug)]
pub struct MinnesotaTilePixels {
    pub indices: Vec<u8>,
}

impl MinnesotaTileItem {
    /// Return a glyph's native palette; out-of-range positions use the primary palette.
    pub fn palette_for_tile(&self, tile: usize) -> u8 {
        self.tile_palettes
            .as_ref()
            .and_then(|values| values.get(tile))
            .copied()
            .unwrap_or(self.palette)
    }

    fn source_index(&self, sheet: &MinnesotaSheet, x: usize, y: usize, palette: u8) -> Result<u8> {
        if self.transparent_background && sheet.alpha(x, y).context("source alpha")? < 128 {
            return Ok(0);
        }
        let rgb = sheet.pixel(x, y).context("validated source pixel")?;
        if palette == self.palette
            && rgb.iter().max().context("RGB maximum")? - rgb.iter().min().context("RGB minimum")?
                < 24
            && let Some(index) = self.neutral_ink_index
        {
            return Ok(index);
        }
        let colors = self
            .source_colors_by_palette
            .get(&palette)
            .unwrap_or(&self.source_colors);
        let index = colors
            .iter()
            .enumerate()
            .filter(|(index, _)| !self.transparent_background || *index != 0)
            .min_by_key(|(_, color)| {
                rgb.iter()
                    .zip(color.iter())
                    .map(|(a, b)| {
                        let delta = i32::from(*a) - i32::from(*b);
                        delta * delta
                    })
                    .sum::<i32>()
            })
            .context("four source colors")?
            .0;
        Ok(u8::try_from(index)?)
    }

    fn area_index(
        &self,
        sheet: &MinnesotaSheet,
        x: usize,
        y: usize,
        step: usize,
        palette: u8,
    ) -> Result<u8> {
        let mut votes = [0_u8; 4];
        for row in 0..8 {
            for column in 0..8 {
                let index = self.source_index(
                    sheet,
                    x + (column * 2 + 1) * step / 16,
                    y + (row * 2 + 1) * step / 16,
                    palette,
                )?;
                votes[usize::from(index)] += 1;
            }
        }
        if votes[0] > 48 {
            return Ok(0);
        }
        if votes[3] >= 12 {
            return Ok(3);
        }
        let index = (1_u8..4)
            .max_by_key(|index| votes[usize::from(*index)])
            .context("three foreground indices")?;
        Ok(index)
    }
}

impl MinnesotaTileCatalog {
    fn validate_neighbors(&self) -> Result<()> {
        for item in &self.items {
            for (side, id) in &item.neighbors {
                let neighbor = self
                    .items
                    .iter()
                    .find(|candidate| candidate.id == *id)
                    .with_context(|| format!("item {} has unknown neighbor {id}", item.id))?;
                let [x, y, width, height] = item.crop;
                let [nx, ny, nw, nh] = neighbor.crop;
                let (opposite, adjoining) = match side.as_str() {
                    "N" => ("S", x == nx && ny.checked_add(nh) == Some(y)),
                    "E" => ("W", y == ny && x.checked_add(width) == Some(nx)),
                    "S" => ("N", x == nx && y.checked_add(height) == Some(ny)),
                    "W" => ("E", y == ny && nx.checked_add(nw) == Some(x)),
                    _ => anyhow::bail!("item {} has invalid neighbor side {side}", item.id),
                };
                ensure!(
                    item.id != *id
                        && adjoining
                        && width == nw
                        && height == nh
                        && item.offset == [0, 0]
                        && neighbor.offset == [0, 0]
                        && item.source_row_split.is_none()
                        && neighbor.source_row_split.is_none()
                        && neighbor.neighbors.get(opposite) == Some(&item.id),
                    "item {} neighbor {id} must be an unshifted reciprocal source seam",
                    item.id
                );
            }
        }
        Ok(())
    }

    /// Decode and check the exact original PNG before fixed-grid conversion.
    ///
    /// Errors on hash drift, malformed geometry, duplicate IDs, unsupported
    /// palettes/collision, invalid assembly seams, or source PNG errors. Every
    /// crop must be square and an integer enlargement of 32 native pixels.
    pub fn convert(&self, png: &[u8]) -> Result<Vec<MinnesotaTilePixels>> {
        ensure!(self.schema_version == 1, "unsupported tile catalog version");
        ensure!(
            format!("{:x}", Sha256::digest(png)) == self.source_sha256,
            "tile source SHA-256 differs from reviewed catalog"
        );
        ensure!(
            !self.items.is_empty() && self.items.len() <= 16,
            "tile item count"
        );
        self.validate_neighbors()?;
        let sheet = MinnesotaSheet::from_png(png)?;
        let mut ids = BTreeSet::new();
        self.items
            .iter()
            .map(|item| {
                ensure!(
                    !item.id.is_empty()
                        && item.id.len() <= 64
                        && item.id.bytes().all(|byte| byte.is_ascii_lowercase()
                            || byte.is_ascii_digit()
                            || byte == b'_')
                        && ids.insert(&item.id)
                        && item.palette < 8,
                    "duplicate/empty item ID or invalid palette"
                );
                ensure!(
                    item.collision
                        .iter()
                        .all(|value| matches!(value.as_str(), "FLOOR" | "WALL" | "COUNTER")),
                    "props support only explicit FLOOR/WALL/COUNTER collision"
                );
                ensure!(
                    item.tile_palettes
                        .as_ref()
                        .is_none_or(|palettes| palettes.iter().all(|value| *value < 8))
                        && item
                            .neutral_ink_index
                            .is_none_or(|index| (1..4).contains(&index))
                        && (item.tile_palettes.is_none() || item.offset == [0, 0])
                        && (!item.area_sampling
                            || (item.transparent_background && item.source_row_split.is_none()))
                        && item.palette_foregrounds.keys().all(|palette| *palette < 8),
                    "invalid per-glyph palette or shifted multi-palette item"
                );
                let [x, y, width, height] = item.crop;
                ensure!(
                    width > 0
                        && width == height
                        && width.is_multiple_of(32)
                        && x.checked_add(width).is_some_and(|end| end <= sheet.width)
                        && y.checked_add(height).is_some_and(|end| end <= sheet.height),
                    "item {} crop must be an in-bounds integer 32px grid",
                    item.id
                );
                let step = width / 32;
                ensure!(
                    item.source_row_split
                        .is_none_or(|split| split > 0 && split < height),
                    "source row split must stay inside the crop"
                );
                let mut indices = Vec::with_capacity(1024);
                // Both sampling modes preserve the fixed grid. Creature silhouette
                // fitting/contour generation would distort tiled seams.
                for row in 0..32 {
                    for column in 0..32 {
                        let sx = x + column * step + step / 2;
                        let sy = if let Some(split) = item.source_row_split {
                            if row < 16 {
                                y + (row * 2 + 1) * split / 32
                            } else {
                                y + split + ((row - 16) * 2 + 1) * (height - split) / 32
                            }
                        } else {
                            y + row * step + step / 2
                        };
                        let palette = item.palette_for_tile(row / 8 * 4 + column / 8);
                        let index = if item.area_sampling {
                            item.area_index(
                                &sheet,
                                x + column * step,
                                y + row * step,
                                step,
                                palette,
                            )?
                        } else {
                            item.source_index(&sheet, sx, sy, palette)?
                        };
                        indices.push(index);
                    }
                }
                if item.offset != [0, 0] {
                    let mut placed = vec![0; 1024];
                    for (n, &index) in indices.iter().enumerate() {
                        if index == 0 {
                            continue;
                        }
                        let column = (n % 32).checked_add_signed(isize::from(item.offset[0]));
                        let row = (n / 32).checked_add_signed(isize::from(item.offset[1]));
                        let (Some(column), Some(row)) = (column, row) else {
                            anyhow::bail!("item {} offset clips source ink", item.id);
                        };
                        ensure!(
                            column < 32 && row < 32,
                            "item {} offset clips source ink",
                            item.id
                        );
                        placed[row * 32 + column] = index;
                    }
                    indices = placed;
                }
                for (n, &index) in indices.iter().enumerate() {
                    let quadrant = (n / 32 / 16) * 2 + (n % 32 / 16);
                    ensure!(
                        index == 0 || item.collision[quadrant] != "FLOOR",
                        "item {} paints prop pixels into its walkable approach",
                        item.id
                    );
                }
                ensure!(
                    indices.iter().filter(|value| **value != 0).count() >= 16,
                    "item {} has no readable silhouette",
                    item.id
                );
                ensure!(
                    (0..32).all(|n| (item.neighbors.contains_key("N") || indices[n] == 0)
                        && (item.neighbors.contains_key("S") || indices[31 * 32 + n] == 0)
                        && (item.neighbors.contains_key("W") || indices[n * 32] == 0)
                        && (item.neighbors.contains_key("E") || indices[n * 32 + 31] == 0)),
                    "item {} clips its block boundary",
                    item.id
                );
                Ok(MinnesotaTilePixels { indices })
            })
            .collect()
    }
}

/// Build a separate Johto-compatible prop tileset with blocks 80 onward.
///
/// Expands the packed graphics to two 128-tile banks, relocating the old
/// bank-one pixels without changing their logical addresses. Existing blocks,
/// palettes and field-move replacement behavior are preserved.
///
/// Errors on catalog/source drift, occupied extension identity, unexpected
/// native Johto geometry, insufficient unused glyphs or pack validation.
pub fn build_minnesota_tile_art_pack(
    base: &CompiledGamePack,
    catalog: &MinnesotaTileCatalog,
    source: &[u8],
) -> Result<CompiledGamePack> {
    let pixels = catalog.convert(source)?;
    let files = base.runtime_files();
    let mut definition = base
        .data()
        .tilesets
        .get("johto")
        .context("Johto tileset")?
        .clone();
    let mut metatiles = files
        .get("data/tilesets/johto_metatiles.bin")
        .context("Johto blocks")?
        .clone();
    let original = files
        .get("gfx/tilesets/johto.2bpp")
        .context("Johto graphics")?;
    ensure!(
        metatiles.len() == 128 * 16 && original.len() == 192 * 16,
        "expected original Johto block extent and two 96-tile banks"
    );
    ensure!(
        !files.contains_key("gfx/tilesets/johto.pal"),
        "expected shared native background palette"
    );
    let referenced: BTreeSet<_> = metatiles.iter().copied().collect();
    ensure!(
        definition.palette_map.len() <= 256,
        "Johto palette-map extent"
    );
    definition.palette_map.resize(256, 0);
    let mut graphics = vec![0_u8; 256 * 16];
    graphics[..96 * 16].copy_from_slice(&original[..96 * 16]);
    graphics[128 * 16..224 * 16].copy_from_slice(&original[96 * 16..]);
    for glyph in &referenced {
        let palette = definition.palette_map[usize::from(*glyph)];
        ensure!(
            palette < 16 && (glyph & 0x7f) < 96 && (palette & 8 != 0 || *glyph < 96),
            "Johto glyph bank convention changed"
        );
    }
    let mut free = (0_u8..=255).filter(|glyph| !referenced.contains(glyph));
    let mut foregrounds = BTreeMap::new();
    let mut shared_glyphs = BTreeMap::new();
    for (offset, (item, pixels)) in catalog.items.iter().zip(&pixels).enumerate() {
        let block = 128 + offset;
        let encoded = minnesota_pokemon_pixel_art::bitplanes(&pixels.indices, 32, 32);
        for (tile, bytes) in encoded.as_chunks::<16>().0.iter().enumerate() {
            // Empty glyphs retain the exact native floor/grass art and its
            // palette instead of painting a flat rectangle around a prop.
            if bytes.iter().all(|byte| *byte == 0) {
                let ground = if item.palette_for_tile(tile) == 2 {
                    2
                } else {
                    1
                };
                metatiles.push(metatiles[ground * 16 + tile]);
                continue;
            }
            let key = (
                *bytes,
                item.palette_for_tile(tile),
                item.palette_foregrounds
                    .get(&item.palette_for_tile(tile))
                    .copied(),
            );
            if let Some(&glyph) = shared_glyphs.get(&key) {
                metatiles.push(glyph);
                continue;
            }
            let glyph = free.next().context("insufficient unused Johto glyphs")?;
            shared_glyphs.insert(key, glyph);
            graphics[usize::from(glyph) * 16..(usize::from(glyph) + 1) * 16].copy_from_slice(bytes);
            definition.palette_map[usize::from(glyph)] =
                item.palette_for_tile(tile) | if glyph >= 128 { 8 } else { 0 };
            if let Some(colors) = item.palette_foregrounds.get(&item.palette_for_tile(tile)) {
                foregrounds.insert(glyph, *colors);
            }
            metatiles.push(glyph);
        }
        ensure!(
            definition
                .collision
                .insert(format!("{block:02x}"), item.collision.to_vec())
                .is_none(),
            "prop block collision address occupied"
        );
    }
    let mut pack = base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: format!("{}-imagegen-v1", catalog.tileset_id.replace('_', "-")),
        tileset_id: catalog.tileset_id.clone(),
        behavior_source_tileset_id: Some("johto".into()),
        definition,
        metatiles,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    })?;
    if !foregrounds.is_empty() {
        pack.runtime_files.insert(
            format!("data/tilesets/{}_foregrounds.json", catalog.tileset_id),
            serde_json::to_vec(&foregrounds)?,
        );
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
    }
    Ok(pack)
}

impl CompiledGamePack {
    /// Add the disposable fair-art gallery with an existing town-map landmark.
    ///
    /// This fixture keeps the campaign start and existing maps unchanged. It
    /// reuses New Bark's landmark solely to satisfy the native shell's exact
    /// map-to-landmark lookup; it does not register a new regional destination.
    /// Returns an error for another map identity/tileset, a changed new-game
    /// start, missing source landmark, or invalid map/pack data.
    pub fn with_minnesota_tile_gallery(&self, extension: CompiledMapExtension) -> Result<Self> {
        ensure!(
            extension.map_name == "MnFairArtGallery"
                && extension.module.attributes.tileset_name == "mn_state_fair_props"
                && !extension.start_new_game_here,
            "expected a disposable fair-art gallery preserving the campaign start"
        );
        let landmark = self
            .data
            .pokegear_landmarks
            .map_to_landmark
            .get("NewBarkTown")
            .context("native gallery landmark template")?
            .clone();
        let mut pack = self.with_map_extension(extension)?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert("MnFairArtGallery".into(), landmark);
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
        Ok(pack)
    }
}

/// Encode a native preview in a caller-supplied palette and integer enlargement.
///
/// Errors on malformed pixel counts, indices, palette values or scale.
pub fn minnesota_tile_preview(
    pixels: &MinnesotaTilePixels,
    palette: [u16; 4],
    scale: usize,
) -> Result<Vec<u8>> {
    ensure!(
        pixels.indices.len() == 1024
            && pixels.indices.iter().all(|index| *index < 4)
            && palette.iter().all(|color| *color < 0x8000)
            && (1..=16).contains(&scale),
        "invalid preview pixels, palette or scale"
    );
    let native = minnesota_pokemon_pixel_art::colored(&pixels.indices, palette);
    let width = 32 * scale;
    let rgb = (0..width * width)
        .map(|n| native[(n / width / scale) * 32 + n % width / scale])
        .collect::<Vec<_>>();
    minnesota_pokemon_pixel_art::png_rgb(width, width, &rgb)
}

/// Encode an item using its per-glyph native day palettes and source alpha mask.
///
/// Transparent items have RGBA output with no painted ground. Errors on bad
/// indices, scale, or a palette lacking both explicit foreground colors and a
/// supported gray/green/fire/wood fallback.
pub fn minnesota_tile_item_preview(
    pixels: &MinnesotaTilePixels,
    item: &MinnesotaTileItem,
    scale: usize,
) -> Result<Vec<u8>> {
    ensure!(
        pixels.indices.len() == 1024
            && pixels.indices.iter().all(|index| *index < 4)
            && (1..=16).contains(&scale),
        "invalid item preview pixels or scale"
    );
    let color = |r: u16, g: u16, b: u16| r | (g << 5) | (b << 10);
    let mut native = Vec::with_capacity(1024);
    for (n, &index) in pixels.indices.iter().enumerate() {
        if item.transparent_background && index == 0 {
            native.push([0, 0, 0, 0]);
            continue;
        }
        let palette_id = item.palette_for_tile(n / 32 / 8 * 4 + n % 32 / 8);
        if index != 0
            && let Some(colors) = item.palette_foregrounds.get(&palette_id)
        {
            let [r, g, b] = colors[usize::from(index - 1)];
            native.push([r, g, b, 255]);
            continue;
        }
        let palette = match palette_id {
            0 => [
                color(27, 31, 27),
                color(21, 21, 21),
                color(13, 13, 13),
                color(7, 7, 7),
            ],
            2 => [
                color(22, 31, 10),
                color(12, 25, 1),
                color(5, 14, 0),
                color(7, 7, 7),
            ],
            4 => [
                color(27, 31, 27),
                color(31, 31, 7),
                color(31, 16, 1),
                color(7, 7, 7),
            ],
            5 => [
                color(27, 31, 27),
                color(24, 18, 7),
                color(20, 15, 3),
                color(7, 7, 7),
            ],
            _ => anyhow::bail!("unsupported item preview palette"),
        };
        let [r, g, b] = minnesota_pokemon_pixel_art::colored(&[index], palette)[0];
        native.push([r, g, b, 255]);
    }
    let width = 32 * scale;
    let rgba = (0..width * width)
        .map(|n| native[n / width / scale * 32 + n % width / scale])
        .collect::<Vec<_>>();
    if item.transparent_background {
        minnesota_pokemon_pixel_art::png_rgba(width, width, &rgba)
    } else {
        let rgb = rgba
            .iter()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<Vec<_>>();
        minnesota_pokemon_pixel_art::png_rgb(width, width, &rgb)
    }
}
