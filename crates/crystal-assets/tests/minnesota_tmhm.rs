//! Full external-pack preservation and purchase/teaching coverage.

use std::path::PathBuf;

use crystal_assets::{build_minnesota_tmhm, read_verified_compiled_game_pack};
use crystal_core::models::Dv;
use crystal_core::state::GameState;
use crystal_core::systems::shop;
use serde_json::Value;

#[test]
#[ignore = "requires the completed Minnesota cries pack and published TM/HM pack"]
fn minnesota_tmhm_preserves_content_and_every_machine_can_be_bought_and_taught() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let source = std::fs::read_to_string(root.join("planning/minnesota/tmhm-distribution.json"))
        .expect("authored distribution");
    let contract: Value = serde_json::from_str(&source).expect("distribution JSON");
    let base =
        read_verified_compiled_game_pack(root.join("content-packs/minnesota-cries.crystalpack"))
            .expect("completed parent");
    let result = build_minnesota_tmhm(&base, &source).expect("build machine distribution");
    let written =
        read_verified_compiled_game_pack(root.join("content-packs/minnesota-all-tmhm.crystalpack"))
            .expect("published distribution");
    assert_eq!(
        result.identity().expect("built identity"),
        written.identity().expect("written identity")
    );
    assert!(result.compiled_audio() == base.compiled_audio());
    assert!(result.runtime_files() == base.runtime_files());
    let before = serde_json::to_value(base.data()).expect("before dataset");
    let after = serde_json::to_value(result.data()).expect("after dataset");
    for (field, value) in before.as_object().expect("dataset fields") {
        if ![
            "marts",
            "items",
            "maps",
            "map_scripts",
            "story_event_script_constants",
        ]
        .contains(&field.as_str())
        {
            assert!(
                after.get(field) == Some(value),
                "unrelated dataset changed: {field}"
            );
        }
    }
    for (id, stock) in &base.data().marts.0 {
        if id != "MART_MN_MP_DEPT_5_CLERK" {
            assert_eq!(&result.data().marts.0[id], stock);
        }
    }
    for (id, item) in &base.data().items {
        let mut expected = item.clone();
        if id.starts_with("HM_") {
            expected.price = 3000;
        }
        assert_eq!(result.data().items[id], expected, "item {id}");
    }
    for (id, map) in &base.data().maps {
        let changed = contract["hidden_caches"]
            .as_array()
            .expect("caches")
            .iter()
            .any(|c| c["map"] == *id);
        let current = &result.data().maps[id];
        if !changed {
            assert_eq!(map, current, "unrelated map {id}");
        } else {
            assert_eq!(map.blocks, current.blocks);
            assert_eq!(map.attributes, current.attributes);
            assert_eq!(map.objects, current.objects);
            assert_eq!(map.events.warps, current.events.warps);
            assert_eq!(map.events.coord_events, current.events.coord_events);
            assert!(current.events.bg_events.starts_with(&map.events.bg_events));
            for pickup in &map.script_field_pickups {
                assert!(current.script_field_pickups.contains(pickup));
            }
            for (label, program) in &map.scripts {
                assert_eq!(current.scripts.get(label), Some(program));
            }
        }
    }
    let data = result.data();
    let stock = &data.marts.0["MART_MN_MP_DEPT_5_CLERK"];
    assert_eq!(stock.len(), 57);
    for (index, id) in stock.iter().enumerate() {
        let item = &data.items[id];
        assert_eq!(item.tmhm_index, Some(index + 1));
        assert!(
            data.pokemon.values().any(|species| {
                species.int_id > 251
                    && item
                        .tmhm_move
                        .as_ref()
                        .is_some_and(|move_id| species.tmhm_learnset.contains(move_id))
            }),
            "at least one original Minnesota species can learn {id}"
        );
        let mut state = GameState {
            money: 999_999,
            ..Default::default()
        };
        let pokemon = data
            .create_pokemon("MEW", 5, Dv::default())
            .expect("compatible native species");
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        let opened = data
            .open_script_shop(
                &mut state,
                "MnMinneapolisDepartmentStore5F",
                "MnMinneapolisDepartmentStore5F",
                "MN_MP_DEPT_5_CLERK",
                2,
            )
            .expect("production clerk shop");
        assert_eq!(&opened.inventory, stock);
        assert!(shop::max_buy_quantity(&state, item) > 0);
        assert!(
            shop::buy_active_shop_item(&mut state, &data.items, id, 1)
                .expect("buy machine")
                .success
        );
        assert_eq!(state.money, 999_999 - u32::from(item.price));
        let (_, learned) = data
            .use_bag_tmhm_on_party_pokemon(&mut state, id, 0, None)
            .unwrap_or_else(|error| panic!("teach {id}: {error:#}"));
        assert_eq!(Some(&learned.learned_move), item.tmhm_move.as_ref());
        assert_eq!(learned.consumed, index < 50);
        assert_eq!(state.bag.quantity(item), u16::from(index >= 50));
        let pokemon = state.storage.party.pokemon[0]
            .as_ref()
            .expect("taught partner");
        let learned_move = pokemon.moves.last().expect("learned move");
        assert_eq!(
            learned_move.current_pp,
            data.moves[&learned.learned_move].pp
        );
        if index >= 50 {
            assert!(
                state.storage.party.add_pokemon(
                    data.create_pokemon("MEW", 5, Dv::default())
                        .expect("second compatible partner")
                )
            );
            state.sync_party_from_storage();
            let (_, reused) = data
                .use_bag_tmhm_on_party_pokemon(&mut state, id, 1, None)
                .expect("reuse the purchased HM on a second partner");
            assert!(!reused.consumed);
            assert_eq!(state.bag.quantity(item), 1);
        }
    }
    assert!(build_minnesota_tmhm(&result, &source).is_err());
    for (field, value) in [
        ("base_content_hash", Value::String("stale".into())),
        ("hm_price", Value::from(0)),
        ("hidden_caches", Value::Array(vec![])),
        ("unexpected", Value::Bool(true)),
    ] {
        let mut wrong = contract.clone();
        wrong[field] = value;
        assert!(
            build_minnesota_tmhm(&base, &wrong.to_string()).is_err(),
            "reject {field}"
        );
    }
}
