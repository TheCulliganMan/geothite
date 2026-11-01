use std::path::PathBuf;

use serde_json::Value;

use super::{build_winter_outdoor_art_pack, physical, remap, validate_residual};
use crate::CompiledGamePack;

const SPECIFICATION: &str =
    include_str!("../../../planning/minnesota/regions/ice/winter-art-v01.json");

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_WINTER_ART_TEST_PACK")
            .expect("completed external Minnesota source pack"),
    );
    crate::read_verified_compiled_game_pack(path).expect("verified winter source pack")
}

#[test]
fn grass_preserves_every_native_zero_hole_while_snow_remaps_its_background() {
    let original = [0b0001_1011, 0b0010_0111].repeat(8);
    let mut grass: [u8; 16] = original.as_slice().try_into().expect("one glyph");
    remap(&mut grass, [0, 1, 2, 3]);
    assert_eq!(grass.as_slice(), original);
    let mut snow = grass;
    remap(&mut snow, [1, 2, 3, 3]);
    for row in 0..8 {
        assert_eq!(snow[row * 2] | snow[row * 2 + 1], 255);
    }
}

#[test]
#[ignore = "requires completed external Minnesota pack"]
fn winter_art_retains_all_original_content_and_native_grass_priority() {
    let base = fixture();
    let output = build_winter_outdoor_art_pack(&base, SPECIFICATION).expect("winter art");
    assert_eq!(base.data.maps, output.data.maps);
    assert_eq!(base.audio_manifest, output.audio_manifest);
    assert_eq!(base.compiled_audio, output.compiled_audio);
    for (path, bytes) in base.runtime_files() {
        assert_eq!(
            output.runtime_files().get(path),
            Some(bytes),
            "original {path}"
        );
    }
    validate_residual(&base, &output).expect("only exact art/behavior/manifest additions");
    let mut damaged = output.clone();
    damaged.report.manifests.push("unrelated-manifest".into());
    assert!(validate_residual(&base, &damaged).is_err());
    let mut damaged = output.clone();
    let rule = damaged
        .data
        .field_moves
        .cut
        .replacements
        .values()
        .next()
        .expect("native Cut rules")
        .clone();
    damaged
        .data
        .field_moves
        .cut
        .replacements
        .insert("unrelated-source".into(), rule);
    assert!(validate_residual(&base, &damaged).is_err());
    for (source, target, first) in [
        ("mn_border_pines", "mn_winter_outdoor", 60),
        ("mn_north_shore", "mn_winter_north_shore", 131),
    ] {
        let old = &base.runtime_files()[&format!("data/tilesets/{source}_metatiles.bin")];
        let new = &output.runtime_files()[&format!("data/tilesets/{target}_metatiles.bin")];
        assert_eq!(&new[..old.len()], old);
        let old_definition = &base.data.tilesets[source];
        let new_definition = &output.data.tilesets[target];
        assert_eq!(old_definition.palette_map.len(), 224);
        assert_eq!(new_definition.palette_map.len(), 256);
        assert_eq!(
            &new_definition.palette_map[..224],
            old_definition.palette_map
        );
        let appended = &new[old.len()..];
        assert!(appended.iter().all(|glyph| *glyph >= 224));
        for glyph in 224_u8..=255 {
            let expected = if appended.contains(&glyph) { 11 } else { 0 };
            assert_eq!(new_definition.palette_map[usize::from(glyph)], expected);
        }
        let old_pixels = &base.runtime_files()[&format!("gfx/tilesets/{source}.2bpp")];
        let new_pixels = &output.runtime_files()[&format!("gfx/tilesets/{target}.2bpp")];
        for &glyph in old {
            let old_attribute = old_definition.palette_map[usize::from(glyph)];
            let new_attribute = new_definition.palette_map[usize::from(glyph)];
            assert_eq!(old_attribute, new_attribute);
            let old_slot = physical(glyph, old_attribute, 192);
            let new_slot = physical(glyph, new_attribute, 256);
            assert_eq!(
                &old_pixels[old_slot * 16..(old_slot + 1) * 16],
                &new_pixels[new_slot * 16..(new_slot + 1) * 16]
            );
        }
        // Exact source indices also preserve every priority zero hole. Merely
        // keeping collision TALL_GRASS cannot establish native sprite visibility.
        let johto = &base.data.tilesets["johto"];
        let grass = &base.runtime_files()["data/tilesets/johto_metatiles.bin"][3 * 16..4 * 16];
        let johto_pixels = &base.runtime_files()["gfx/tilesets/johto.2bpp"];
        for (&old_glyph, &new_glyph) in grass.iter().zip(&new[(first + 1) * 16..(first + 2) * 16]) {
            let a = physical(old_glyph, johto.palette_map[usize::from(old_glyph)], 192);
            let b = physical(
                new_glyph,
                new_definition.palette_map[usize::from(new_glyph)],
                256,
            );
            assert_eq!(
                &johto_pixels[a * 16..(a + 1) * 16],
                &new_pixels[b * 16..(b + 1) * 16]
            );
        }
        let collision = &output.data.tilesets[target].collision;
        for (offset, kind) in ["FLOOR", "TALL_GRASS", "ICE", "WALL", "WALL"]
            .into_iter()
            .enumerate()
        {
            assert!(
                collision[&format!("{:02x}", first + offset)]
                    .iter()
                    .all(|v| v == kind)
            );
        }
    }
}

#[test]
#[ignore = "requires completed external Minnesota pack"]
fn winter_art_rejects_grass_occlusion_blank_ice_and_source_drift() {
    let base = fixture();
    for (index, map) in [(1, [1, 2, 3, 3]), (2, [1, 1, 1, 3])] {
        let mut spec: Value = serde_json::from_str(SPECIFICATION).expect("source specification");
        spec["recipes"][index]["index_map"] = serde_json::json!(map);
        let error = build_winter_outdoor_art_pack(&base, &spec.to_string())
            .expect_err("cannot hide player or erase ice stripes");
        assert!(format!("{error:#}").contains("priority holes and ice stripes"));
    }
    let mut damaged = base.clone();
    damaged
        .runtime_files
        .get_mut("gfx/tilesets/ice_path.2bpp")
        .expect("native source")[0] ^= 1;
    assert!(build_winter_outdoor_art_pack(&damaged, SPECIFICATION).is_err());
    let mut damaged = base.clone();
    damaged
        .data
        .tilesets
        .get_mut("johto")
        .expect("native Johto")
        .collision
        .insert("03".into(), vec!["FLOOR".into(); 4]);
    assert!(build_winter_outdoor_art_pack(&damaged, SPECIFICATION).is_err());
    let first = build_winter_outdoor_art_pack(&base, SPECIFICATION).expect("first addition");
    assert!(build_winter_outdoor_art_pack(&first, SPECIFICATION).is_err());
}
