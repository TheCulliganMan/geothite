//! Native publication contracts; production controller gameplay remains separate.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_brainerd_city_modpack, build_minnesota_gym_review_pack,
    read_verified_compiled_game_pack,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}
fn inputs() -> &'static (String, String, String) {
    static INPUTS: OnceLock<(String, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let town_path = std::env::var_os("CRYSTAL_BRAINERD_CITY_SOURCE").map_or_else(
            || root().join("planning/minnesota/towns/MnBrainerd.json"),
            PathBuf::from,
        );
        let layout_path = std::env::var_os("CRYSTAL_BRAINERD_CITY_LAYOUT").map_or_else(
            || root().join("planning/minnesota/towns/MnBrainerd.layout.json"),
            PathBuf::from,
        );
        let canonical_path = std::env::var_os("CRYSTAL_CITY_GYMS_SOURCE").map_or_else(
            || root().join("planning/minnesota/gyms/interiors.json"),
            PathBuf::from,
        );
        // Consume each exact source string once. A frozen candidate must already
        // match the supplied canonical gym; never normalize or re-sign fixtures.
        let town = std::fs::read_to_string(town_path).expect("exact authored Brainerd source");
        let layout = std::fs::read_to_string(layout_path).expect("exact authored Brainerd strokes");
        let canonical =
            std::fs::read_to_string(canonical_path).expect("exact canonical gym source");
        (town, layout, canonical)
    })
}
fn base() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        if let Some(path) = std::env::var_os("CRYSTAL_BRAINERD_CITY_TEST_PACK") {
            read_verified_compiled_game_pack(PathBuf::from(path))
                .expect("completed current canonical gym pack")
        } else {
            let source =
                read_verified_compiled_game_pack(root().join("content-packs/text-tui.crystalpack"))
                    .expect("completed external native source pack");
            build_minnesota_gym_review_pack(&source, &inputs().2)
                .expect("canonical gym native fixture, not city replacement")
        }
    })
}
fn result() -> &'static CompiledGamePack {
    static OUTPUT: OnceLock<CompiledGamePack> = OnceLock::new();
    OUTPUT.get_or_init(|| {
        build_brainerd_city_modpack(base(), &inputs().0, &inputs().1, &inputs().2)
            .expect("native Brainerd and canonical Gym6 binding")
    })
}

#[test]
fn brainerd_city_preserves_source_and_rebinds_only_the_canonical_gym_warps() {
    let before = base();
    let after = result();
    assert_eq!(after.data().maps.len(), before.data().maps.len() + 5);
    for (name, module) in &before.data().maps {
        let actual = &after.data().maps[name];
        if name == "MnBrainerdGym" {
            let mut normalized = actual.clone();
            for (new, old) in normalized.events.warps.iter_mut().zip(&module.events.warps) {
                assert_eq!(new.target_map_constant, "MN_BRAINERD");
                assert_eq!(new.target_map, "MN_BRAINERD");
                assert_eq!(new.target_warp_id, 6);
                new.target_map_constant = old.target_map_constant.clone();
                new.target_map = old.target_map.clone();
                new.target_warp_id = old.target_warp_id;
            }
            for (new, old) in normalized
                .map_event_section_commands
                .iter_mut()
                .zip(&module.map_event_section_commands)
            {
                if new.command == "warp_event" {
                    new.args = old.args.clone();
                }
            }
            assert_eq!(
                &normalized, module,
                "canonical scripts/puzzles/actors/geometry must stay exact"
            );
        } else {
            assert_eq!(actual, module, "unrelated map {name}");
        }
    }
    assert_eq!(
        after.runtime_files(),
        before.runtime_files(),
        "all original art files retained; no new art copied"
    );
    assert_eq!(after.compiled_audio(), before.compiled_audio());
    assert_eq!(
        after.audio_manifest().expect("after audio"),
        before.audio_manifest().expect("before audio")
    );
    assert_eq!(after.data().audio, before.data().audio);
    assert_eq!(
        after.data().runtime_spawn_points,
        before.data().runtime_spawn_points,
        "HOME and recovery table unchanged"
    );
    assert_eq!(
        after.data().trainers,
        before.data().trainers,
        "reuse exact canonical parties/moves/rewards"
    );
    assert_eq!(after.data().items, before.data().items);
    assert_eq!(after.data().moves, before.data().moves);
    assert_eq!(after.data().pokemon, before.data().pokemon);
    assert_eq!(after.data().field_moves, before.data().field_moves);
    for (key, value) in &before.data().story_event_script_constants.global {
        assert_eq!(
            after.data().story_event_script_constants.global.get(key),
            Some(value),
            "original constant {key}"
        );
    }
    for (constant, meta) in &before.data().runtime_map_metadata {
        assert_eq!(
            after.data().runtime_map_metadata.get(constant),
            Some(meta),
            "original map metadata {constant}"
        );
    }
    for (name, stock) in &before.data().marts.0 {
        assert_eq!(after.data().marts.0.get(name), Some(stock));
    }
    for (index, name) in [
        "MnBrainerd",
        "MnBrainerdPokecenter1F",
        "MnBrainerdMart",
        "MnBrainerdRangerStation",
        "MnBrainerdOutfitter",
    ]
    .iter()
    .enumerate()
    {
        let module = &after.data().maps[*name];
        let constant = module
            .attributes
            .map_constant
            .as_ref()
            .expect("city constant");
        let meta = &after.data().runtime_map_metadata[constant];
        assert_eq!(meta.group_id, 69);
        assert_eq!(usize::from(meta.map_id), index + 1);
        assert!(
            module.attributes.connections.is_empty(),
            "source port reservations must not become fake travel"
        );
    }
    let exterior = &after.data().maps["MnBrainerd"];
    assert_eq!(exterior.events.warps.len(), 10);
    assert_eq!(
        [exterior.events.warps[4].x, exterior.events.warps[4].y],
        [12, 35]
    );
    assert_eq!(
        [exterior.events.warps[5].x, exterior.events.warps[5].y],
        [12, 36]
    );
    assert_eq!(
        exterior.events.warps[4].target_map_constant,
        "MN_BRAINERD_GYM"
    );
    assert_eq!(exterior.events.warps[4].target_warp_id, 3);
}

