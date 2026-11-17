//! Golden prairie ground derived from native glyphs and the native gold palette.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

const GOLD_PALETTE: u8 = 7;

/// Adds two regional art variants with golden lawn and dark-stemmed tall grass.
///
/// Native Kanto01/0b become80/81 in `mn_prairie_kanto`; native Headwaters02/03
/// become83/84 in `mn_prairie_headwaters`. Lawn detail pixels use the visible
/// gold ink instead of the palette's indistinguishable background color. All
/// original metatiles, palettes, collision and source art remain intact. New
/// blocks retain their source FLOOR/TALL_GRASS collision. Cut and Whirlpool
/// replacement behavior is inherited through the ordinary extension boundary.
/// This adds art only; callers must explicitly author and publish field terrain.
///
/// # Errors
/// Returns an error for missing/incompatible native art, occupied identities,
/// unexpected source grass collision, insufficient unused glyph slots or failed
/// pack validation. The borrowed base is unchanged on success and failure.
pub fn build_prairie_grass_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let pack = append_grass(base, "kanto", "mn_prairie_kanto", [0x01, 0x0b], 128)?;
    append_grass(
        &pack,
        "mn_headwaters",
        "mn_prairie_headwaters",
        [0x02, 0x03],
        131,
    )
}

/// Repair the derived lawn glyphs in a pack that already contains prairie art.
///
/// Only the prairie lawn's palette indices and its generated PNG are changed.
/// Maps, collision, encounters, audio, and the source tilesets remain intact.
///
/// # Errors
/// Returns an error if the existing art is missing, does not match the native
/// source glyphs, shares a lawn glyph with another block, or fails validation.
pub fn repair_prairie_lawn_art(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let mut pack = base.clone();
    for (source, target, source_block, target_block) in [
        ("kanto", "mn_prairie_kanto", 0x01_usize, 0x80_usize),
        (
            "mn_headwaters",
            "mn_prairie_headwaters",
            0x02_usize,
            0x83_usize,
        ),
    ] {
        let layout_path = |name: &str| format!("data/tilesets/{name}_metatiles.bin");
        let graphics_path = |name: &str| format!("gfx/tilesets/{name}.2bpp");
        let old_layout = base
            .runtime_files
            .get(&layout_path(source))
            .context("source metatiles")?;
        let layout = base
            .runtime_files
            .get(&layout_path(target))
            .context("prairie metatiles")?;
        let old_graphics = base
            .runtime_files
            .get(&graphics_path(source))
            .context("source glyphs")?;
        let old_definition = base.data.tilesets.get(source).context("source tileset")?;
        let definition = base.data.tilesets.get(target).context("prairie tileset")?;
        let source_glyphs = old_layout
            .get(source_block * 16..(source_block + 1) * 16)
            .context("source lawn")?;
        let target_glyphs = layout
            .get(target_block * 16..(target_block + 1) * 16)
            .context("prairie lawn")?;
        let mut graphics = base
            .runtime_files
            .get(&graphics_path(target))
            .context("prairie glyphs")?
            .clone();
        for (&original, &glyph) in source_glyphs.iter().zip(target_glyphs) {
            ensure!(
                layout
                    .iter()
                    .enumerate()
                    .all(|(offset, value)| *value != glyph || offset / 16 == target_block),
                "prairie lawn glyph is shared by another block"
            );
            let original_slot =
                physical(original, old_definition.palette_map[usize::from(original)]);
            let target_palette = definition.palette_map[usize::from(glyph)];
            ensure!(
                target_palette & 7 == GOLD_PALETTE,
                "prairie lawn palette changed"
            );
            let target_slot = physical(glyph, target_palette);
            let original_bytes = old_graphics
                .get(original_slot * 16..(original_slot + 1) * 16)
                .context("source lawn pixels")?;
            let target_bytes = graphics
                .get_mut(target_slot * 16..(target_slot + 1) * 16)
                .context("prairie lawn pixels")?;
            let mut repaired = original_bytes.to_vec();
            restore_gold_lawn_detail(&mut repaired);
            ensure!(
                target_bytes == original_bytes || *target_bytes == repaired,
                "prairie lawn pixels differ from source"
            );
            target_bytes.copy_from_slice(&repaired);
        }
        pack.runtime_files.insert(
            format!("gfx/tilesets/{target}.png"),
            native_tile_png::from_2bpp(&graphics)?,
        );
        pack.runtime_files.insert(graphics_path(target), graphics);
    }
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

fn physical(glyph: u8, palette: u8) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 0x7f) + 96
    }
}

