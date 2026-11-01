//! Golden prairie ground from unchanged native glyphs and the native gold palette.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

const GOLD_PALETTE: u8 = 7;

/// Adds two regional art variants with golden lawn and dark-stemmed tall grass.
///
/// Native Kanto01/0b become80/81 in `mn_prairie_kanto`; native Headwaters02/03
/// become83/84 in `mn_prairie_headwaters`. Source pixels are copied exactly into
/// unused glyph slots and assigned the existing gold background palette. All
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
                    assert_eq!(
                        &old_pixels[old_slot * 16..(old_slot + 1) * 16],
                        &pixels[slot * 16..(slot + 1) * 16]
                    );
                }
                assert_eq!(
                    old_definition.collision[&format!("{block:02x}")],
                    definition.collision[&format!("{:02x}", first + index)]
                );
            }
        }
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