fn commands<'a>(pack: &'a CompiledGamePack, map: &str, label: &str) -> &'a [Value] {
    pack.data().maps[map].scripts[label]
        .as_array()
        .expect("actual authored script")
        .as_slice()
}
#[test]
fn brainerd_city_lowers_native_services_and_success_only_story_gifts() {
    let pack = result();
    let clinic = &pack.data().maps["MnBrainerdPokecenter1F"];
    assert!(
        !clinic.script_runtime_commands.is_empty(),
        "native special table lowered"
    );
    assert!(
        !clinic.script_object_commands.is_empty(),
        "faceplayer resolves actual object commands"
    );
    assert!(
        commands(pack, &clinic.id, "MN_BRAINERD_NURSE")
            .iter()
            .any(|c| c["command"] == "special" && c["args"] == json!(["HealParty"]))
    );
    assert!(
        commands(pack, &clinic.id, "MnBrainerdClinicPC")
            .iter()
            .any(|c| c["command"] == "special" && c["args"] == json!(["PokemonCenterPC"]))
    );
    assert!(
        !pack.data().maps["MnBrainerdMart"]
            .script_shop_commands
            .is_empty()
    );
    let town: Value = serde_json::from_str(&inputs().0).expect("town source");
    let expected = town["interiors"][1]["inventory_contract"]
        .as_array()
        .expect("native stock")
        .iter()
        .map(|v| v.as_str().expect("item").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(pack.data().marts.0["MART_MN_BRAINERD"], expected);
    let ranger = commands(pack, "MnBrainerd", "MN_BRAINERD_RANGER");
    assert!(
        ranger
            .iter()
            .any(|c| c["command"] == "checkflag" && c["args"] == json!(["ENGINE_MINERALBADGE"]))
    );
    let give = ranger
        .iter()
        .position(|c| c["command"] == "giveitem" && c["args"] == json!(["HM_SURF", "1"]))
        .expect("existing native Surf item");
    assert_eq!(
        ranger[give + 1]["command"],
        "iffalse",
        "full TM pocket must branch before story flags"
    );
    assert!(
        ranger
            .iter()
            .enumerate()
            .filter(|(_, c)| c["command"] == "setevent")
            .all(|(i, _)| i > give + 1)
    );
    assert!(
        !ranger.iter().any(|c| c["command"] == "setflag"),
        "ranger cannot award badges"
    );
    let gift = commands(pack, "MnBrainerdOutfitter", "MN_BRAINERD_GEAR_KEEPER");
    let give = gift
        .iter()
        .position(|c| c["command"] == "giveitem")
        .expect("optional native gift");
    assert_eq!(gift[give + 1]["command"], "iffalse");
    let once = gift
        .iter()
        .position(|c| {
            c["command"] == "setevent" && c["args"] == json!(["EVENT_MN_BRAINERD_CAMP_REPEL"])
        })
        .expect("success-only local flag");
    assert!(once > give + 1);
    assert!(
        pack.data().maps["MnBrainerd"]
            .objects
            .iter()
            .any(
                |a| a.object_identifier.as_deref() == Some("MN_BRAINERD_RANGER")
                    && [a.x, a.y] == [25, 19]
            )
    );
}

fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .map(|(k, v)| (k.clone(), sorted(v)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
        _ => value.clone(),
    }
}
fn resign(town: &mut Value, gyms: &Value) {
    let gym = gyms["gyms"]
        .as_array()
        .expect("gyms")
        .iter()
        .find(|g| g["slot"] == 6)
        .expect("Gym6");
    let signature = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&sorted(gym)).expect("canonical JSON"))
    );
    town["interiors"][2]["canonical_gym_sha256"] = signature.clone().into();
    town["gym_contract"]["canonical_gym_sha256"] = signature.into();
}
#[test]
fn brainerd_city_rejects_runtime_canonical_mismatch_even_after_source_resigning() {
    for variant in ["actor", "party"] {
        let mut town: Value = serde_json::from_str(&inputs().0).expect("town");
        let mut gyms: Value = serde_json::from_str(&inputs().2).expect("gyms");
        let gym = gyms["gyms"]
            .as_array_mut()
            .expect("gyms")
            .iter_mut()
            .find(|g| g["slot"] == 6)
            .expect("Gym6");
        if variant == "actor" {
            let x = gym["leader_at"][0].as_u64().expect("x");
            gym["leader_at"][0] = (x + 1).into();
        } else {
            gym["party"][2]["level"] = 36.into();
        }
        resign(&mut town, &gyms);
        let error =
            build_brainerd_city_modpack(base(), &town.to_string(), &inputs().1, &gyms.to_string())
                .expect_err("a matching source SHA cannot disguise stale native gym contents");
        let message = format!("{error:#}");
        assert!(
            message.contains("installed Gym6 module differs")
                || message.contains("installed canonical Brainerd party/moves differ"),
            "{variant}: {message}"
        );
    }
}

