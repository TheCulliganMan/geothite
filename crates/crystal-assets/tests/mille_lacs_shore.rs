//! Native quest components after explicit position, badge and inventory fixtures.
//!
//! These invoke production mutation/control/save APIs, not a local script
//! interpreter. VisibleShellController still must prove dialogue ownership and
//! an earned walking journey after the shared owner publishes the actual pack.
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    CompiledGamePack, GameDataSet, RuntimeMutationCommand, RuntimeMutationResult,
    RuntimeScriptCommandRef, SHORE_WATCH_REPORT_EVENTS, SHORE_WATCH_REWARDED_EVENT,
    build_iron_range_descent_modpack, build_mille_lacs_shore_modpack,
    read_verified_compiled_game_pack,
};
use crystal_core::models::bag::KEY_ITEM_POCKET_CAPACITY;
use crystal_core::save::{
    SaveModpackIdentity, read_save_game_for_modpack, write_save_game_for_modpack,
};
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::script_control::ScriptControlAction;
use crystal_core::systems::script_items::ScriptItemGrantOutcome;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use serde_json::{Value, json};

const MAP: &str = "MnR18C00R00";
const RANGER: &str = "MN_R18_DOCK_RANGER";

fn inputs() -> &'static (CompiledGamePack, CompiledGamePack, String, String) {
    static PACKS: OnceLock<(CompiledGamePack, CompiledGamePack, String, String)> = OnceLock::new();
    PACKS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonical repository");
        let source =
            read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
                .expect("supplied external pack");
        let read = |name: &str| {
            fs::read_to_string(root.join("planning/minnesota/routes").join(name))
                .expect("authoring JSON")
        };
        let base = build_iron_range_descent_modpack(
            &source,
            &read("MN_R10.layout.json"),
            &read("MN_R10.json"),
        )
        .expect("existing descent overlay");
        let layout = read("MN_R18.layout.json");
        let dossier = read("MN_R18.json");
        let pack = build_mille_lacs_shore_modpack(&base, &layout, &dossier)
            .expect("native shore and SQ04");
        (base, pack, layout, dossier)
    })
}

fn state() -> GameState {
    let mut state = GameState::reset_wram_for_new_game();
    state.overworld = OverworldMemory::Active {
        map_name: MAP.into(),
        tile: TilePosition::new(26, 23),
        facing: Direction::Right,
        mode: MovementMode::Normal,
    };
    state
}

fn set_reports(data: &GameDataSet, state: &mut GameState, count: usize) {
    for (index, actor) in [
        "MN_R18_WATCH_REED",
        "MN_R18_WATCH_MEADOW",
        "MN_R18_WATCH_BEACH",
    ]
    .into_iter()
    .enumerate()
    .take(count)
    {
        let map = if index == 2 { "MnR18C01R00" } else { MAP };
        let mut session = data
            .overworld_session(map, TilePosition::new(2, 6), 0)
            .expect("report component session");
        data.apply_script_flag_mutation(state, &mut session, map, map, actor, 5)
            .expect("native report event after authored dialogue");
    }
}

#[test]
fn mille_lacs_publishes_exact_quest_tables_and_preserves_existing_world() {
    let (base, pack, _, dossier) = inputs();
    assert_eq!(pack.data().maps.len(), base.data().maps.len() + 2);
    for (name, module) in &base.data().maps {
        assert_eq!(
            pack.data().maps.get(name),
            Some(module),
            "changed map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(name),
            Some(bytes),
            "changed file {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(pack.data().tilesets.get(name), Some(tileset));
    }
    assert_eq!(pack.data().items, base.data().items);
    assert_eq!(pack.data().marts, base.data().marts);
    assert_eq!(pack.data().pokemon, base.data().pokemon);
    assert_eq!(
        pack.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("quest audio"),
        base.audio_manifest().expect("source audio")
    );
    let globals = &pack.data().story_event_script_constants.global;
    for (name, value) in &base.data().story_event_script_constants.global {
        assert_eq!(globals.get(name), Some(value), "changed constant {name}");
    }
    let ids: BTreeSet<_> = SHORE_WATCH_REPORT_EVENTS
        .into_iter()
        .chain([SHORE_WATCH_REWARDED_EVENT])
        .map(|event| globals[event])
        .collect();
    assert_eq!(ids.len(), 4);
    assert!(ids.iter().all(|value| {
        !base
            .data()
            .story_event_script_constants
            .global
            .iter()
            .any(|(name, old)| name.starts_with("EVENT_") && old == value)
    }));
    let design: Value = serde_json::from_str(dossier).expect("dossier");
    for (id, commands) in design["quest_contract"]["script_labels"]
        .as_object()
        .expect("quest roots")
    {
        let module = if id == "MN_R18_WATCH_BEACH" {
            &pack.data().maps["MnR18C01R00"]
        } else {
            &pack.data().maps[MAP]
        };
        assert_eq!(&module.scripts[id], commands, "quest root differs {id}");
    }
    let west = &pack.data().maps[MAP];
    let east = &pack.data().maps["MnR18C01R00"];
    for (module, width, objects, trainers, signs) in [(west, 32, 12, 6, 5), (east, 8, 2, 0, 1)] {
        assert_eq!(
            (module.attributes.width, module.attributes.height),
            (width, 24)
        );
        assert_eq!(module.objects.len(), objects);
        assert_eq!(module.trainer_scripts.len(), trainers);
        assert_eq!(module.events.bg_events.len(), signs);
        assert_eq!(module.attributes.tileset_name, "mn_headwaters");
        assert_eq!(module.attributes.connections.len(), 1);
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(&module.id)
                .expect("native split payload"),
            *module
        );
    }
    assert!(
        west.script_item_grants
            .iter()
            .any(|grant| grant.source_script == RANGER
                && grant.command_index == 18
                && grant.item_id == "GOOD_ROD"
                && grant.quantity == 1)
    );
    assert!(
        west.script_control_commands
            .iter()
            .any(|command| command.source_script == RANGER
                && command.command_index == 19
                && command.command == "iffalse"
                && command.target_label.as_deref() == Some("MN_R18_WatchBagFull"))
    );
    assert!(
        west.script_flag_commands
            .iter()
            .any(|command| command.source_script == RANGER
                && command.command_index == 20
                && command.command == "setevent"
                && command.flag_id == SHORE_WATCH_REWARDED_EVENT)
    );
}

