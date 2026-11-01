//! External preimages, uncached mirrors and whole-pack isolation for long rivers.
use std::path::PathBuf;
use std::sync::OnceLock;

use super::{Contract, build_river_continuity_long_pack};
use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/connections/river-continuity-long-v01/source-v02/candidate.contract.json"
);
const FIRST: &str = "MnMississippiLowerWoodsC00R00";
const GUARDED: &str = "MnR05C00R02";
const PROGRAM: &str = "MnMississippiLowerWoodsC00R01";

fn fixture() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_RIVER_LONG_TEST_PACK")
                .expect("exact completed city-river-bend parent"),
        );
        crate::read_verified_compiled_game_pack(path).expect("verified long-river parent")
    })
}

#[test]
fn river_long_frozen_source_contract_and_all_explicit_layouts_are_exact() {
    assert_eq!(super::hash(CONTRACT.as_bytes()), super::CONTRACT_SHA);
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/connections/river-continuity-long-v01/source-v02");
    let freeze: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(directory.join("freeze.json")).expect("frozen source inventory"),
    )
    .expect("source inventory JSON");
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen source contract");
    for row in freeze["maps"].as_array().expect("15 frozen maps") {
        let source =
            std::fs::read_to_string(directory.join(row["file"].as_str().expect("source file")))
                .expect("frozen explicit layout");
        assert_eq!(
            super::hash(source.as_bytes()),
            row["source_sha256"].as_str().expect("frozen source hash")
        );
        let name = row["map"].as_str().expect("native map name");
        // The route editor's parse whitelist accepts whole route dossiers,
        // not these frozen clipped native chunks. Check their own exact
        // envelope and reuse its deserializer/rasterizer without changing
        // the shared route-level profile policy.
        let document: serde_json::Value = serde_json::from_str(&source).expect("source JSON");
        let parsed: Layout = serde_json::from_str(&source).expect("explicit chunk schema");
        assert_eq!(parsed.id, name);
        assert_eq!(parsed.schema_version, 2);
        assert_eq!(
            parsed.compiled_tileset,
            if name.starts_with("MnR05") {
                "mn_headwaters"
            } else {
                "johto"
            }
        );
        assert_eq!(document["shore_fitting"], "explicit_source_edges");
        assert_eq!(document["reject_cliff_cuts"], true);
        assert!(
            document["layers"]
                .as_array()
                .expect("explicit rows")
                .iter()
                .all(|layer| layer["kind"] == "block_rect")
        );
        let blocks = parsed
            .compile()
            .expect("frozen source compiles without edge fitting");
        let change = contract
            .changes
            .iter()
            .find(|c| c.map == name)
            .expect("frozen change");
        let [width, height] = change.dimensions_metatiles;
        assert_eq!(parsed.dimensions_metatiles, change.dimensions_metatiles);
        assert_eq!(blocks.len(), usize::from(width) * usize::from(height));
        for cell in &change.cells {
            let [x, y] = cell.metatile;
            assert_eq!(
                blocks[usize::from(y) * usize::from(width) + usize::from(x)],
                u16::from_str_radix(&cell.after, 16).expect("native metatile")
            );
        }
    }
}

#[test]
#[ignore = "requires exact external parent in CRYSTAL_RIVER_LONG_TEST_PACK"]
fn river_long_native_1678_cells_14_mirrors_guarded_map_and_uncached_programs_are_exact() {
    let base = fixture();
    let output =
        build_river_continuity_long_pack(base, CONTRACT).expect("reviewed long-river revision");
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen contract");
    super::preserve(base, &output, &contract)
        .expect("strict full-pack residual and uncached programs");
    assert_eq!(output.data.maps.len(), 683);
    let changed_cells = base
        .data
        .maps
        .iter()
        .flat_map(|(name, m)| m.blocks.iter().zip(&output.data.maps[name].blocks))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(changed_cells, 1678);
    assert_eq!(
        base.data
            .map_blocks
            .iter()
            .filter(|(name, payload)| output.data.map_blocks.get(*name) != Some(*payload))
            .count(),
        14
    );
    assert_eq!(base.data.maps[GUARDED], output.data.maps[GUARDED]);
    assert_eq!(base.data.map_scripts, output.data.map_scripts);
    assert_eq!(base.data.map_attributes, output.data.map_attributes);
    assert_ne!(base.identity, output.identity);
    assert!(build_river_continuity_long_pack(&output, CONTRACT).is_err());
}

