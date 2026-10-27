//! Native asset preparation only; actual publication/controller proof is separate.
use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_core::models::Dv;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::script_items::ScriptItemGrantOutcome;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use serde_json::{Value, json};

use super::*;
use crate::{
    ROOT_BEER_FIRST_GIFT_EVENT, ROOT_BEER_MART_ID, RuntimeMutationCommand, RuntimeMutationResult,
    RuntimeScriptCommandRef,
};

fn inputs() -> &'static (CompiledGamePack, String, String, MapModule) {
    static INPUTS: OnceLock<(CompiledGamePack, String, String, MapModule)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let path = std::env::var_os("CRYSTAL_ROOT_BEER_CABIN_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/text-tui.crystalpack"))
            .canonicalize()
            .expect("external native source");
        let source = crate::read_verified_compiled_game_pack(path).expect("verified source");
        let pack = if source.data().tilesets.contains_key("mn_boundary_safari") {
            source
        } else {
            crate::build_boundary_safari_art_pack(&source).expect("actual native preserve art")
        };
        let cabin =
            std::fs::read_to_string(root.join("planning/minnesota/playable/root-beer-cabin.json"))
                .expect("actual authored cabin");
        let terrain = std::fs::read_to_string(
            root.join("planning/minnesota/zones/boundary-waters-safari.layout.json"),
        )
        .expect("actual authored preserve");
        let layout = Layout::parse(&terrain).expect("native terrain parser");
        let blocks = layout.compile().expect("actual preserve strokes");
        // Explicit unpublished asset fixture: clear a native module schema,
        // then use the actual compiled preserve chunk and authored Granny.
        // This is neither a cloned house nor a published Safari session.
        let mut exterior = authoring::blank(
            &pack,
            "Route32",
            "MnBoundarySafariC00R00",
            "MN_BOUNDARY_SAFARI_C00R00",
        )
        .expect("cleared exterior schema");
        exterior.attributes.tileset_name = "mn_boundary_safari".into();
        exterior.attributes.width = 32;
        exterior.attributes.height = 24;
        exterior.blocks = (0..24)
            .flat_map(|y| blocks[y * 48..y * 48 + 32].iter().copied())
            .collect();
        authoring::speech(
            &pack,
            &mut exterior,
            "Dorothy",
            "SPRITE_GRANNY",
            (49, 43),
            &["Welcome, paddler!"],
        )
        .expect("actual authored outside Dorothy");
        exterior.objects[0].object_identifier = Some("BWS_ROOT_BEER_LADY".into());
        exterior.objects[0].spritemovedata = "SPRITEMOVEDATA_STANDING_RIGHT".into();
        authoring::finish(&pack, &mut exterior).expect("native fixture parsing");
        (pack, cabin, terrain, exterior)
    })
}

fn prepared(rest: RootBeerRestPolicy) -> PreparedRootBeerCabin {
    let (pack, cabin, layout, exterior) = inputs();
    prepare_root_beer_cabin(pack, cabin, layout, exterior, rest).expect("actual cabin preparation")
}