#[test]
fn shore_watch_native_badge_and_each_report_guard_deny_early_reward() {
    let data = inputs().1.data();
    let mut state = state();
    data.check_script_flag(&mut state, MAP, MAP, RANGER, 1)
        .expect("native badge check");
    assert!(
        matches!(data.apply_script_control_command(&mut state,MAP,MAP,RANGER,2).expect("badge denial"),ScriptControlAction::Jump {target_script,..} if target_script=="MN_R18_WatchBadgeDenied")
    );
    state
        .flags
        .set_engine_flag("ENGINE_MINERALBADGE", true)
        .expect("explicit sixth-badge fixture");
    for index in 0..3 {
        data.check_script_flag(&mut state, MAP, MAP, RANGER, 5 + index * 2)
            .expect("missing report check");
        assert!(
            matches!(data.apply_script_control_command(&mut state,MAP,MAP,RANGER,6+index*2).expect("missing report denial"),ScriptControlAction::Jump {target_script,..} if target_script=="MN_R18_WatchNeedsNews")
        );
    }
    assert_eq!(state.bag.quantity(&data.items["GOOD_ROD"]), 0);
    assert_eq!(
        state.flags.is_event_flag_set(SHORE_WATCH_REWARDED_EVENT),
        Ok(false)
    );
}

#[test]
fn shore_watch_native_key_pocket_retry_sets_completion_only_after_real_delivery_and_survives_save()
{
    let pack = &inputs().1;
    let mut data = pack.data().clone();
    let mut state = state();
    state
        .flags
        .set_engine_flag("ENGINE_MINERALBADGE", true)
        .expect("badge fixture");
    set_reports(&data, &mut state, 3);
    // The original catalog has only20 non-reward key items against25 slots.
    // Five explicit cloned-key fixture definitions exercise a mod-filled pocket;
    // no new item or alias is added to the published content pack.
    for index in 0..5 {
        let mut item = data.items["OLD_ROD"].clone();
        item.script_name = format!("MN_R18_TEST_KEY_{index}");
        data.items.insert(item.script_name.clone(), item);
    }
    for item in data
        .items
        .values()
        .filter(|item| {
            item.pocket == "KEY_ITEM"
                && !matches!(item.script_name.as_str(), "GOOD_ROD" | "SUPER_ROD")
        })
        .take(KEY_ITEM_POCKET_CAPACITY)
    {
        assert!(
            state
                .bag
                .add_item(item, 1)
                .expect("full key-pocket fixture")
        );
    }
    assert_eq!(state.bag.key_items.len(), KEY_ITEM_POCKET_CAPACITY);
    assert_eq!(state.bag.quantity(&data.items["OLD_ROD"]), 1);
    let audio = data.audio_ids().into_iter().map(str::to_owned).collect();
    let mut session = data
        .overworld_session(MAP, TilePosition::new(26, 23), 0)
        .expect("ranger component session");
    let before = (state.bag.clone(), state.flags.clone());
    let failed = data
        .apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::GrantScriptItem(RuntimeScriptCommandRef::new(MAP, RANGER, 18)),
            &audio,
            &audio,
            &audio,
        )
        .expect("native full-key-pocket result");
    assert!(matches!(
        failed.result,
        RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::BagFull { .. })
    ));
    assert_eq!((state.bag.clone(), state.flags.clone()), before);
    assert!(
        matches!(data.apply_script_control_command(&mut state,MAP,MAP,RANGER,19).expect("failure guard"),ScriptControlAction::Jump {target_script,..} if target_script=="MN_R18_WatchBagFull")
    );
    let freed = "MN_R18_TEST_KEY_0";
    assert!(
        state
            .bag
            .remove_item(&data.items[freed], 1)
            .expect("free explicit fixture slot")
    );
    let delivered = data
        .apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::GrantScriptItem(RuntimeScriptCommandRef::new(MAP, RANGER, 18)),
            &audio,
            &audio,
            &audio,
        )
        .expect("native retry delivery");
    assert!(matches!(
        delivered.result,
        RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::Granted {
            quantity: 1,
            ..
        })
    ));
    assert!(matches!(
        data.apply_script_control_command(&mut state, MAP, MAP, RANGER, 19)
            .expect("delivery fallthrough"),
        ScriptControlAction::Continue { .. }
    ));
    data.apply_script_flag_mutation(&mut state, &mut session, MAP, MAP, RANGER, 20)
        .expect("completion only after native successful delivery");
    assert_eq!(state.bag.quantity(&data.items["GOOD_ROD"]), 1);
    assert_eq!(state.bag.quantity(&data.items["OLD_ROD"]), 1);
    let hash = pack
        .identity()
        .expect("actual pack identity")
        .content_hash
        .clone();
    let identity = SaveModpackIdentity::new("mille-lacs-component-fixture", hash.clone())
        .expect("explicit component save identity");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "geothite-shore-watch-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&directory).expect("owned disposable save directory");
    let path = directory.join("watch.crystalsave");
    write_save_game_for_modpack(&path, state.clone(), &identity, &hash)
        .expect("production save writer");
    let restored =
        read_save_game_for_modpack(&path, &identity, &hash).expect("production save reader");
    let mut restored = restored.state().clone();
    for event in SHORE_WATCH_REPORT_EVENTS
        .into_iter()
        .chain([SHORE_WATCH_REWARDED_EVENT])
    {
        assert_eq!(restored.flags.is_event_flag_set(event), Ok(true));
    }
    assert_eq!(restored.bag, state.bag);
    data.check_script_flag(&mut restored, MAP, MAP, RANGER, 3)
        .expect("next visit completion check");
    assert!(
        matches!(data.apply_script_control_command(&mut restored,MAP,MAP,RANGER,4).expect("no duplicate rod after saved completion"),ScriptControlAction::Jump{target_script,..} if target_script=="MN_R18_WatchAlreadyRewarded")
    );
    fs::remove_dir_all(directory).expect("remove owned component save fixture");
}

