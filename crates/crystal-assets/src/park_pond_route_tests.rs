use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use super::{MAP, TEXT, TILESET, build_park_pond_route_pack, hash, parse_contract, preserve};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/r04-route-quality-v01/source-v05/native.contract.json"
);

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_PARK_POND_TEST_PACK")
            .expect("exact published long-river parent path"),
    );
    assert_eq!(
        hash(&std::fs::read(&path).expect("external parent bytes")),
        super::BASE_FILE
    );
    crate::read_verified_compiled_game_pack(path).expect("verified long-river parent")
}

fn kind(pack: &CompiledGamePack, x: i32, y: i32) -> &str {
    let x = usize::try_from(x).expect("nonnegative core x");
    let y = usize::try_from(y).expect("nonnegative core y");
    let block = pack.data.maps[MAP].blocks[(y / 2) * 28 + x / 2];
    &pack.data.tilesets[TILESET].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn grass_cost(pack: &CompiledGamePack, start: (i32, i32), target: (i32, i32)) -> Option<u16> {
    let occupied = pack.data.maps[MAP]
        .objects
        .iter()
        .map(|o| (i32::from(o.x), i32::from(o.y)))
        .collect::<BTreeSet<_>>();
    let mut todo = VecDeque::from([start]);
    let mut cost = BTreeMap::from([(start, 0)]);
    while let Some((x, y)) = todo.pop_front() {
        for next in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if !(0..56).contains(&next.0) || !(0..36).contains(&next.1) || occupied.contains(&next)
            {
                continue;
            }
            let collision = kind(pack, next.0, next.1);
            if !matches!(collision, "FLOOR" | "TALL_GRASS" | "LONG_GRASS") {
                continue;
            }
            let extra = u16::from(collision != "FLOOR");
            let proposed = cost[&(x, y)] + extra;
            if proposed < cost.get(&next).copied().unwrap_or(u16::MAX) {
                cost.insert(next, proposed);
                if extra == 0 {
                    todo.push_front(next);
                } else {
                    todo.push_back(next);
                }
            }
        }
    }
    cost.get(&target).copied()
}

#[test]
#[ignore = "requires exact external published long-river parent"]
fn park_pond_closes_ends_rails_bridge_and_keeps_paid_grass_and_native_art() {
    let base = fixture();
    let output = build_park_pond_route_pack(&base, CONTRACT).expect("reviewed park revision");
    let plan = parse_contract(&base, CONTRACT).expect("frozen combined plan");
    preserve(&base, &output, &plan).expect("full pack residual and uncached proof");
    let mut wire_data = base.data.clone();
    wire_data.maps.remove(MAP).expect("old park cache");
    let wire = serde_json::to_value(
        wire_data
            .assemble_map_module_from_compiled_payloads(MAP)
            .expect("old native assembly"),
    )
    .expect("old wire fields");
    let cached = serde_json::to_value(&base.data.maps[MAP]).expect("old cached fields");
    let inherited_differences = cached
        .as_object()
        .expect("cached module fields")
        .iter()
        .filter_map(|(key, value)| (wire.get(key) != Some(value)).then_some(key))
        .collect::<Vec<_>>();
    eprintln!("Inherited park cache/assembly fields: {inherited_differences:?}");
    assert_eq!(
        base.data.maps[MAP]
            .blocks
            .iter()
            .zip(&output.data.maps[MAP].blocks)
            .filter(|(a, b)| a != b)
            .count(),
        57
    );
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let old = &base.runtime_files[&path];
    let new = &output.runtime_files[&path];
    assert_eq!(&new[..old.len()], old);
    assert_eq!(new.len(), 72 * 16);
    for x in 0..56 {
        for y in [0, 35] {
            assert!(!kind(&output, x, y).starts_with("WATER"));
        }
    }
    for y in 0..36 {
        for x in [0, 55] {
            assert!(!kind(&output, x, y).starts_with("WATER"));
        }
    }
    for x in 26..32 {
        for y in [16, 17, 20, 21] {
            assert_eq!(kind(&output, x, y), "WALL");
        }
        for y in [18, 19] {
            assert_eq!(kind(&output, x, y), "FLOOR");
        }
    }
    for x in [24, 25, 32, 33] {
        for y in 17..21 {
            assert_eq!(kind(&output, x, y), kind(&base, x, y));
        }
    }
    for [x, y] in [[19, 8], [20, 8], [21, 8], [24, 8], [25, 8], [24, 10]] {
        assert_eq!(output.data.maps[MAP].blocks[y * 28 + x], 0);
    }
    assert!(
        output.data.tilesets[TILESET].collision["00"]
            .iter()
            .all(|q| q == "WALL")
    );
    for y in 0..18 {
        for x in 0..12 {
            assert_eq!(
                base.data.maps[MAP].blocks[y * 28 + x],
                output.data.maps[MAP].blocks[y * 28 + x]
            );
        }
    }
    for y in 12..18 {
        for x in 18..28 {
            assert_eq!(
                base.data.maps[MAP].blocks[y * 28 + x],
                output.data.maps[MAP].blocks[y * 28 + x]
            );
        }
    }
    assert_eq!(output.data.maps[MAP].blocks[12 * 28 + 24], 0x0e);
    assert_eq!(grass_cost(&base, (42, 27), (48, 27)), Some(4));
    assert_eq!(grass_cost(&output, (42, 27), (48, 27)), Some(4));
    assert!(grass_cost(&output, (0, 18), (55, 18)).is_some());
    assert_eq!(
        output.data.map_scripts[TEXT][0]["args"][0],
        "\"PARK POND BRIDGE\""
    );
    assert_eq!(base.data.maps[MAP].events, output.data.maps[MAP].events);
    assert_eq!(base.data.maps[MAP].objects, output.data.maps[MAP].objects);
    assert!(build_park_pond_route_pack(&output, CONTRACT).is_err());
}

