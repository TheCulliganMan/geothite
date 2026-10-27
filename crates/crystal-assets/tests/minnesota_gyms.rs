//! Native gym publication must preserve the supplied game and reject bad rooms.
use std::path::PathBuf;

use crystal_assets::{
    build_minnesota_gym_review_pack, build_minnesota_gym_review_room,
    read_verified_compiled_game_pack,
};
use serde_json::Value;

#[test]
fn minnesota_gyms_publish_all_rooms_native_scripts_and_preserve_source() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
        .expect("external native source pack");
    let design = std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
        .expect("authored gym design");
    let source: Value = serde_json::from_str(&design).expect("gym design JSON");
    let story: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("planning/minnesota/story.json"))
            .expect("authoritative story"),
    )
    .expect("story JSON");
    for (gym, story_gym) in source["gyms"]
        .as_array()
        .expect("gym designs")
        .iter()
        .zip(story["gyms"].as_array().expect("story gyms"))
    {
        assert_eq!(gym["slot"], story_gym["slot"]);
        assert_eq!(gym["city"], story_gym["town"]);
        assert_eq!(gym["leader"], story_gym["leader"]);
        for name in std::iter::once(gym["leader"].as_str().expect("leader name")).chain(
            gym["trainers"]
                .as_array()
                .expect("practice trainers")
                .iter()
                .map(|trainer| trainer["name"].as_str().expect("practice trainer name")),
        ) {
            assert!(
                !base
                    .data()
                    .trainers
                    .trainers
                    .values()
                    .any(|t| t.name == name),
                "authored character {name} reuses a source trainer name"
            );
        }
        assert_eq!(
            gym["party"].as_array().expect("gym party").len(),
            story_gym["party"].as_array().expect("story party").len()
        );
        for (member, planned) in gym["party"]
            .as_array()
            .expect("design party")
            .iter()
            .zip(story_gym["party"].as_array().expect("story party"))
        {
            assert_eq!(member["species"], planned["species"]);
            assert_eq!(member["level"], planned["level"]);
        }
    }
    let result =
        build_minnesota_gym_review_pack(&base, &design).expect("eight native gym interiors");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 9);
    for (name, module) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(module),
            "changed source map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("published audio"),
        base.audio_manifest().expect("source audio")
    );
    assert_eq!(result.data().audio, base.data().audio);
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().items, base.data().items);
    let mut original_field_moves = result.data().field_moves.clone();
    assert!(
        original_field_moves
            .cut
            .replacements
            .remove("mn_gym_7")
            .is_some()
    );
    assert_eq!(original_field_moves, base.data().field_moves);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    for (id, trainer) in &base.data().trainers.trainers {
        assert_eq!(result.data().trainers.trainers.get(id), Some(trainer));
    }
    for (index, gym) in source["gyms"]
        .as_array()
        .expect("eight designs")
        .iter()
        .enumerate()
    {
        let name = gym["id"].as_str().expect("gym name");
        let room = &result.data().maps[name];
        assert_eq!(
            u64::from(room.attributes.width),
            gym["dimensions_metatiles"][0].as_u64().expect("width")
        );
        assert_eq!(
            u64::from(room.attributes.height),
            gym["dimensions_metatiles"][1].as_u64().expect("height")
        );
        let lifts = gym
            .get("lifts")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        assert_eq!(room.events.warps.len(), 3 + lifts * 2);
        assert!(
            room.objects
                .iter()
                .all(|actor| !matches!(actor.sprite.as_str(), "SPRITE_PAPER" | "SPRITE_POKEDEX")),
            "floor books are not gym controls"
        );
        let pad_tiles = gym
            .get("pads")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|pad| pad["tiles_core"].as_array().expect("pad tiles").len())
            .sum::<usize>();
        let cut_trees = gym
            .get("cut_gates")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        assert_eq!(room.events.coord_events.len(), pad_tiles + cut_trees);
        assert!(
            room.events
                .coord_events
                .iter()
                .all(|event| event.scene_id.is_empty()),
            "floor pads use the runtime's scene-independent coordinate events"
        );
        assert_eq!(
            u64::from(room.events.warps[2].x),
            gym["arrival_core"][0].as_u64().expect("arrival x")
        );
        assert_eq!(
            u64::from(room.events.warps[2].y),
            gym["arrival_core"][1].as_u64().expect("arrival y")
        );
        assert!(!room.scripted_trainer_battles.is_empty());
        assert_eq!(
            room.script_block_changes.is_empty(),
            gym["gates"].as_array().expect("gates").is_empty() && cut_trees == 0
        );
        for gate in gym
            .get("cut_gates")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            assert!(
                room.script_block_changes
                    .iter()
                    .any(|change| u64::from(change.x)
                        == gate["tree_core"][0].as_u64().expect("tree x") / 2 * 2
                        && u64::from(change.y)
                            == gate["tree_core"][1].as_u64().expect("tree y") / 2 * 2),
                "saved pruning uses the real native changeblock coordinates"
            );
        }
        for gate in gym["gates"].as_array().expect("gates") {
            assert!(
                room.script_block_changes
                    .iter()
                    .any(|change| u64::from(change.x)
                        == gate["at"][0].as_u64().expect("gate x") * 2
                        && u64::from(change.y) == gate["at"][1].as_u64().expect("gate y") * 2),
                "native changeblock arguments use core tiles, not metatile indices"
            );
        }
        assert_eq!(
            room.script_vertical_menus.len(),
            gym["stations"].as_array().expect("stations").len()
                + gym
                    .get("routers")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len)
        );
        assert_eq!(
            room.events.bg_events.len(),
            room.script_vertical_menus.len()
                - gym
                    .get("routers")
                    .and_then(Value::as_array)
                    .map_or(0, |routers| routers
                        .iter()
                        .filter(|r| r.get("operator").is_some())
                        .count())
                + gym
                    .get("switch_puzzle")
                    .map_or(0, |puzzle| puzzle["controls"]
                        .as_array()
                        .expect("switch controls")
                        .len())
        );
        let id = format!("MN_GYM_{}", index + 1);
        let trainer = &result.data().trainers.trainers[&id];
        assert_eq!(
            trainer.name,
            gym["leader"].as_str().expect("original leader name")
        );
        assert!(
            matches!(
                trainer.trainer_class.as_str(),
                "COOLTRAINERF" | "COOLTRAINERM" | "HIKER" | "GENTLEMAN" | "BIRD_KEEPER" | "SAILOR"
            ),
            "authored leaders use generic classes, never canon leader identities"
        );
        assert!(trainer.items.is_empty());
        let expected = gym["party"].as_array().expect("leader team");
        assert_eq!(trainer.party.len(), expected.len());
        for (actual, planned) in trainer.party.iter().zip(expected) {
            assert_eq!(
                actual.species,
                planned["species"].as_str().expect("species")
            );
            assert_eq!(
                u64::from(actual.level),
                planned["level"].as_u64().expect("level")
            );
            assert_eq!(
                actual
                    .moves
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>(),
                planned["moves"]
                    .as_array()
                    .expect("moves")
                    .iter()
                    .map(|m| m.as_str().expect("move"))
                    .collect::<Vec<_>>()
            );
        }
    }
    let rochester =
        build_minnesota_gym_review_room(&base, &design, 1).expect("one-gym review build");
    assert_eq!(rochester.data().maps.len(), base.data().maps.len() + 2);
    assert!(rochester.data().maps.contains_key("MnRochesterGym"));
    assert!(!rochester.data().maps.contains_key("MnMankatoGym"));
    assert_eq!(rochester.data().maps["MnGymReviewHall"].attributes.width, 5);
    assert_eq!(
        rochester.data().maps["MnGymReviewHall"].events.warps.len(),
        3
    );
    let mut unqualified_court = source.clone();
    unqualified_court["gyms"][0]["court_requires"] = serde_json::json!(["BADGE_1"]);
    assert!(
        build_minnesota_gym_review_room(&base, &unqualified_court.to_string(), 1).is_err(),
        "an open court must require its actual practice trainers"
    );
    let mankato =
        build_minnesota_gym_review_room(&base, &design, 2).expect("one-room terrace review");
    assert_eq!(mankato.data().maps.len(), base.data().maps.len() + 2);
    assert!(mankato.data().maps.contains_key("MnMankatoGym"));
    assert!(!mankato.data().maps.contains_key("MnRochesterGym"));
    let mut bad_ledge = source.clone();
    bad_ledge["gyms"][1]["ledges"][0]["landing"] = serde_json::json!([7, 10]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad_ledge.to_string(), 2).is_err(),
        "declared landing must match the native two-tile hop"
    );
    let mut bad_ledge = source.clone();
    bad_ledge["gyms"][1]["art"]["h"] = serde_json::json!(["cave", "02"]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad_ledge.to_string(), 2).is_err(),
        "ordinary dirt cannot substitute for a real one-way ledge"
    );
    let mut bypassed_ledge = source.clone();
    bypassed_ledge["gyms"][1]["rows"][4] = serde_json::json!("ltph...r");
    assert!(
        build_minnesota_gym_review_room(&base, &bypassed_ledge.to_string(), 2).is_err(),
        "the west bank cannot have a short walking bypass at its right end"
    );
    let mut bad_ledge = source.clone();
    bad_ledge["gyms"][1]["ledges"][0]["return_passage_core"] = serde_json::json!([1, 1, 2, 2]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad_ledge.to_string(), 2).is_err(),
        "return contract must seal the actual ascent"
    );
    assert!(
        build_minnesota_gym_review_pack(&result, &design).is_err(),
        "duplicate gym installation"
    );
    let mut bad = source.clone();
    bad["gyms"][0]["trainers"][0]["at"] = serde_json::json!([0, 0]);
    assert!(
        build_minnesota_gym_review_pack(&base, &bad.to_string()).is_err(),
        "trainer on solid wall"
    );
    let mut bad = source.clone();
    bad["gyms"][0]["party"][0]["moves"] = serde_json::json!(["SACRED_FIRE"]);
    assert!(
        build_minnesota_gym_review_pack(&base, &bad.to_string()).is_err(),
        "illegal leader move"
    );
    let minneapolis =
        build_minnesota_gym_review_room(&base, &design, 3).expect("isolated switch-pair room");
    assert_eq!(minneapolis.data().maps.len(), base.data().maps.len() + 2);
    let room = &minneapolis.data().maps["MnMinneapolisGym"];
    assert!(room.script_vertical_menus.is_empty());
    assert_eq!(room.events.bg_events.len(), 6);
    let st_paul = build_minnesota_gym_review_room(&base, &design, 4).expect("isolated lift hall");
    assert_eq!(st_paul.data().maps.len(), base.data().maps.len() + 2);
    assert_eq!(st_paul.data().maps["MnStPaulGym"].events.warps.len(), 11);
    let mut bad_lift = source.clone();
    bad_lift["gyms"][3]["lifts"][0]["ends_core"][0] = serde_json::json!([2, 11]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad_lift.to_string(), 4).is_err(),
        "lift endpoint must be the actual native pad quadrant"
    );
    let mut bad_lift = source.clone();
    bad_lift["gyms"][3]["lifts"][1]["ends_core"][0] = serde_json::json!([1, 11]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad_lift.to_string(), 4).is_err(),
        "two lifts cannot claim the same native pad"
    );
    let mut open_grille_end = source.clone();
    open_grille_end["gyms"][3]["rows"][2] = serde_json::json!("..pvWXW");
    assert!(
        build_minnesota_gym_review_room(&base, &open_grille_end.to_string(), 4).is_err(),
        "the court grille must join the side wall without a walking bypass"
    );
    let mut bad = source.clone();
    bad["gyms"][2]["switch_puzzle"]["events"][0] =
        serde_json::json!("EVENT_MN_GYM_3_TRAINER_1_DEFEATED");
    assert!(
        build_minnesota_gym_review_room(&base, &bad.to_string(), 3).is_err(),
        "a wrong switch must never clear a trainer victory"
    );
    let mut bad = source.clone();
    bad["gyms"][2]["switch_puzzle"]["controls"][3]["at"] = serde_json::json!([0, 0]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad.to_string(), 3).is_err(),
        "all switch cans must remain approachable"
    );
    let mut bad = source.clone();
    bad["gyms"][2]["switch_puzzle"]["sequence"][1] = serde_json::json!("LOWER_LEFT");
    assert!(
        build_minnesota_gym_review_room(&base, &bad.to_string(), 3).is_err(),
        "the hinted second switch must be in the adjacent can on the same row"
    );
    let mut bad = source.clone();
    bad["gyms"][2]["art"]["c"] = serde_json::json!(["game_corner", "01"]);
    assert!(
        build_minnesota_gym_review_room(&base, &bad.to_string(), 3).is_err(),
        "an invisible floor trigger cannot substitute for a solid switch can"
    );
    let cloud = build_minnesota_gym_review_room(&base, &source.to_string(), 5)
        .expect("one native wind-bridge room");
    assert_eq!(cloud.data().maps.len(), base.data().maps.len() + 2);
    let mut unsafe_wind = source.clone();
    unsafe_wind["gyms"][4]["rows"][6] = serde_json::json!("#..B>>#.#");
    assert!(
        build_minnesota_gym_review_room(&base, &unsafe_wind.to_string(), 5).is_err(),
        "native direct-step wind must never terminate inside a wall"
    );
    let mut hidden_wind = source.clone();
    hidden_wind["gyms"][4]["art"][">"] = serde_json::json!(["tower", "09"]);
    assert!(
        build_minnesota_gym_review_room(&base, &hidden_wind.to_string(), 5).is_err(),
        "ordinary wood must never conceal forced wind collision"
    );
    let brainerd = build_minnesota_gym_review_room(&base, &source.to_string(), 6)
        .expect("compact native pool gym");
    assert_eq!(brainerd.data().maps.len(), base.data().maps.len() + 2);
    let mut open_pool = source.clone();
    open_pool["gyms"][5]["rows"][3] = serde_json::json!("Wuudr");
    assert!(
        build_minnesota_gym_review_room(&base, &open_pool.to_string(), 6).is_err(),
        "indoor pool must never spill onto the map edge"
    );
    let mut bad_operator = source.clone();
    bad_operator["gyms"][5]["routers"][0]["operator"] = serde_json::json!("UNKNOWN_TRAINER");
    assert!(
        build_minnesota_gym_review_room(&base, &bad_operator.to_string(), 6).is_err(),
        "a trainer control must belong to its actual trainer"
    );
    let bemidji =
        build_minnesota_gym_review_room(&base, &source.to_string(), 7).expect("native Cut garden");
    assert_eq!(bemidji.data().maps.len(), base.data().maps.len() + 2);
    let mut wrong_cut = source.clone();
    wrong_cut["gyms"][6]["cut_gates"][0]["replacement_symbol"] = serde_json::json!(".");
    assert!(
        build_minnesota_gym_review_room(&base, &wrong_cut.to_string(), 7).is_err(),
        "Cut must use the supplied native replacement with its other quadrants intact"
    );
    let mut incidental_cut = source.clone();
    incidental_cut["gyms"][6]["rows"][5] = serde_json::json!("pc##.p");
    assert!(
        build_minnesota_gym_review_room(&base, &incidental_cut.to_string(), 7).is_err(),
        "every native Cut quadrant must be an authored gameplay gate"
    );
    let mut missing_cut = source.clone();
    missing_cut["gyms"][6]["rows"][3] = serde_json::json!("###o##");
    assert!(
        build_minnesota_gym_review_room(&base, &missing_cut.to_string(), 7).is_err(),
        "a declared tree must retain native Cut collision"
    );
    for (row, terrain) in [(6, "#c##.#"), (3, "#.#C##")] {
        let mut bypass = source.clone();
        bypass["gyms"][6]["rows"][row] = serde_json::json!(terrain);
        assert!(
            build_minnesota_gym_review_room(&base, &bypass.to_string(), 7).is_err(),
            "the hedge must seal its named court even after every other tree is cut"
        );
    }
    let mut bad = source;
    let width = bad["gyms"][2]["dimensions_metatiles"][0]
        .as_u64()
        .expect("locked gym width") as usize;
    bad["gyms"][2]["rows"][3] = serde_json::json!(format!("#{}#", ".".repeat(width - 2)));
    bad["gyms"][2]["rows"][2] = serde_json::json!(format!("#{}#", ".".repeat(width - 2)));
    assert!(
        build_minnesota_gym_review_pack(&base, &bad.to_string()).is_err(),
        "unearned leader-court bypass"
    );
}
