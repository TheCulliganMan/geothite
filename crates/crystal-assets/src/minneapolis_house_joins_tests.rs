use std::path::PathBuf;

use serde_json::Value;

use super::{MAP, TILESET, build_minneapolis_house_joins_pack, parse_contract, preserve};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/minneapolis-houses-v01/source-v04/native-joins.contract.json"
);
const INVENTORY: &str = include_str!(
    "../../../planning/minnesota/revisions/minneapolis-houses-v01/exterior-inventory-v01.json"
);

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_MINNEAPOLIS_JOINS_TEST_PACK")
            .expect("published first river-bend parent path"),
    );
    crate::read_verified_compiled_game_pack(path).expect("verified first-bend parent")
}

fn integer(value: &Value) -> usize {
    usize::try_from(value.as_u64().expect("native inventory integer"))
        .expect("native inventory extent")
}

#[test]
#[ignore = "requires exact external first river-bend parent"]
fn minneapolis_joins_close_all_reported_facade_seams_without_upper_floor_foundations() {
    let base = fixture();
    let output =
        build_minneapolis_house_joins_pack(&base, CONTRACT).expect("reviewed join correction");
    let plan = parse_contract(&base, CONTRACT).expect("frozen contract");
    preserve(&base, &output, &plan).expect("full native residual proof");
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let old = &base.runtime_files[&path];
    let new = &output.runtime_files[&path];
    assert_eq!(&new[..old.len()], old);
    assert_eq!(new.len(), 169 * 16);
    let original_glyph = |block: usize, x: usize, y: usize| old[block * 16 + y * 4 + x];
    let left = original_glyph(0x10, 0, 0);
    let right = original_glyph(0x11, 3, 0);
    let ground_only = [
        original_glyph(0x10, 0, 3),
        original_glyph(0x10, 1, 3),
        original_glyph(0x11, 3, 3),
        original_glyph(0x20, 0, 3),
        original_glyph(0x20, 1, 3),
    ];
    let inventory: Vec<Value> = serde_json::from_str(INVENTORY).expect("forty-front inventory");
    assert_eq!(inventory.len(), 40);
    for record in &inventory {
        let bounds = record["footprint_metatiles"]
            .as_array()
            .expect("native footprint");
        let (x0, y0, x1, y1) = (
            integer(&bounds[0]),
            integer(&bounds[1]),
            integer(&bounds[2]),
            integer(&bounds[3]),
        );
        let width = (x1 - x0) * 4;
        let height = (y1 - y0) * 4;
        let glyph = |x: usize, y: usize| {
            let block = usize::from(output.data.maps[MAP].blocks[(y0 + y / 4) * 40 + x0 + x / 4]);
            new[block * 16 + (y % 4) * 4 + x % 4]
        };
        let first = base.data.maps[MAP].blocks[y0 * 40 + x0];
        if first == 0x18 {
            for y in 0..4 {
                assert_eq!(
                    glyph(0, y),
                    original_glyph(0x18, 0, y),
                    "{} left roof",
                    record["id"]
                );
                assert_eq!(
                    glyph(width - 1, y),
                    original_glyph(0x19, 3, y),
                    "{} right roof",
                    record["id"]
                );
                for x in 1..width - 1 {
                    assert_eq!(
                        glyph(x, y),
                        original_glyph(0x18, 1, y),
                        "{} interior roof",
                        record["id"]
                    );
                }
            }
            for y in 4..height - 4 {
                assert_eq!(glyph(0, y), left, "{} upper left", record["id"]);
                assert_eq!(glyph(width - 1, y), right, "{} upper right", record["id"]);
                for x in 1..width - 1 {
                    assert_ne!(glyph(x, y), left, "{} interior left cap", record["id"]);
                    assert_ne!(glyph(x, y), right, "{} interior right cap", record["id"]);
                    assert!(
                        !ground_only.contains(&glyph(x, y)),
                        "{} floating foundation",
                        record["id"]
                    );
                }
            }
        } else if first == 0x80 {
            for y in 0..4 {
                assert_eq!(
                    glyph(width - 1, height - 4 + y),
                    original_glyph(0x88, 3, y),
                    "{} missing masonry right cap",
                    record["id"]
                );
            }
        }
    }
    let old_definition = &base.data.tilesets[TILESET];
    let definition = &output.data.tilesets[TILESET];
    for (before, after) in base.data.maps[MAP]
        .blocks
        .iter()
        .zip(&output.data.maps[MAP].blocks)
    {
        assert_eq!(
            old_definition.collision[&format!("{before:02x}")],
            definition.collision[&format!("{after:02x}")]
        );
    }
    assert_eq!(base.data.maps[MAP].events, output.data.maps[MAP].events);
    assert!(build_minneapolis_house_joins_pack(&output, CONTRACT).is_err());
}

