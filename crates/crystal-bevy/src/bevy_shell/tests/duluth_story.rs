//! Initial local placement/partners/upstream flags are explicit fixtures.
//! The reveal, page ownership, signal and Continue use production inputs only.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty, visible_field_dialog_pages,
};

const ROOM: &str = "MnDuluthHarborWorkshop";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const READ: &str = "MN_HARBOR_RECEIVERText";

fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    fixture_at(ROOM, (10, 5), [true; 8], true, false, false)
}

pub(super) fn fixture_at(
    map: &str,
    at: (i16, i16),
    badges: [bool; 8],
    depot: bool,
    initial_signal: bool,
    injured: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_DULUTH_STORY_TEST_PACK").expect("published Duluth story pack"),
    )
    .canonicalize()
    .expect("external pack");
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: map.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial local position");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("initial name");
    let state = c.shell.shell.session_mut().state_mut();
    let mut mon = runtime
        .data()
        .create_pokemon("MEW", 45, Dv::default())
        .expect("legal partner");
    if injured {
        mon.hp = mon.max_hp.saturating_sub(15).max(1);
        mon.status = Some("PARALYSIS".into());
        for movement in &mut mon.moves {
            movement.current_pp = 0;
        }
    }
    assert!(state.storage.party.add_pokemon(mon));
    state.sync_party_from_storage();
    state.money = 10000;
    state.badges.johto = badges;
    for (index, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, badges[index])
            .expect("initial native badge flag");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_DEPOT_CLEARED", depot)
        .expect("explicit upstream handover");
    state
        .flags
        .set_event_flag(SIGNAL, initial_signal)
        .expect("explicit upstream signal fixture");
    for flag in [
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("explicit upstream custody fixture");
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c)
}

pub(super) fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production input");
    c.wait_frames(128).expect("complete visible page reveal");
    assert_eq!(c.shell.last_error, None);
}

fn signal(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(SIGNAL)
        .expect("signal event")
}

pub(super) fn talk(c: &mut VisibleShellController) {
    let tile = c.snapshot().expect("before facing").overworld.tile;
    press(c, GameButton::Right);
    assert_eq!(c.snapshot().expect("actor collision").overworld.tile, tile);
    press(c, GameButton::A);
}

pub(super) fn finish(c: &mut VisibleShellController) {
    for _ in 0..64 {
        let s = c.snapshot().expect("native scene");
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("harbor dialogue did not finish");
}

pub(super) fn save_path(stage: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save nonce")
        .as_nanos();
    let dir = std::env::var_os("CRYSTAL_DULUTH_STORY_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).expect("review save directory");
    dir.join(format!(
        "ms10-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ))
}

#[test]
#[ignore = "requires published external Duluth story pack"]
fn duluth_story_controller_names_kettlurk_requires_every_page_and_retains_only_signal() {
    let (root, runtime, mut c) = fixture();
    let before = c.shell.shell.session().state().clone();
    let ready = save_path("ready");
    c.save(&ready).expect("review save before reveal");
    talk(&mut c);
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("native text source");
    let pages = visible_field_dialog_pages(&raw, &c.shell).expect("native pages");
    assert_eq!(pages.len(), 10);
    assert!(pages.iter().any(|p| p.contains("KETTLURK")));
    assert!(pages.iter().any(|p| p.contains("BOUNDARY WATERS")));
    assert!(pages.iter().any(|p| p.contains("ICE wants it gone.")));
    assert!(pages.iter().any(|p| p.contains("drainage plan")));
    assert!(pages.iter().any(|p| p.contains("ICE kidnaps people")));
    assert!(pages.iter().any(|p| p.contains("after the warning.")));
    for (index, expected) in pages.iter().enumerate() {
        assert!(!signal(&c), "signal before acknowledging page {index}");
        let s = c.snapshot().expect("visible reading");
        let t = s.ui.text.as_ref().expect("reading owns text");
        assert_eq!(t.label, READ);
        assert_eq!(t.asm_text.as_deref(), Some(expected.as_str()));
        let reveal = c
            .shell
            .field_text_reveal
            .as_ref()
            .expect("production page cursor");
        assert_eq!(reveal.page_index, index);
        let at = s.overworld.tile;
        press(&mut c, GameButton::Down);
        assert_eq!(c.snapshot().expect("text ownership").overworld.tile, at);
        assert!(!signal(&c), "movement cannot acknowledge the reading");
        press(&mut c, GameButton::A);
    }
    assert!(signal(&c));
    finish(&mut c);
    let state = c.shell.shell.session().state();
    let mut flags = before.flags.clone();
    flags.set_event_flag(SIGNAL, true).expect("only result");
    assert_eq!(state.flags, flags);
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.bag, before.bag);
    assert_eq!(state.money, before.money);
    assert_eq!(state.storage, before.storage);
    let cleared = save_path("signal");
    c.save(&cleared).expect("save actual acknowledged signal");
    let mut continued =
        VisibleShellController::load_save(root.clone(), runtime.clone(), cleared.clone(), None)
            .expect("production Continue");
    assert!(signal(&continued));
    talk(&mut continued);
    assert_eq!(
        continued
            .snapshot()
            .expect("repeat")
            .ui
            .text
            .as_ref()
            .expect("repeat text")
            .label,
        "MN_HARBOR_RECEIVERDoneText"
    );
    finish(&mut continued);
    assert!(signal(&continued));
    let mut before_reveal = VisibleShellController::load_save(root, runtime, ready.clone(), None)
        .expect("Continue before reveal");
    assert!(!signal(&before_reveal));
    talk(&mut before_reveal);
    assert_eq!(
        before_reveal
            .snapshot()
            .expect("retry reading")
            .ui
            .text
            .as_ref()
            .expect("reading")
            .label,
        READ
    );
    if std::env::var_os("CRYSTAL_DULUTH_STORY_SAVE_DIR").is_none() {
        for path in [ready, cleared] {
            std::fs::remove_file(path).expect("remove only own temporary save");
        }
    }
}