#[test]
fn root_beer_cabin_uses_exact_native_rows_passive_arrival_and_one_outside_dorothy() {
    let (base, source, _, exterior) = inputs();
    let prepared = prepared(RootBeerRestPolicy::Offer);
    assert_eq!(prepared.pack.data().maps, base.data().maps);
    assert_eq!(
        prepared.pack.data().runtime_map_metadata,
        base.data().runtime_map_metadata
    );
    assert_eq!(
        prepared.pack.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(prepared.pack.data().trainers, base.data().trainers);
    assert_eq!(prepared.pack.data().tilesets, base.data().tilesets);
    assert_eq!(prepared.pack.runtime_files(), base.runtime_files());
    assert_eq!(prepared.pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        prepared.pack.audio_manifest().expect("prepared audio"),
        base.audio_manifest().expect("source audio")
    );
    for (name, item) in &base.data().items {
        assert_eq!(prepared.pack.data().items.get(name), Some(item));
    }
    let source: Value = serde_json::from_str(source).expect("cabin source");
    let expected = source["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|block| u16::from_str_radix(block, 16).expect("native block"))
        .collect::<Vec<_>>();
    assert_eq!(prepared.cabin.blocks, expected);
    assert_ne!(
        prepared.cabin.blocks,
        base.data().maps["VioletNicknameSpeechHouse"].blocks,
        "actual rows must replace the source house"
    );
    assert_eq!(
        (
            prepared.cabin.attributes.width,
            prepared.cabin.attributes.height
        ),
        (5, 4)
    );
    assert_eq!(prepared.cabin.attributes.tileset_name, "traditional_house");
    assert_eq!(prepared.cabin.attributes.border_block, 0);
    assert!(prepared.cabin.objects.is_empty());
    assert!(prepared.cabin.scripts.is_empty());
    assert!(prepared.cabin.events.bg_events.is_empty());
    assert!(prepared.cabin.events.coord_events.is_empty());
    assert_eq!(prepared.arrival, [5, 6]);
    assert_eq!(prepared.exterior_return, [51, 40]);
    assert_eq!(prepared.cabin.events.warps.len(), 3);
    for (warp, (index, tile, kind)) in prepared.cabin.events.warps.iter().zip([
        (1, [5, 7], "WARP_CARPET_DOWN"),
        (2, [6, 7], "WARP_CARPET_DOWN"),
        (3, [5, 6], "FLOOR"),
    ]) {
        assert_eq!(warp.index, index);
        assert_eq!([warp.x, warp.y], tile);
        assert_eq!(warp.target_map, "MN_BOUNDARY_SAFARI_C00R00");
        assert_eq!(warp.target_map_constant, "MN_BOUNDARY_SAFARI_C00R00");
        assert_eq!(warp.target_warp_id, 2);
        assert_eq!(
            collision(base, &prepared.cabin.blocks, 5, tile, "traditional_house")
                .expect("native warp kind"),
            kind
        );
    }
    assert_eq!(prepared.exterior.events.warps.len(), 2);
    for (warp, (index, tile, kind)) in prepared
        .exterior
        .events
        .warps
        .iter()
        .zip([(1, [51, 39], "DOOR"), (2, [51, 40], "FLOOR")])
    {
        assert_eq!(warp.index, index);
        assert_eq!([warp.x, warp.y], tile);
        assert_eq!(warp.target_map, ROOT_BEER_CABIN_CONSTANT);
        assert_eq!(warp.target_map_constant, ROOT_BEER_CABIN_CONSTANT);
        assert_eq!(warp.target_warp_id, 3);
        assert_eq!(
            collision(base, &exterior.blocks, 32, tile, "mn_boundary_safari")
                .expect("exterior kind"),
            kind
        );
    }
    assert_eq!(prepared.exterior.blocks, exterior.blocks);
    assert_eq!(prepared.exterior.attributes, exterior.attributes);
    let mut expected_objects = exterior.objects.clone();
    expected_objects[0].script = prepared.service_script.clone();
    assert_eq!(prepared.exterior.objects, expected_objects);
    for (name, script) in &exterior.scripts {
        assert_eq!(prepared.exterior.scripts.get(name), Some(script));
    }
    let kind = |at| {
        collision(base, &prepared.cabin.blocks, 5, at, "traditional_house")
            .expect("native cabin collision")
    };
    let floor = (0..8)
        .flat_map(|y| (0..10).map(move |x| [x, y]))
        .filter(|&tile| kind(tile) == "FLOOR")
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::from([[5_u16, 6_u16]]);
    let mut queue = VecDeque::from([[5_u16, 6_u16]]);
    while let Some([x, y]) = queue.pop_front() {
        for next in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if floor.contains(&next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    assert_eq!(floor.len(), 52);
    assert_eq!(seen, floor, "all actual cabin floor remains reachable");
    for y in 2..=5 {
        assert_eq!(kind([4, y]), "COUNTER");
    }
    assert!(seen.contains(&[5, 3]) && seen.contains(&[3, 3]));
    assert_eq!(prepared.pack.data().items[ROOT_BEER_ITEM_ID].price, 300);
    assert_eq!(
        prepared.pack.data().marts.0[ROOT_BEER_MART_ID],
        [ROOT_BEER_ITEM_ID]
    );
    assert!(
        prepared
            .exterior
            .script_runtime_commands
            .iter()
            .any(|command| command.command == "special" && command.args == ["HealParty"])
    );
    assert!(
        prepared
            .exterior
            .scripts
            .iter()
            .filter(|(label, _)| label.starts_with(&prepared.service_script))
            .all(|(_, script)| !script.to_string().to_ascii_lowercase().contains("safari"))
    );
}

#[test]
fn root_beer_cabin_rejects_malformed_arrivals_and_actual_nonfloor_passive_targets() {
    let (pack, cabin, terrain, exterior) = inputs();
    for replacement in [
        json!({"index":2,"tile":[5,6],"collision":"FLOOR","active_trigger":false}),
        json!({"index":3,"tile":[5,7],"collision":"FLOOR","active_trigger":false}),
        json!({"index":3,"tile":[5,6],"collision":"FLOOR","active_trigger":true}),
        json!({"index":3,"tile":[4,3],"collision":"COUNTER","active_trigger":false}),
    ] {
        let mut source: Value = serde_json::from_str(cabin).expect("cabin");
        source["warp_contract"]["interior_passive_arrival_warp"] = replacement;
        assert!(
            prepare_root_beer_cabin(
                pack,
                &source.to_string(),
                terrain,
                exterior,
                RootBeerRestPolicy::Offer
            )
            .is_err()
        );
    }
    let prepared = prepared(RootBeerRestPolicy::Offer);
    for tile in [[4, 3], [5, 7]] {
        let mut source: Value = serde_json::from_str(cabin).expect("cabin");
        source["passive_arrival"] = json!({"index":3,"tile":tile});
        source["clear_arrival"] = json!(tile);
        let room: Interior =
            serde_json::from_value(source).expect("malformed typed arrival fixture");
        let Err(error) = validate_room(pack, &prepared.cabin, &room) else {
            panic!("native nonfloor arrival accepted");
        };
        assert!(
            error.to_string().contains("passive room arrival"),
            "{error:#}"
        );
        assert_ne!(
            collision(pack, &prepared.cabin.blocks, 5, tile, "traditional_house")
                .expect("actual target collision"),
            "FLOOR"
        );
    }
}

#[test]
fn root_beer_cabin_rejects_fake_exterior_door_return_actor_and_duplicate_inside_presence() {
    let (pack, cabin, terrain, exterior) = inputs();
    for (key, value) in [
        ("target_warp", json!(1)),
        ("return_warp_index", json!(1)),
        ("return_tile", json!([51, 39])),
    ] {
        let mut layout: Value = serde_json::from_str(terrain).expect("terrain");
        layout["root_beer_lady"]["door"][key] = value;
        assert!(
            prepare_root_beer_cabin(
                pack,
                cabin,
                &layout.to_string(),
                exterior,
                RootBeerRestPolicy::Offer
            )
            .is_err()
        );
    }
    let mut bad = exterior.clone();
    bad.blocks[0] = 1;
    assert!(
        prepare_root_beer_cabin(pack, cabin, terrain, &bad, RootBeerRestPolicy::Offer).is_err()
    );
    bad = exterior.clone();
    bad.objects.clear();
    assert!(
        prepare_root_beer_cabin(pack, cabin, terrain, &bad, RootBeerRestPolicy::Offer).is_err()
    );
    bad = exterior.clone();
    bad.objects.push(bad.objects[0].clone());
    assert!(
        prepare_root_beer_cabin(pack, cabin, terrain, &bad, RootBeerRestPolicy::Offer).is_err()
    );
    let mut source: Value = serde_json::from_str(cabin).expect("cabin");
    source["residents"] = json!([{"id":"SECOND_DOROTHY","tile":[3,3],"sprite":"SPRITE_GRANNY","facing":"east","lines":["Hello."]}]);
    assert!(
        prepare_root_beer_cabin(
            pack,
            &source.to_string(),
            terrain,
            exterior,
            RootBeerRestPolicy::Offer
        )
        .is_err()
    );
    source = serde_json::from_str(cabin).expect("cabin");
    source["rows"][3] = json!("20 20 00 22 20");
    assert!(
        prepare_root_beer_cabin(
            pack,
            &source.to_string(),
            terrain,
            exterior,
            RootBeerRestPolicy::Offer
        )
        .is_err(),
        "native blocked arrival/carpet accepted"
    );
}

#[test]
fn root_beer_cabin_outside_service_uses_actual_native_gift_shop_and_rest_mutations() {
    let prepared = prepared(RootBeerRestPolicy::Offer);
    let mut data = prepared.pack.data().clone();
    data.maps
        .insert(prepared.exterior.id.clone(), prepared.exterior.clone());
    data.maps
        .insert(prepared.cabin.id.clone(), prepared.cabin.clone());
    // Explicit unpublished asset-mutation fixture on the actual authored
    // preserve module. No VisibleShellController or published Safari is claimed.
    let mut state = GameState::reset_wram_for_new_game();
    state.overworld = OverworldMemory::Active {
        map_name: prepared.exterior.id.clone(),
        tile: TilePosition::new(50, 43),
        facing: Direction::Left,
        mode: MovementMode::Normal,
    };
    let mut partner = data
        .create_pokemon("MEW", 45, Dv::default())
        .expect("source-created partner");
    partner.hp = 1;
    partner.status = Some("POISON".into());
    for movement in &mut partner.moves {
        movement.current_pp = 0;
    }
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    let map = &prepared.exterior.id;
    let script = &prepared.service_script;
    let mut session = data
        .overworld_session(map, TilePosition::new(50, 43), 0)
        .expect("actual exterior asset session");
    let audio = data.audio_ids().into_iter().map(str::to_owned).collect();
    let delivered = data
        .apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::GrantScriptItem(RuntimeScriptCommandRef::new(map, script, 7)),
            &audio,
            &audio,
            &audio,
        )
        .expect("native outside gift grant");
    assert!(matches!(
        delivered.result,
        RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::Granted {
            quantity: 1,
            ..
        })
    ));
    data.apply_script_flag_mutation(&mut state, &mut session, map, map, script, 9)
        .expect("native delivered gift flag");
    assert_eq!(state.bag.quantity(&data.items[ROOT_BEER_ITEM_ID]), 1);
    assert_eq!(
        state.flags.is_event_flag_set(ROOT_BEER_FIRST_GIFT_EVENT),
        Ok(true)
    );
    state.money = 600;
    data.open_script_shop(&mut state, map, map, &format!("{script}Shop"), 5)
        .expect("actual parsed standard mart");
    assert!(
        data.buy_shop_item(&mut state, ROOT_BEER_ITEM_ID, 1)
            .expect("native purchase")
            .success
    );
    assert_eq!(state.money, 300);
    assert_eq!(state.bag.quantity(&data.items[ROOT_BEER_ITEM_ID]), 2);
    let resources = (
        state.money,
        state.bag.clone(),
        state.flags.clone(),
        state.script_runtime.memory.clone(),
    );
    data.apply_special_routine(&mut state, "HealParty", &BTreeSet::new())
        .expect("native nursing rest");
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("rested partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(partner.status, None);
    assert!(partner.moves.iter().all(|movement| movement.current_pp > 0));
    assert_eq!(
        (
            state.money,
            state.bag.clone(),
            state.flags.clone(),
            state.script_runtime.memory.clone()
        ),
        resources
    );
    let omitted = prepared_without_rest();
    assert!(
        !omitted
            .exterior
            .script_runtime_commands
            .iter()
            .any(
                |command| command.source_script.starts_with(&omitted.service_script)
                    && command.command == "special"
            )
    );
}

fn prepared_without_rest() -> PreparedRootBeerCabin {
    prepared(RootBeerRestPolicy::Omit)
}
