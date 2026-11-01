//! Frozen source, native representation isolation and composable actual parents.
use std::path::PathBuf;
use std::sync::OnceLock;

use super::{Contract, build_north_shore_warm_pack};
use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/r11-warm/source-v01/native.contract.json"
);
const FIRST: &str = "MnR11C00R00";
const GUARDED: &str = "MnR11C01R03";

fn fixture() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_NORTH_SHORE_WARM_TEST_PACK")
                .expect("completed approved parent retaining frozen R11 preimages"),
        ))
        .expect("verified warm-coast actual parent")
    })
}

fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../planning/minnesota/revisions/north-shore-ely-geography-v01/r11-warm/source-v01",
    )
}

#[test]
fn north_shore_warm_frozen_source_hashes_and_828_cell_contract_are_exact() {
    assert_eq!(super::hash(CONTRACT.as_bytes()), super::CONTRACT_SHA);
    let value: serde_json::Value = serde_json::from_str(CONTRACT).expect("frozen contract");
    for (name, expected) in value["candidates_sha256"]
        .as_object()
        .expect("immutable candidate sources")
    {
        assert_eq!(
            super::hash(&std::fs::read(source().join(name)).expect("frozen candidate")),
            expected.as_str().expect("candidate hash")
        );
    }
    // Canonical preimage paths are historical guards, and can change after a
    // separately approved source promotion. Only the immutable source epochs
    // are required to retain their original bytes permanently.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, expected) in value["source_files_sha256"]
        .as_object()
        .expect("source baseline hashes")
    {
        if name.contains("/regions/ice/") {
            assert_eq!(
                super::hash(&std::fs::read(root.join(name)).expect("immutable winter source")),
                expected.as_str().expect("source hash")
            );
        }
    }
    let text = std::fs::read_to_string(source().join("MN_R11.warm.candidate.layout.json"))
        .expect("warm layout");
    let layout: Layout = serde_json::from_str(&text).expect("hash-bound exact whole-route schema");
    assert_eq!(layout.dimensions_metatiles, [56, 96]);
    assert_eq!(layout.compiled_tileset, super::WARM);
    let blocks = layout.compile().expect("explicit warm source compiles");
    assert_eq!(blocks.len(), 56 * 96);
    assert!(blocks.iter().all(|b| *b < 0x83));
    let plan: Contract = serde_json::from_str(CONTRACT).expect("frozen restoration");
    assert_eq!(
        plan.modules
            .iter()
            .map(|m| m.cells.len())
            .collect::<Vec<_>>(),
        [499, 329, 0, 0, 0, 0, 0, 0]
    );
    for row in &plan.modules {
        let (_, chunk) = row.map.split_once('C').expect("native chunk");
        let (column, r) = chunk.split_once('R').expect("native chunk origin");
        let column: usize = column.parse().expect("column");
        let r: usize = r.parse().expect("row");
        for cell in &row.cells {
            let [x, y] = cell.metatile;
            assert_eq!(
                blocks[(r * 24 + usize::from(y)) * 56 + column * 32 + usize::from(x)],
                cell.after
            );
        }
    }
}

