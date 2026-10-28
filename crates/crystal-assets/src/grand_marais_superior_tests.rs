use std::path::PathBuf;

use serde_json::json;

use super::{
    MAP, SIGN, TILESET, WRAPPER, build_grand_marais_superior_pack, identity, kind, parse_contract,
    preserve,
};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/grand-marais/source-v02/native-edit.contract.json"
);

#[test]
fn grand_marais_superior_preserves_only_inherited_transport_header_omission() {
    let mut cached = json!({"scripts": {}, "events": {"count": 1}});
    let before = json!({"scripts": {(super::EVENTS): [1]}, "events": {"count": 1}});
    let after = json!({"scripts": {(super::EVENTS): [1, 2]}, "events": {"count": 2}});
    super::merge_wire_delta(&mut cached, &before, &after).expect("typed events retain wire delta");
    assert_eq!(cached, json!({"scripts": {}, "events": {"count": 2}}));
    let before = json!({"scripts": {"ordinary_script": [1]}});
    let after = json!({"scripts": {"ordinary_script": [2]}});
    assert!(super::merge_wire_delta(&mut cached, &before, &after).is_err());
}

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_MARAIS_SUPERIOR_TEST_PACK")
            .expect("explicit completed art-parent path"),
    );
    crate::read_verified_compiled_game_pack(path).expect("verified native art parent")
}

#[test]
#[ignore = "requires external native art parent"]
fn grand_marais_superior_joins_west_and_north_with_safe_native_gate_indexes() {
    let base = fixture();
    let result = build_grand_marais_superior_pack(&base, CONTRACT).expect("bounded harbor stage");
    let plan = parse_contract(&base, CONTRACT).expect("frozen owned preimages");
    preserve(&base, &result, &plan).expect("whole-pack residual");
    let old = &base.data.maps[MAP];
    let town = &result.data.maps[MAP];
    assert_eq!(
        old.blocks
            .iter()
            .zip(&town.blocks)
            .filter(|(a, b)| a != b)
            .count(),
        28
    );
    assert_eq!(old.events.warps, town.events.warps);
    assert_eq!(old.events.bg_events, town.events.bg_events);
    assert_eq!(town.events.warps.len(), 6);
    assert_eq!(town.events.coord_events.len(), 50);
    assert_eq!(
        town.attributes.connection_flags.as_deref(),
        Some("NORTH | WEST")
    );
    assert_eq!(
        town.attributes.connections[0].target_map,
        "MnMaraisCedarApproach"
    );
    assert_eq!(town.attributes.connections[0].offset, -19);
    assert_eq!(town.attributes.connections[1].target_map, "MnR11C01R00");
    for y in 16..37 {
        for x in 1..4 {
            assert_eq!(kind(&result, x, y), "WATER");
        }
        for (x, script) in [(2, "MnCohesionMaraisWest"), (1, WRAPPER)] {
            assert!(
                town.events
                    .coord_events
                    .iter()
                    .any(|e| usize::from(e.x) == x
                        && usize::from(e.y) == y
                        && e.script_name == script)
            );
        }
    }
    for (id, x) in [
        ("MNCOHESIONMARAISEAST_A", 18),
        ("MNCOHESIONMARAISEAST_B", 19),
    ] {
        let guard = town
            .objects
            .iter()
            .find(|o| o.object_identifier.as_deref() == Some(id))
            .expect("original officer identity");
        assert_eq!((guard.x, guard.y), (x, 2));
        assert_eq!(guard.spritemovedata, "SPRITEMOVEDATA_STANDING_DOWN");
    }
    assert!(
        town.script_variable_commands
            .iter()
            .any(|c| c.source_script == WRAPPER && c.command == "readvar")
    );
    assert!(
        town.script_control_commands
            .iter()
            .any(|c| c.source_script == WRAPPER
                && c.compare_value.as_deref() == Some("RIGHT")
                && c.resolved_target_script.as_deref() == Some("MnCohesionMaraisWestInward"))
    );
    assert!(
        town.script_control_commands
            .iter()
            .any(|c| c.source_script == "MnCohesionMaraisEastInward"
                && c.compare_value.as_deref() == Some("UP"))
    );
    assert_eq!(
        town.scripts["MnCohesionMaraisWest"],
        old.scripts["MnCohesionMaraisWest"]
    );
    assert_eq!(
        town.scripts["MnCohesionMaraisEast"],
        old.scripts["MnCohesionMaraisEast"]
    );
    assert_eq!(
        result.data.map_scripts[SIGN][1]["args"][0],
        json!("\"NORTH SHORE: WEST\"")
    );
    assert_eq!(
        result.data.field_moves.cut.replacements[TILESET],
        base.data.field_moves.cut.replacements["johto"]
    );
    assert_eq!(
        result.data.field_moves.whirlpool.replacements.get(TILESET),
        base.data.field_moves.whirlpool.replacements.get("johto")
    );
    let old_art = &base.runtime_files["data/tilesets/johto_metatiles.bin"];
    assert_eq!(
        &result.runtime_files["data/tilesets/mn_marais_shore_metatiles.bin"][..old_art.len()],
        old_art
    );
    assert_eq!(
        result.identity,
        identity(&result).expect("derived stage identity")
    );
    assert!(build_grand_marais_superior_pack(&result, CONTRACT).is_err());
}

