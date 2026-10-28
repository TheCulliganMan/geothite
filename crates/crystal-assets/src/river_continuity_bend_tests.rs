//! External native preimages and whole-pack residual checks for the city bend.
use std::path::PathBuf;
use std::sync::OnceLock;

use super::{Contract, EVENTS, ROAD, SIGN, build_river_continuity_bend_pack, preserve};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/connections/river-continuity-v01/source-v01/candidate.contract.json"
);

#[test]
fn river_bend_explicit_authoring_retains_all_768_reviewed_cells_and_old_lower_shores() {
    use crate::authored_route_layout::Layout;

    let before = Layout::parse(include_str!(
        "../../../planning/minnesota/connections/river-continuity-v01/source-v02/MN_R03.before.layout.json"
    ))
    .expect("original fitted river source")
    .compile()
    .expect("original river terrain");
    let candidate_text = include_str!(
        "../../../planning/minnesota/connections/river-continuity-v01/source-v02/MN_R03.candidate.layout.json"
    );
    let candidate = Layout::parse(candidate_text)
        .expect("explicit hand-authored R03 edges")
        .compile()
        .expect("hand-authored river terrain");
    let bytes = candidate
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<Result<Vec<_>, _>>()
        .expect("native one-byte metatiles");
    assert_eq!(candidate.len(), 768);
    assert_eq!(
        super::hash(&bytes),
        "b5ee7cd18c141d74f5c714c4650025c6a0ab2a0aa1b75a4ae3f901a9ce44fe87"
    );
    assert_eq!(candidate[24 * 24..], before[24 * 24..]);
    assert_eq!(
        candidate
            .iter()
            .zip(&before)
            .filter(|(a, b)| a != b)
            .count(),
        55
    );
    let refitted =
        Layout::parse(&candidate_text.replace("explicit_source_edges", "johto_8_neighbors"))
            .expect("legacy fitting remains supported")
            .compile()
            .expect("legacy fitter");
    assert_ne!(
        refitted, candidate,
        "automatic fitting must not silently replace authored edges"
    );
}

fn fixture() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_RIVER_BEND_TEST_PACK")
                .expect("exact completed winter parent"),
        );
        crate::read_verified_compiled_game_pack(path).expect("verified city bend parent")
    })
}

#[test]
#[ignore = "requires exact external winter parent in CRYSTAL_RIVER_BEND_TEST_PACK"]
fn river_bend_native_51_cells_sign_raw_typed_and_uncached_mirrors_are_exact() {
    let base = fixture();
    let output =
        build_river_continuity_bend_pack(base, CONTRACT).expect("reviewed native city bend");
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen contract");
    preserve(base, &output, &contract).expect("whole pack residual and uncached event parsing");
    assert_eq!(output.data.maps.len(), 683);
    let changed = base
        .data
        .maps
        .iter()
        .flat_map(|(name, m)| m.blocks.iter().zip(&output.data.maps[name].blocks))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(changed, 51);
    let sign = output.data.maps[ROAD]
        .events
        .bg_events
        .iter()
        .find(|e| e.script == SIGN)
        .expect("same north sign");
    assert_eq!([sign.x, sign.y], [22, 8]);
    assert_eq!(output.data.map_scripts[EVENTS][6]["args"][1], "8");
    assert_eq!(
        output.data.maps[ROAD].map_event_section_commands[6].args[1],
        "8"
    );
    assert_eq!(output.data.maps[ROAD].scripts, base.data.maps[ROAD].scripts);
    assert_ne!(output.identity.content_hash, base.identity.content_hash);
    assert!(build_river_continuity_bend_pack(&output, CONTRACT).is_err());
}

#[test]
#[ignore = "requires exact external winter parent in CRYSTAL_RIVER_BEND_TEST_PACK"]
fn river_bend_rejects_stale_native_terrain_sign_and_contract() {
    let base = fixture();
    let mut bad = base.clone();
    bad.data.maps.get_mut(ROAD).expect("road").blocks[15] ^= 1;
    assert!(build_river_continuity_bend_pack(&bad, CONTRACT).is_err());
    let mut bad = base.clone();
    bad.data.map_scripts.get_mut(EVENTS).expect("events")[6]["args"][1] = serde_json::json!("7");
    assert!(build_river_continuity_bend_pack(&bad, CONTRACT).is_err());
    assert!(build_river_continuity_bend_pack(base, &format!(" {CONTRACT}")).is_err());
}

#[test]
#[ignore = "requires exact external winter parent in CRYSTAL_RIVER_BEND_TEST_PACK"]
fn river_bend_residual_rejects_unrelated_roads_programs_audio_report_and_runtime_files() {
    let base = fixture();
    let output = build_river_continuity_bend_pack(base, CONTRACT).expect("reviewed bend");
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen contract");
    let mut bad = output.clone();
    bad.data.maps.get_mut(ROAD).expect("road").blocks[100] ^= 1;
    assert!(preserve(base, &bad, &contract).is_err());
    let mut bad = output.clone();
    bad.data
        .maps
        .get_mut(ROAD)
        .expect("road")
        .scripts
        .insert("UnreviewedRiverProgram".into(), serde_json::json!([]));
    assert!(preserve(base, &bad, &contract).is_err());
    let mut bad = output.clone();
    let audio = bad
        .audio_manifest
        .music
        .values_mut()
        .next()
        .expect("native audio descriptor");
    audio.pcm_frame_count = Some(audio.pcm_frame_count.expect("native PCM frame count") + 1);
    assert!(preserve(base, &bad, &contract).is_err());
    let mut bad = output.clone();
    bad.report
        .manifests
        .push("unreviewed-river-manifest".into());
    assert!(preserve(base, &bad, &contract).is_err());
    let mut bad = output.clone();
    bad.runtime_files
        .insert("unreviewed-river-file".into(), vec![1]);
    assert!(preserve(base, &bad, &contract).is_err());
}