fn append_grass(
    base: &CompiledGamePack,
    source: &str,
    target: &str,
    grass: [usize; 2],
    expected_blocks: usize,
) -> Result<CompiledGamePack> {
    ensure!(
        !base
            .runtime_files()
            .contains_key(&format!("gfx/tilesets/{source}.pal")),
        "prairie variants require the shared native palette"
    );
    let mut definition = base
        .data()
        .tilesets
        .get(source)
        .context("native prairie source tileset")?
        .clone();
    let files = base.runtime_files();
    let mut metatiles = files
        .get(&format!("data/tilesets/{source}_metatiles.bin"))
        .context("native prairie source metatiles")?
        .clone();
    let mut graphics = files
        .get(&format!("gfx/tilesets/{source}.2bpp"))
        .context("native prairie glyphs")?
        .clone();
    ensure!(
        metatiles.len() == expected_blocks * 16 && [96 * 16, 192 * 16].contains(&graphics.len()),
        "prairie native art extent changed"
    );
    let original_slots = graphics.len() / 16;
    // Kanto has one96-glyph bank; the authored variant can add a second bank
    // without changing any source metatile or used glyph/palette address.
    graphics.resize(192 * 16, 0);
    let source_graphics = graphics.clone();
    let used = metatiles.iter().copied().collect::<BTreeSet<_>>();
    let protected = used
        .iter()
        .map(|&glyph| physical(glyph, definition.palette_map[usize::from(glyph)]))
        .collect::<BTreeSet<_>>();
    ensure!(
        protected.iter().all(|slot| *slot < original_slots),
        "source grass art addresses a missing native glyph bank"
    );
    let mut free = (0..192)
        .filter(|slot| !protected.contains(slot))
        .filter_map(|slot| {
            let (logical, bank) = if slot < 96 {
                (slot, 0)
            } else {
                ((slot - 96) | 0x80, 8)
            };
            let glyph = u8::try_from(logical).ok()?;
            (!used.contains(&glyph)).then_some((glyph, slot, bank))
        });
    let mut replacements = BTreeMap::new();
    for (index, block) in grass.into_iter().enumerate() {
        let collision = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("native prairie collision")?
            .clone();
        let expected = if index == 0 { "FLOOR" } else { "TALL_GRASS" };
        ensure!(
            collision.iter().all(|kind| kind == expected),
            "native prairie field collision changed"
        );
        let source = metatiles
            .get(block * 16..(block + 1) * 16)
            .context("native prairie block")?
            .to_vec();
        for glyph in source {
            let replacement = if let Some(&replacement) = replacements.get(&glyph) {
                replacement
            } else {
                let palette = *definition
                    .palette_map
                    .get(usize::from(glyph))
                    .context("native grass palette")?;
                let source_slot = physical(glyph, palette);
                let (new_glyph, slot, bank) = free
                    .next()
                    .context("no unused native glyph slot for golden prairie grass")?;
                graphics[slot * 16..(slot + 1) * 16].copy_from_slice(
                    source_graphics
                        .get(source_slot * 16..(source_slot + 1) * 16)
                        .context("native grass pixels")?,
                );
                if index == 0 {
                    restore_gold_lawn_detail(&mut graphics[slot * 16..(slot + 1) * 16]);
                }
                definition.palette_map[usize::from(new_glyph)] = bank | GOLD_PALETTE;
                replacements.insert(glyph, new_glyph);
                new_glyph
            };
            metatiles.push(replacement);
        }
        definition
            .collision
            .insert(format!("{:02x}", expected_blocks + index), collision);
    }
    let png = native_tile_png::from_2bpp(&graphics)?;
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: format!("minnesota-{target}-art-v1"),
        tileset_id: target.into(),
        behavior_source_tileset_id: Some(source.into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: png,
    })
}