#[test]
#[ignore = "requires exact external first river-bend parent"]
fn minneapolis_joins_reject_stale_source_and_unreviewed_contract() {
    let base = fixture();
    let mut changed = base.clone();
    changed.data.maps.get_mut(MAP).expect("city").blocks[4 + 14 * 40] = 1;
    assert!(build_minneapolis_house_joins_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed
        .runtime_files
        .get_mut("data/tilesets/mn_minneapolis_city_metatiles.bin")
        .expect("native metatiles")[0] ^= 1;
    assert!(build_minneapolis_house_joins_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed
        .data
        .tilesets
        .get_mut(TILESET)
        .expect("city art")
        .palette_map[0] ^= 1;
    assert!(build_minneapolis_house_joins_pack(&changed, CONTRACT).is_err());
    assert!(build_minneapolis_house_joins_pack(&base, &(CONTRACT.to_owned() + " ")).is_err());
}

#[test]
#[ignore = "requires exact external first river-bend parent"]
fn minneapolis_joins_residual_rejects_other_geometry_programs_art_audio_and_reports() {
    let base = fixture();
    let output = build_minneapolis_house_joins_pack(&base, CONTRACT).expect("join correction");
    let plan = parse_contract(&base, CONTRACT).expect("contract");
    let mut changed = output.clone();
    changed.data.maps.get_mut(MAP).expect("city").blocks[0] ^= 1;
    assert!(preserve(&base, &changed, &plan).is_err());
    let mut changed = output.clone();
    changed
        .data
        .maps
        .get_mut(MAP)
        .expect("city")
        .scripts
        .insert("UnrelatedJoinScript".into(), serde_json::json!([]));
    assert!(preserve(&base, &changed, &plan).is_err());
    let mut changed = output.clone();
    changed
        .runtime_files
        .get_mut("gfx/tilesets/mn_minneapolis_city.2bpp")
        .expect("native glyphs")[0] ^= 1;
    assert!(preserve(&base, &changed, &plan).is_err());
    let mut changed = output.clone();
    changed
        .runtime_files
        .get_mut("data/tilesets/mn_minneapolis_city_metatiles.bin")
        .expect("recipes")[157 * 16] ^= 1;
    assert!(preserve(&base, &changed, &plan).is_err());
    let mut changed = output.clone();
    changed
        .data
        .tilesets
        .get_mut(TILESET)
        .expect("city art")
        .palette_map[0] ^= 1;
    assert!(preserve(&base, &changed, &plan).is_err());
    let mut changed = output.clone();
    changed
        .report
        .manifests
        .push("unrelated-join-manifest".into());
    assert!(preserve(&base, &changed, &plan).is_err());
    let mut changed = output.clone();
    let audio = changed
        .audio_manifest
        .music
        .values_mut()
        .next()
        .expect("actual music descriptor");
    audio.pcm_frame_count = Some(audio.pcm_frame_count.expect("source PCM count") + 1);
    assert!(preserve(&base, &changed, &plan).is_err());
}