#[test]
#[ignore = "requires exact external published long-river parent"]
fn park_pond_rejects_stale_parent_art_sign_and_unreviewed_contract() {
    let base = fixture();
    assert!(build_park_pond_route_pack(&base, &(CONTRACT.to_owned() + " ")).is_err());
    let mut changed = base.clone();
    changed.data.maps.get_mut(MAP).expect("park").blocks[0] = 1;
    assert!(build_park_pond_route_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed
        .runtime_files
        .get_mut("data/tilesets/mn_river_park_metatiles.bin")
        .expect("metatiles")[0] ^= 1;
    assert!(build_park_pond_route_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed.data.map_scripts.get_mut(TEXT).expect("raw sign")[0]["args"][0] =
        serde_json::json!("\"WRONG POND\"");
    assert!(build_park_pond_route_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed
        .report
        .manifests
        .push("unreviewed-park-parent".into());
    assert!(build_park_pond_route_pack(&changed, CONTRACT).is_err());
}

#[test]
#[ignore = "requires exact external published long-river parent"]
fn park_pond_residual_rejects_caps_rails_cut_cells_actors_programs_audio_and_mirrors() {
    let base = fixture();
    let output = build_park_pond_route_pack(&base, CONTRACT).expect("park candidate");
    let plan = parse_contract(&base, CONTRACT).expect("contract");
    for case in 0..15 {
        let mut bad = output.clone();
        match case {
            0 => {
                bad.runtime_files
                    .get_mut("data/tilesets/mn_river_park_metatiles.bin")
                    .expect("cap")[68 * 16] ^= 1
            }
            1 => {
                bad.data
                    .tilesets
                    .get_mut(TILESET)
                    .expect("rails")
                    .collision
                    .get_mut("46")
                    .expect("rail collision")[2] = "FLOOR".into()
            }
            2 => {
                bad.data
                    .tilesets
                    .get_mut(TILESET)
                    .expect("elm")
                    .collision
                    .get_mut("00")
                    .expect("elm collision")[0] = "CUT_TREE".into()
            }
            3 => bad.data.maps.get_mut(MAP).expect("park").blocks[0] = 1,
            4 => bad.data.maps.get_mut(MAP).expect("actor").objects[0].x += 1,
            5 => {
                bad.data
                    .maps
                    .get_mut(MAP)
                    .expect("script")
                    .scripts
                    .insert("ExtraParkProgram".into(), serde_json::json!([]));
            }
            6 => {
                bad.data
                    .map_scripts
                    .insert("ExtraParkProgram".into(), serde_json::json!([]));
            }
            7 => {
                bad.runtime_files
                    .get_mut("gfx/tilesets/mn_river_park.2bpp")
                    .expect("bitmap")[0] ^= 1
            }
            8 => {
                bad.data
                    .tilesets
                    .get_mut(TILESET)
                    .expect("palette")
                    .palette_map[0] ^= 1
            }
            9 => {
                let audio = bad
                    .audio_manifest
                    .music
                    .values_mut()
                    .next()
                    .expect("native music");
                audio.pcm_frame_count = Some(audio.pcm_frame_count.expect("native PCM count") + 1);
            }
            10 => bad.report.manifests.push("unrelated-park-change".into()),
            11 => {
                bad.data.map_scripts.get_mut(TEXT).expect("sign")[1]["args"][0] =
                    serde_json::json!("\"Changed second line\"")
            }
            12 => {
                bad.data
                    .maps
                    .get_mut(MAP)
                    .expect("typed sign")
                    .script_text_bodies
                    .get_mut(TEXT)
                    .expect("body")
                    .commands[0]
                    .args[0] = "\"WRONG\"".into()
            }
            13 => {
                bad.data.map_blocks.insert(
                    "MnR04C00R00_Blocks".into(),
                    crate::encode_base64_bytes(&vec![0; 504]),
                );
            }
            14 => {
                bad.data
                    .maps
                    .get_mut("MnMinneapolis")
                    .expect("unrelated city")
                    .blocks[0] ^= 1
            }
            _ => unreachable!("bounded negative case"),
        }
        assert!(
            preserve(&base, &bad, &plan).is_err(),
            "reject out-of-scope mutation {case}"
        );
    }
}