// Native palette 7 assigns the same color to indices 0 and 1. Move only
// index-1 pixels to the visible gold ink at index 2, preserving
// the location of each native lawn mark and leaving other pixels untouched.
fn restore_gold_lawn_detail(glyph: &mut [u8]) {
    for row in glyph.chunks_exact_mut(2) {
        let index_one = row[0] & !row[1];
        row[0] &= !index_one;
        row[1] |= index_one;
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn fixture() -> CompiledGamePack {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_PRAIRIE_GRASS_TEST_PACK")
                .expect("completed external geography pack"),
        );
        crate::read_verified_compiled_game_pack(path).expect("verified prairie source pack")
    }

    #[test]
    #[ignore = "requires the completed external geography pack"]
    fn golden_grass_copies_pixels_and_collision_without_recoloring_source_blocks() {
        let base = fixture();
        let output = build_prairie_grass_art_pack(&base).expect("golden variants");
        assert_eq!(base.data().maps, output.data().maps);
        assert_eq!(
            base.data().runtime_spawn_points,
            output.data().runtime_spawn_points
        );
        assert_eq!(base.audio_manifest, output.audio_manifest);
        for (name, bytes) in base.runtime_files() {
            assert_eq!(
                output.runtime_files().get(name),
                Some(bytes),
                "original file {name}"
            );
        }
        for (source, target, originals, first) in [
            ("kanto", "mn_prairie_kanto", [1_usize, 11], 128),
            ("mn_headwaters", "mn_prairie_headwaters", [2_usize, 3], 131),
        ] {
            let old = &base.runtime_files()[&format!("data/tilesets/{source}_metatiles.bin")];
            let new = &output.runtime_files()[&format!("data/tilesets/{target}_metatiles.bin")];
            assert_eq!(&new[..old.len()], old);
            let old_definition = &base.data().tilesets[source];
            let definition = &output.data().tilesets[target];
            let old_pixels = &base.runtime_files()[&format!("gfx/tilesets/{source}.2bpp")];
            let pixels = &output.runtime_files()[&format!("gfx/tilesets/{target}.2bpp")];
            for (index, block) in originals.into_iter().enumerate() {
                let source_glyphs = &old[block * 16..(block + 1) * 16];
                let golden_glyphs = &new[(first + index) * 16..(first + index + 1) * 16];
                for (&a, &b) in source_glyphs.iter().zip(golden_glyphs) {
                    let old_slot = physical(a, old_definition.palette_map[usize::from(a)]);
                    let new_palette = definition.palette_map[usize::from(b)];
                    let slot = physical(b, new_palette);
                    assert_eq!(new_palette & 7, GOLD_PALETTE);
                    let mut expected = old_pixels[old_slot * 16..(old_slot + 1) * 16].to_vec();
                    if index == 0 {
                        restore_gold_lawn_detail(&mut expected);
                    }
                    assert_eq!(&expected, &pixels[slot * 16..(slot + 1) * 16]);
                }
                assert_eq!(
                    old_definition.collision[&format!("{block:02x}")],
                    definition.collision[&format!("{:02x}", first + index)]
                );
            }
        }
    }

    #[test]
    fn gold_lawn_detail_preserves_pixel_positions() {
        let mut glyph = [0_u8; 16];
        glyph[0] = 0b0101_0101;
        glyph[1] = 0b0011_0011;
        restore_gold_lawn_detail(&mut glyph);
        assert_eq!(glyph[0], 0b0001_0001);
        assert_eq!(glyph[1], 0b0111_0111);
    }

    #[test]
    #[ignore = "requires an installed external Northstar pack"]
    fn installed_prairie_lawn_repair_preserves_gameplay_and_is_idempotent() {
        let base = crate::read_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_PRAIRIE_INSTALLED_PACK").expect("installed pack"),
        )
        .expect("verified installed pack");
        let repaired = repair_prairie_lawn_art(&base).expect("repair installed prairie art");
        assert_eq!(repaired.data, base.data);
        assert_eq!(repaired.audio_manifest, base.audio_manifest);
        assert_eq!(
            repair_prairie_lawn_art(&repaired).expect("repeat repair"),
            repaired
        );
    }

    #[test]
    #[ignore = "requires the completed external geography pack"]
    fn golden_grass_rejects_wrong_collision_before_publishing() {
        let mut base = fixture();
        base.data
            .tilesets
            .get_mut("kanto")
            .expect("source")
            .collision
            .insert(
                "0b".into(),
                vec![
                    "FLOOR".into(),
                    "FLOOR".into(),
                    "FLOOR".into(),
                    "FLOOR".into(),
                ],
            );
        let before = base.clone();
        let error = build_prairie_grass_art_pack(&base)
            .expect_err("encounter collision cannot silently become ordinary ground");
        assert!(format!("{error:#}").contains("native prairie field collision changed"));
        assert_eq!(base, before);
    }

    #[test]
    #[ignore = "requires the completed external geography pack"]
    fn golden_grass_rejects_occupied_targets_without_mutating_base() {
        let base = build_prairie_grass_art_pack(&fixture()).expect("first publication");
        let before = base.clone();
        assert!(build_prairie_grass_art_pack(&base).is_err());
        assert_eq!(base, before);
    }
}
