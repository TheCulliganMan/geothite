//! Native Cut gates seal actual side areas while preserving earlier content.
use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    CompiledGamePack, build_leech_lake_modpack, build_northern_lakes_modpack,
    build_pine_country_modpack, build_river_bluffs_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

type Publisher = fn(&CompiledGamePack, &str, &str) -> anyhow::Result<CompiledGamePack>;

#[test]
fn cut_gated_routes_compose_preserve_sources_and_reject_free_or_fake_gates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let read = |id: u8, layout: bool| {
        fs::read_to_string(root.join(format!(
            "planning/minnesota/routes/MN_R{id:02}{}.json",
            if layout { ".layout" } else { "" }
        )))
        .expect("authoring")
    };
    let source = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external source");
    let mut base = build_river_bluffs_modpack(&source, &read(5, true), &read(5, false))
        .expect("previous route");
    for (id, additional_maps, build) in [
        (6, 3, build_pine_country_modpack as Publisher),
        (7, 2, build_leech_lake_modpack as Publisher),
        (8, 4, build_northern_lakes_modpack as Publisher),
    ] {
        let layout = read(id, true);
        let dossier = read(id, false);
        let result = build(&base, &layout, &dossier).expect("native gated route");
        assert_eq!(
            result.data().maps.len(),
            base.data().maps.len() + additional_maps
        );
        assert_eq!(result.compiled_audio(), base.compiled_audio());
        assert_eq!(
            result.audio_manifest().expect("audio"),
            base.audio_manifest().expect("audio")
        );
        assert_eq!(
            result.data().runtime_spawn_points,
            base.data().runtime_spawn_points
        );
        for (name, map) in &base.data().maps {
            assert_eq!(result.data().maps.get(name), Some(map), "changed {name}");
        }
        for (name, bytes) in base.runtime_files() {
            assert_eq!(
                result.runtime_files().get(name),
                Some(bytes),
                "changed {name}"
            );
        }
        for (name, tileset) in &base.data().tilesets {
            assert_eq!(
                result.data().tilesets.get(name),
                Some(tileset),
                "changed {name}"
            );
        }
        assert_eq!(
            result.data().field_moves.cut.replacements["mn_headwaters"],
            result.data().field_moves.cut.replacements["johto"]
        );
        let design: Value = serde_json::from_str(&dossier).expect("dossier");
        let gate = &design["cut_gates"][0];
        let x = gate["tile"][0].as_u64().expect("x");
        let y = gate["tile"][1].as_u64().expect("y");
        let native = &result.data().maps[&format!("MnR{id:02}C{:02}R{:02}", x / 64, y / 48)];
        let block = native.blocks
            [(y % 48 / 2) as usize * usize::from(native.attributes.width) + (x % 64 / 2) as usize];
        let rule = &result.data().field_moves.cut.replacements["mn_headwaters"][&block];
        assert_eq!(
            rule.replacement_block_id,
            u16::from_str_radix(
                gate["replacement_block"]
                    .as_str()
                    .expect("native replacement"),
                16
            )
            .expect("hex")
        );
        assert_eq!(rule.variant, "tree");
        for mutation in 0..4 {
            let mut bad = design.clone();
            match mutation {
                0 => bad["cut_gates"][0]["replacement_block"] = json!("01"),
                1 => bad["cut_gates"][0]["targets"] = json!([gate["approach"].clone()]),
                2 => bad["cut_gates"][0]["gated_actor_ids"] = json!([]),
                _ => bad["cut_gates"][0]["gated_actor_ids"] = json!(["UNAUTHORED_ACTOR"]),
            }
            assert!(
                build(&base, &layout, &bad.to_string()).is_err(),
                "R{id:02} accepted fake gate mutation {mutation}"
            );
        }
        let mut bad: Value = serde_json::from_str(&layout).expect("layout");
        bad["layers"].as_array_mut().expect("layers").push(json!({"kind":"point","name":"Decorative Cut tree on untouched background","position":[0,0],"block":"5b"}));
        assert!(
            build(&base, &bad.to_string(), &dossier).is_err(),
            "R{id:02} accepted undeclared Cut quadrant"
        );
        base = result;
    }
    assert_eq!(base.data().maps.len(), 401);
}