#[test]
#[ignore = "requires external native art parent"]
fn grand_marais_superior_rejects_stale_owned_inputs_and_preserves_upstream_additions() {
    let base = fixture();
    assert!(build_grand_marais_superior_pack(&base, &(CONTRACT.to_owned() + " ")).is_err());
    let mut changed = base.clone();
    changed.data.maps.get_mut(MAP).expect("town").blocks[0] = 1;
    changed.identity = identity(&changed).expect("changed test identity");
    assert!(build_grand_marais_superior_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed
        .data
        .map_scripts
        .get_mut("MnCohesionMaraisEastOneStep")
        .expect("raw move")[0]["args"][0] = json!("UP");
    changed.identity = identity(&changed).expect("changed test identity");
    assert!(build_grand_marais_superior_pack(&changed, CONTRACT).is_err());
    let mut upstream = base.clone();
    upstream
        .report
        .manifests
        .push("minnesota-superior-composition-fixture-v1".into());
    upstream.identity = identity(&upstream).expect("upstream identity");
    let result = build_grand_marais_superior_pack(&upstream, CONTRACT)
        .expect("unrelated upstream manifest retained");
    assert!(
        result
            .report
            .manifests
            .contains(&"minnesota-superior-composition-fixture-v1".into())
    );
}

#[test]
#[ignore = "requires external native art parent"]
fn grand_marais_superior_full_inverse_rejects_unowned_mutations() {
    let base = fixture();
    let plan = parse_contract(&base, CONTRACT).expect("plan");
    let result = build_grand_marais_superior_pack(&base, CONTRACT).expect("candidate");
    type Corrupt = fn(&mut CompiledGamePack);
    let cases: [(&str, Corrupt); 12] = [
        ("town cell", |p| {
            p.data.maps.get_mut(MAP).expect("town").blocks[5] = 1
        }),
        ("actor", |p| {
            p.data.maps.get_mut(MAP).expect("town").objects[0].x += 1
        }),
        ("door", |p| {
            p.data.maps.get_mut(MAP).expect("town").events.warps[0].x += 1
        }),
        ("water event", |p| {
            p.data
                .maps
                .get_mut(MAP)
                .expect("town")
                .events
                .coord_events
                .pop();
        }),
        ("gate program", |p| {
            p.data
                .map_scripts
                .get_mut("MnCohesionMaraisWest")
                .expect("gate")[0]["args"][0] = json!("ENGINE_HIVEBADGE")
        }),
        ("wrapper", |p| {
            p.data.map_scripts.get_mut(WRAPPER).expect("wrapper")[1]["args"][0] = json!("LEFT")
        }),
        ("room", |p| {
            p.data
                .maps
                .get_mut("MnGrandMaraisPokecenter")
                .expect("nurse")
                .blocks[0] ^= 1
        }),
        ("old glyph", |p| {
            p.runtime_files
                .get_mut("gfx/tilesets/johto.2bpp")
                .expect("glyphs")[0] ^= 1
        }),
        ("new reef", |p| {
            p.runtime_files
                .get_mut("data/tilesets/mn_marais_shore_metatiles.bin")
                .expect("reef")[128 * 16] ^= 1
        }),
        ("new collision", |p| {
            p.data
                .tilesets
                .get_mut(TILESET)
                .expect("alias")
                .collision
                .get_mut("80")
                .expect("reef")[1] = "WATER".into()
        }),
        ("report", |p| {
            p.report.manifests.push("unreviewed-output-mutation".into())
        }),
        ("audio", |p| {
            p.data.audio.pop();
        }),
    ];
    for (name, corrupt) in cases {
        let mut changed = result.clone();
        corrupt(&mut changed);
        assert!(
            preserve(&base, &changed, &plan).is_err(),
            "unowned mutation accepted: {name}"
        );
    }
}