#[test]
#[ignore = "requires exact external parent in CRYSTAL_RIVER_LONG_TEST_PACK"]
fn river_long_rejects_stale_parent_module_old_programs_and_unreviewed_contract() {
    let base = fixture();
    for case in 0..3 {
        let mut bad = base.clone();
        match case {
            0 => bad.identity.content_hash = "stale-parent".into(),
            1 => bad.data.maps.get_mut(FIRST).expect("river").blocks[0] ^= 1,
            _ => {
                let scripts = &mut bad
                    .data
                    .maps
                    .get_mut(PROGRAM)
                    .expect("trainer river")
                    .scripts;
                let key = scripts.keys().next().expect("old native program").clone();
                scripts.insert(key, serde_json::json!({"changed_old_program": true}));
            }
        }
        assert!(
            build_river_continuity_long_pack(&bad, CONTRACT).is_err(),
            "case {case}"
        );
    }
    let mut extra: serde_json::Value = serde_json::from_str(CONTRACT).expect("source JSON");
    extra["changes"][0]["cells"]
        .as_array_mut()
        .expect("terrain cells")
        .push(serde_json::json!({"metatile":[5,5],"before":"05","after":"35"}));
    assert!(
        build_river_continuity_long_pack(base, &extra.to_string()).is_err(),
        "an unreviewed isolated water cell must not enter the frozen revision"
    );
}

#[test]
#[ignore = "requires exact external parent in CRYSTAL_RIVER_LONG_TEST_PACK"]
fn river_long_residual_rejects_isolated_water_program_mirror_guard_audio_report_and_runtime_drift()
{
    let base = fixture();
    let output = build_river_continuity_long_pack(base, CONTRACT).expect("reviewed rivers");
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen contract");
    for case in 0..8 {
        let mut bad = output.clone();
        match case {
            0 => {
                let map = bad.data.maps.get_mut(FIRST).expect("river");
                let index = 5 * usize::from(map.attributes.width) + 5;
                assert_ne!(
                    map.blocks[index], 0x35,
                    "counterexample must add actual isolated water"
                );
                map.blocks[index] = 0x35;
            }
            1 => {
                let scripts = &mut bad
                    .data
                    .maps
                    .get_mut(PROGRAM)
                    .expect("trainer river")
                    .scripts;
                let key = scripts.keys().next().expect("old native program").clone();
                scripts.insert(key, serde_json::json!({"changed_old_program": true}));
            }
            2 => {
                let key = bad.data.maps[FIRST]
                    .attributes
                    .blocks_label
                    .clone()
                    .expect("block label");
                bad.data
                    .map_blocks
                    .insert(key.clone(), base.data.map_blocks[&key].clone());
            }
            3 => {
                bad.data
                    .maps
                    .get_mut(GUARDED)
                    .expect("guarded unchanged map")
                    .blocks[0] ^= 1
            }
            4 => {
                let audio = bad
                    .audio_manifest
                    .music
                    .values_mut()
                    .next()
                    .expect("native music");
                audio.pcm_frame_count =
                    Some(audio.pcm_frame_count.expect("native frame count") + 1);
            }
            5 => bad.report.manifests.push("unreviewed-river-report".into()),
            6 => {
                bad.runtime_files
                    .insert("unreviewed-river-file".into(), vec![1]);
            }
            _ => {
                let key = bad
                    .data
                    .map_scripts
                    .keys()
                    .next()
                    .expect("native raw programs")
                    .clone();
                bad.data
                    .map_scripts
                    .insert(key, serde_json::json!({"changed_old_raw_program": true}));
            }
        }
        assert!(
            super::preserve(base, &bad, &contract).is_err(),
            "unreviewed case {case}"
        );
    }
}

#[test]
#[ignore = "requires exact external parent in CRYSTAL_RIVER_LONG_TEST_PACK"]
fn river_long_preserves_valid_unrelated_minneapolis_terrain_and_report_in_actual_parent() {
    let original = fixture();
    let mut base = original.clone();
    let city = base
        .data
        .maps
        .get_mut("MnMinneapolis")
        .expect("unrelated Minneapolis");
    let tileset = &base.data.tilesets[&city.attributes.tileset_name];
    let old = city.blocks[0];
    let permission = &tileset.collision[&format!("{old:02x}")];
    let replacement = tileset
        .collision
        .iter()
        .find_map(|(key, value)| {
            let block = u16::from_str_radix(key, 16).ok()?;
            (block != old && block <= 255 && value == permission).then_some(block)
        })
        .expect("another native block with identical collision");
    city.blocks[0] = replacement;
    let label = city
        .attributes
        .blocks_label
        .clone()
        .expect("unrelated city payload");
    base.data.map_blocks.insert(
        label,
        super::encode_blocks(city).expect("valid city block mirror"),
    );
    base.report
        .manifests
        .push("approved-upstream-house-revision-fixture".into());
    base.identity = super::identity(&base).expect("new actual parent identity");
    crate::verify_compiled_game_pack_for_runtime(&base).expect("unrelated valid upstream revision");
    let output =
        build_river_continuity_long_pack(&base, CONTRACT).expect("compose onto approved parent");
    assert_eq!(
        output.data.maps["MnMinneapolis"],
        base.data.maps["MnMinneapolis"]
    );
    assert_eq!(output.report, base.report);
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen source baseline");
    super::preserve(&base, &output, &contract).expect("actual parent residual is exact");
    assert_ne!(output.identity, base.identity);
}