#[test]
fn shore_watch_native_existing_good_or_super_rod_acknowledges_without_duplicate_or_downgrade() {
    let data = inputs().1.data();
    for (rod, check, branch) in [("SUPER_ROD", 11, 12), ("GOOD_ROD", 13, 14)] {
        let mut state = state();
        state
            .flags
            .set_engine_flag("ENGINE_MINERALBADGE", true)
            .expect("badge fixture");
        set_reports(data, &mut state, 3);
        for id in ["OLD_ROD", rod] {
            assert!(
                state
                    .bag
                    .add_item(&data.items[id], 1)
                    .expect("existing rod fixture")
            );
        }
        let before = state.bag.clone();
        let audio = data.audio_ids().into_iter().map(str::to_owned).collect();
        let mut session = data
            .overworld_session(MAP, TilePosition::new(26, 23), 0)
            .expect("ranger component session");
        data.apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::CheckScriptItem(RuntimeScriptCommandRef::new(
                MAP, RANGER, check,
            )),
            &audio,
            &audio,
            &audio,
        )
        .expect("native adequate-rod check");
        assert!(
            matches!(data.apply_script_control_command(&mut state,MAP,MAP,RANGER,branch).expect("adequate rod acknowledgment"),ScriptControlAction::Jump{target_script,..} if target_script=="MN_R18_WatchHasRod")
        );
        data.apply_script_flag_mutation(
            &mut state,
            &mut session,
            MAP,
            MAP,
            "MN_R18_WatchHasRod",
            4,
        )
        .expect("native acknowledgment completion");
        assert_eq!(state.bag, before);
        assert_eq!(
            state.flags.is_event_flag_set(SHORE_WATCH_REWARDED_EVENT),
            Ok(true)
        );
    }
}

#[test]
fn shore_watch_rejects_fake_delivery_guards_and_undeclared_surf_bypass() {
    let (base, _, layout, dossier) = inputs();
    let mut changed: Value = serde_json::from_str(dossier).expect("dossier");
    changed["quest_contract"]["script_labels"][RANGER][19]["args"] = json!(["MN_R18_WatchHasRod"]);
    assert!(
        build_mille_lacs_shore_modpack(base, layout, &changed.to_string()).is_err(),
        "full bag may not falsely acknowledge reward"
    );
    let mut changed: Value = serde_json::from_str(dossier).expect("dossier");
    changed["surf_only_actor_ids"] = json!([]);
    assert!(
        build_mille_lacs_shore_modpack(base, layout, &changed.to_string()).is_err(),
        "Surf-only pickup must not skip walking validation"
    );
    let mut changed: Value = serde_json::from_str(dossier).expect("dossier");
    changed["surf_only_actor_ids"] = json!(["MN_R18_DOCK_RANGER"]);
    assert!(
        build_mille_lacs_shore_modpack(base, layout, &changed.to_string()).is_err(),
        "SQ04 ranger must remain walking accessible"
    );
}