#[test]
fn brainerd_city_rejects_unsafe_source_and_alternative_gym_contracts() {
    let original: Value = serde_json::from_str(&inputs().0).expect("town");
    for variant in [
        "port",
        "door",
        "room-return",
        "service",
        "missing-room",
        "ranger-badge",
        "canonical-sha",
        "alternative-gym",
    ] {
        let mut town = original.clone();
        match variant {
            "port" => town["ports"][0]["gate_metatile"] = json!([15, 22]),
            "door" => town["doors"][0]["tile"] = json!([8, 21]),
            "room-return" => town["interiors"][1]["exits"][0]["target_warp"] = 3.into(),
            "service" => town["interiors"][0]["residents"][0]["service"] = "magic_heal".into(),
            "missing-room" => {
                town["interiors"].as_array_mut().expect("rooms").pop();
            }
            "ranger-badge" => town["residents"][0]["requires"] = json!(["BADGE_2"]),
            "canonical-sha" => town["interiors"][2]["canonical_gym_sha256"] = "stale".into(),
            "alternative-gym" => town["interiors"][2]["rows"] = json!(["............"]),
            _ => unreachable!("enumerated negative"),
        }
        assert!(
            build_brainerd_city_modpack(base(), &town.to_string(), &inputs().1, &inputs().2)
                .is_err(),
            "accepted unsafe {variant}"
        );
    }
    assert!(
        build_brainerd_city_modpack(result(), &inputs().0, &inputs().1, &inputs().2).is_err(),
        "group69 and city identities cannot be overwritten"
    );
}

#[test]
fn brainerd_city_requires_an_actual_preinstalled_canonical_gym() {
    let source =
        read_verified_compiled_game_pack(root().join("content-packs/text-tui.crystalpack"))
            .expect("external core pack");
    let error = build_brainerd_city_modpack(&source, &inputs().0, &inputs().1, &inputs().2)
        .expect_err("source-only gym is not an installed native map");
    assert!(format!("{error:#}").contains("requires the already installed canonical Gym6"));
}