#[test]
#[ignore = "requires approved native parent in CRYSTAL_NORTH_SHORE_WARM_TEST_PACK"]
fn north_shore_warm_restores_all8_modules_828_cells_2_mirrors_and_uncached_programs() {
    let base = fixture();
    let output = build_north_shore_warm_pack(base, CONTRACT).expect("approved warm restoration");
    let plan: Contract = serde_json::from_str(CONTRACT).expect("frozen restoration");
    super::preserve(base, &output, &plan)
        .expect("all native representations and whole-parent residual");
    assert_eq!(
        base.data
            .maps
            .iter()
            .flat_map(|(name, m)| m.blocks.iter().zip(&output.data.maps[name].blocks))
            .filter(|(a, b)| a != b)
            .count(),
        828
    );
    assert_eq!(
        base.data
            .map_blocks
            .iter()
            .filter(|(n, p)| output.data.map_blocks.get(*n) != Some(*p))
            .count(),
        2
    );
    assert_eq!(
        base.data
            .map_attributes
            .iter()
            .filter(|(n, a)| output.data.map_attributes.get(*n) != Some(*a))
            .count(),
        2
    );
    assert!(
        output
            .data
            .maps
            .iter()
            .filter(|(n, _)| n.starts_with("MnR11C"))
            .all(|(_, m)| m.attributes.tileset_name == super::WARM
                && m.blocks.iter().all(|b| *b < 0x83))
    );
    let layout: Layout = serde_json::from_slice(
        &std::fs::read(source().join("MN_R11.warm.candidate.layout.json")).expect("frozen layout"),
    )
    .expect("whole-route layout");
    let blocks = layout.compile().expect("whole warm native grid");
    for row in &plan.modules {
        let (_, chunk) = row.map.split_once('C').expect("native chunk");
        let (column, r) = chunk.split_once('R').expect("chunk origin");
        let column: usize = column.parse().expect("column");
        let r: usize = r.parse().expect("row");
        let map = &output.data.maps[&row.map];
        let expected = (0..usize::from(map.attributes.height))
            .flat_map(|y| {
                let blocks = &blocks;
                (0..usize::from(map.attributes.width))
                    .map(move |x| blocks[(r * 24 + y) * 56 + column * 32 + x])
            })
            .collect::<Vec<_>>();
        assert_eq!(
            map.blocks, expected,
            "{} full native warm geometry",
            row.map
        );
    }
    assert_eq!(base.data.maps[GUARDED], output.data.maps[GUARDED]);
    assert_eq!(base.data.map_scripts, output.data.map_scripts);
    assert_eq!(base.runtime_files, output.runtime_files);
    assert_ne!(base.identity, output.identity);
    assert!(build_north_shore_warm_pack(&output, CONTRACT).is_err());
}

fn valid_map_delta(base: &mut CompiledGamePack, name: &str) {
    let map = base.data.maps.get_mut(name).expect("existing map");
    let collision = &base.data.tilesets[&map.attributes.tileset_name].collision;
    let old = map.blocks[0];
    let permissions = &collision[&format!("{old:02x}")];
    let replacement = collision
        .iter()
        .find_map(|(key, value)| {
            let block = u16::from_str_radix(key, 16).ok()?;
            (block != old && block <= 255 && value == permissions).then_some(block)
        })
        .expect("another valid native recipe with identical collision");
    map.blocks[0] = replacement;
    let label = map.attributes.blocks_label.clone().expect("block label");
    base.data.map_blocks.insert(
        label,
        super::encode_blocks(map).expect("valid block mirror"),
    );
    base.identity = super::identity(base).expect("updated actual parent identity");
    crate::verify_compiled_game_pack_for_runtime(base).expect("valid native parent delta");
}

#[test]
#[ignore = "requires approved native parent in CRYSTAL_NORTH_SHORE_WARM_TEST_PACK"]
fn north_shore_warm_rejects_stale_owned_maps_identity_contract_and_art() {
    let base = fixture();
    let mut stale = base.clone();
    stale.identity.content_hash = "stale-parent".into();
    assert!(build_north_shore_warm_pack(&stale, CONTRACT).is_err());
    for name in [FIRST, GUARDED] {
        let mut stale = base.clone();
        valid_map_delta(&mut stale, name);
        assert!(
            build_north_shore_warm_pack(&stale, CONTRACT).is_err(),
            "stale owned {name}"
        );
    }
    let mut art = base.clone();
    let path = "gfx/tilesets/mn_north_shore.2bpp";
    art.runtime_files.get_mut(path).expect("warm bitplanes")[0] ^= 1;
    assert!(super::art_preimages(&art).is_err());
    let mut extra: serde_json::Value = serde_json::from_str(CONTRACT).expect("contract");
    extra["modules"][0]["cells"]
        .as_array_mut()
        .expect("cells")
        .push(
            serde_json::json!({"metatile":[10,10],"before":131,"after":1,
            "before_quadrants":["FLOOR","FLOOR","FLOOR","FLOOR"],
            "after_quadrants":["FLOOR","FLOOR","FLOOR","FLOOR"]}),
        );
    assert!(build_north_shore_warm_pack(base, &extra.to_string()).is_err());
}

#[test]
#[ignore = "requires approved native parent in CRYSTAL_NORTH_SHORE_WARM_TEST_PACK"]
fn north_shore_warm_residual_rejects_extra_cells_programs_mirrors_art_flags_and_report() {
    let base = fixture();
    let output = build_north_shore_warm_pack(base, CONTRACT).expect("warm shore");
    let plan: Contract = serde_json::from_str(CONTRACT).expect("contract");
    for case in 0..10 {
        let mut bad = output.clone();
        match case {
            0 | 1 => {
                let name = if case == 0 { FIRST } else { GUARDED };
                let map = bad.data.maps.get_mut(name).expect("shore map");
                map.blocks[0] ^= 1;
                let label = map.attributes.blocks_label.clone().expect("payload");
                bad.data.map_blocks.insert(
                    label,
                    super::encode_blocks(map).expect("synchronized counterexample mirror"),
                );
            }
            2 => {
                let scripts = &mut bad.data.maps.get_mut(FIRST).expect("shore").scripts;
                let key = scripts.keys().next().expect("native program").clone();
                scripts.insert(key, serde_json::json!({"changed_old_program":true}));
            }
            3 => {
                let label = bad.data.maps[FIRST]
                    .attributes
                    .map_scripts_label
                    .clone()
                    .expect("raw program label");
                bad.data.map_scripts.insert(label, serde_json::json!([]));
            }
            4 => {
                bad.data
                    .map_attributes
                    .get_mut(FIRST)
                    .expect("attributes")
                    .tileset_name = super::COLD.into();
            }
            5 => {
                bad.runtime_files
                    .get_mut(super::ART[0].0)
                    .expect("warm art mirror")[0] ^= 1;
            }
            6 => {
                let audio = bad
                    .audio_manifest
                    .music
                    .values_mut()
                    .next()
                    .expect("existing music");
                audio.pcm_frame_count =
                    Some(audio.pcm_frame_count.expect("native frame count") + 1);
            }
            7 => {
                bad.data
                    .story_event_script_constants
                    .global
                    .insert("COUNTEREXAMPLE_WARM_FLAG".into(), 999);
            }
            8 => {
                bad.report.manifests.push("unapproved-warm-report".into());
            }
            _ => {
                let label = &plan.modules[0].blocks_label;
                bad.data
                    .map_blocks
                    .insert(label.clone(), base.data.map_blocks[label].clone());
            }
        }
        bad.identity = super::identity(&bad).expect("counterexample has current derived identity");
        assert!(
            super::preserve(base, &bad, &plan).is_err(),
            "residual case{case}"
        );
    }
}

#[test]
#[ignore = "requires approved native parent in CRYSTAL_NORTH_SHORE_WARM_TEST_PACK"]
fn north_shore_warm_composes_with_valid_unrelated_park_parent_change() {
    let mut base = fixture().clone();
    valid_map_delta(&mut base, "MnR04C00R00");
    base.report
        .manifests
        .push("approved-upstream-park-parent-fixture".into());
    base.identity = super::identity(&base).expect("actual approved parent identity");
    crate::verify_compiled_game_pack_for_runtime(&base).expect("valid actual unrelated parent");
    let output = build_north_shore_warm_pack(&base, CONTRACT).expect("compose onto actual parent");
    assert_eq!(
        base.data.maps["MnR04C00R00"],
        output.data.maps["MnR04C00R00"]
    );
    assert_eq!(base.report, output.report);
    let plan: Contract = serde_json::from_str(CONTRACT).expect("frozen contract");
    super::preserve(&base, &output, &plan).expect("all actual-parent unrelated content retained");
}
