//! Three earned New Game journeys and one explicitly seeded capacity regression.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::bag::{BALL_POCKET_CAPACITY, MAX_ITEM_STACK};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::indoor_walk;

const HOME: &str = "EVENT_MN_HOME_INTRO";
const STARTER: &str = "EVENT_MN_STARTER_RECEIVED";
const PARCEL: &str = "EVENT_MN_R01_PARCEL_RECEIVED";
const BALLS: &str = "EVENT_MN_WINONA_BALLS_RECEIVED";

pub(super) fn load() -> (AssetRoot, CrystalRuntime) {
    static INPUT: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    INPUT
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_WINONA_TEST_PACK").expect("written Winona pack"),
            )
            .canonicalize()
            .expect("external pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified written pack"),
            )
            .expect("runtime");
            (root, runtime)
        })
        .clone()
}
pub(super) fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("production input");
    // The production controller already settles authored presentation to its
    // next input boundary, exactly as the stdio MCP client does.
    c.wait_frames(1).expect("released joypad frame");
    assert_eq!(c.shell.last_error, None);
}
pub(super) fn at(c: &mut VisibleShellController, map: &str, tile: (i16, i16)) {
    let s = c.snapshot().expect("position");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), tile);
}
fn flag(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("event")
}
fn count(c: &VisibleShellController) -> usize {
    c.shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .count()
}
pub(super) fn finish(c: &mut VisibleShellController) -> usize {
    let mut pages = 0;
    for _ in 0..128 {
        c.wait_frames(1).expect("settle the shared native printer");
        let s = c.snapshot().expect("dialogue");
        if s.ui
            .text
            .as_ref()
            .is_some_and(|t| t.label == "MnNSEReedDismissText")
        {
            super::northstar_story_events::observe_case(c);
        }
        if let Some(crystal_runtime::RuntimeBattleSnapshot {
            kind: crystal_runtime::RuntimeBattleKind::Trainer { ref trainer_id, .. },
            ..
        }) = s.battle
        {
            if (trainer_id.starts_with("MN_REED_") && trainer_id != "MN_REED_WINONA")
                || trainer_id.starts_with("MN_ICE_HQ_")
                || trainer_id.starts_with("MN_ICE_ARC_")
                || trainer_id.starts_with("MN_LODE_ARC_")
            {
                let (_, runtime) = load();
                let flag = format!("EVENT_{trainer_id}_DEFEATED");
                assert!(super::rochester_earned::try_battle(
                    c, &runtime, trainer_id, &flag
                ));
                continue;
            }
        }
        if s.ui.pending_yes_no.is_some()
            && s.ui
                .text
                .as_ref()
                .is_some_and(|t| t.label.starts_with("MnArc"))
        {
            let (_, runtime) = load();
            super::northstar_rival_arc::settle_walk_scene(c, &runtime);
            continue;
        }
        if s.battle.is_some() {
            // The event-driven story explicitly adds this early rival fight.
            // Every other battle remains unexpected in ordinary dialogue.
            super::northstar_story_events::play_reed(c);
            continue;
        }
        assert!(
            c.shell.pending_name_choice.is_none(),
            "nickname needs an explicit answer"
        );
        assert!(
            s.ui.pending_yes_no.is_none(),
            "choice needs an explicit answer"
        );
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
        {
            return pages;
        }
        if s.ui.text.is_some() {
            pages += 1;
        }
        press(c, GameButton::A);
    }
    panic!("opening dialogue did not release input");
}
pub(super) fn travel(c: &mut VisibleShellController, b: GameButton, map: &str, tile: (i16, i16)) {
    for _ in 0..12 {
        let s = c.snapshot().expect("travel");
        if s.overworld.map_name == map && (s.overworld.tile.x, s.overworld.tile.y) == tile {
            return;
        }
        press(c, b);
    }
    let actual = c.snapshot().expect("failed travel location").overworld;
    panic!(
        "did not reach {map} {tile:?}; actual {} {:?}",
        actual.map_name, actual.tile
    );
}
fn prompt(c: &mut VisibleShellController) {
    for _ in 0..32 {
        c.wait_frames(128).expect("question");
        if c.snapshot().expect("question").ui.pending_yes_no.is_some() {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("starter yes/no did not appear");
}
fn answer(c: &mut VisibleShellController, yes: bool) {
    for _ in 0..32 {
        if c.shell.pending_name_choice.is_some() || c.shell.pending_name_input.is_some() {
            return;
        }
        if c.snapshot().expect("question").ui.pending_yes_no.is_none() {
            return;
        }
        if c.shell
            .yes_no_cursor
            .as_ref()
            .expect("native choice")
            .option_index
            != usize::from(!yes)
        {
            press(
                c,
                if yes {
                    GameButton::Up
                } else {
                    GameButton::Down
                },
            );
        }
        press(c, GameButton::A);
    }
    panic!("starter choice did not resolve");
}
pub(super) fn continued(
    mut c: VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    continue_in_place(&mut c, root, r, stage);
    c
}

pub(super) fn continue_in_place(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    stage: &str,
) {
    let dir = std::env::var_os("CRYSTAL_WINONA_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).expect("save folder");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let stage = if std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some() {
        format!("boosted-review-{}", stage.replace("-earned-", "-"))
    } else {
        stage.to_owned()
    };
    let path = dir.join(format!(
        "winona-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    let before = c.snapshot().expect("before save");
    let state = c.shell.shell.session().state().clone();
    c.save(&path).expect("real save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), r.clone(), path, None).expect("Continue");
    let after = loaded.snapshot().expect("after save");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(state.badges, loaded.shell.shell.session().state().badges);
    assert_eq!(state.money, loaded.shell.shell.session().state().money);
    for name in [HOME, STARTER, PARCEL, BALLS] {
        assert_eq!(
            state.flags.is_event_flag_set(name).expect("old flag"),
            flag(&loaded, name)
        );
    }
    *c = loaded;
}

fn declined_starter(
    index: usize,
    species: &str,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = VisibleShellController::new_game(root.clone(), r.clone(), "CHRIS", None)
        .expect("production New Game");
    c.set_runtime_journal_enabled(false);
    // No position, party, badge, flag, item, money or RNG writes in this journey.
    at(&mut c, "MnWinonaHome2F", (3, 3));
    assert_eq!(count(&c), 0);
    assert!(!flag(&c, HOME) && !flag(&c, STARTER) && !flag(&c, PARCEL) && !flag(&c, BALLS));
    c = continued(c, &root, &r, "bedroom");
    indoor_walk(&mut c, &r, (7, 1));
    travel(&mut c, GameButton::Up, "MnWinonaHome1F", (9, 0));
    assert!(c.snapshot().expect("guardian").ui.text_window_open);
    assert!(!flag(&c, HOME));
    press(&mut c, GameButton::Down);
    at(&mut c, "MnWinonaHome1F", (9, 0));
    assert!(finish(&mut c) >= 3, "all guardian pages must be read");
    assert!(flag(&c, HOME));
    travel(&mut c, GameButton::Down, "MnWinonaHome1F", (9, 1));
    c = continued(c, &root, &r, "home-intro");
    indoor_walk(&mut c, &r, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (18, 20));
    travel(&mut c, GameButton::Up, "MnWinonaFieldStation", (4, 10));
    let x = 6 + i16::try_from(index).expect("starter index");
    indoor_walk(&mut c, &r, (x, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    prompt(&mut c);
    answer(&mut c, false);
    finish(&mut c);
    assert_eq!(count(&c), 0);
    assert!(!flag(&c, STARTER) && !flag(&c, PARCEL) && !flag(&c, BALLS));
    c = continued(c, &root, &r, &format!("{species}-declined"));
    (root, r, c)
}

pub(super) fn opening(
    index: usize,
    species: &str,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    if load().1.data().maps["MnWinonaFieldStation"]
        .scripts
        .contains_key("MnOverhaulLabEntry")
    {
        return super::dialogue_overhaul::opening(index);
    }
    let (root, r, mut c) = declined_starter(index, species);
    let money = c.shell.shell.session().state().money;
    let x = 6 + i16::try_from(index).expect("starter index");
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    prompt(&mut c);
    answer(&mut c, true);
    for _ in 0..64 {
        if c.shell.pending_name_choice.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert_eq!(
        count(&c),
        1,
        "real givepoke must run before its nickname prompt"
    );
    assert!(
        c.shell.pending_gift_pokemon_nickname.is_some(),
        "settling must preserve the player's nickname question"
    );
    assert!(
        c.snapshot()
            .expect("nickname projection")
            .ui
            .pending_yes_no
            .is_none(),
        "the preceding source confirmation must not cover the nickname owner"
    );
    assert!(!flag(&c, STARTER));
    for _ in 0..8 {
        if c.shell
            .pending_name_choice
            .as_ref()
            .expect("nickname")
            .nickname_pages
            .len()
            <= 1
        {
            break;
        }
        press(&mut c, GameButton::A);
        assert!(!flag(&c, STARTER));
    }
    assert_eq!(
        c.shell
            .pending_name_choice
            .as_ref()
            .expect("nickname")
            .nickname_pages
            .len(),
        1
    );
    if index == 1 {
        // Use the actual naming grid, including its two-press Start/END path.
        press(&mut c, GameButton::A);
        assert!(c.shell.pending_name_input.is_some());
        assert!(!flag(&c, STARTER));
        press(&mut c, GameButton::A);
        press(&mut c, GameButton::Start);
        press(&mut c, GameButton::Start);
        assert_eq!(
            c.shell.shell.session().state().storage.party.pokemon[0]
                .as_ref()
                .expect("named starter")
                .nickname,
            "A"
        );
    } else {
        press(&mut c, GameButton::Down);
        assert_eq!(
            c.shell
                .pending_name_choice
                .as_ref()
                .expect("nickname")
                .selected,
            1
        );
        press(&mut c, GameButton::A);
    }
    assert!(flag(&c, STARTER));
    assert!(!flag(&c, PARCEL));
    let p = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("real starter");
    let script = ["MN_W_CHIKORITA", "MN_W_CYNDAQUIL", "MN_W_TOTODILE"][index];
    let expected = r.data().maps["MnWinonaFieldStation"].scripts[script]
        .as_array()
        .expect("authored starter program")
        .iter()
        .find(|row| row["command"] == "givepoke")
        .and_then(|row| row["args"].get(0))
        .and_then(serde_json::Value::as_str)
        .expect("actual published starter species");
    assert_eq!(p.species.id, expected);
    assert_eq!(p.level, 5);
    assert!(!p.moves.is_empty());
    press(&mut c, GameButton::Down);
    at(&mut c, "MnWinonaFieldStation", (x, 4));
    finish(&mut c);
    assert!(flag(&c, PARCEL) && flag(&c, BALLS));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["POKE_BALL"]),
        5
    );
    c = continued(c, &root, &r, &format!("{species}-received"));
    let before = c.snapshot().expect("earned partner and balls");
    let repeat_money = c.shell.shell.session().state().money;
    let prize = r
        .data()
        .trainers
        .trainers
        .get("MN_REED_WINONA")
        .map_or(0, |t| {
            t.base_reward * 4 * u32::from(t.party.last().expect("rival party").level)
        });
    assert_eq!(repeat_money, money + prize);
    for other in 0..3 {
        if other == index {
            continue;
        }
        indoor_walk(&mut c, &r, (6 + i16::try_from(other).expect("ball"), 4));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        finish(&mut c);
    }
    assert_eq!(count(&c), 1);
    let after = c.snapshot().expect("repeat resources");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(c.shell.shell.session().state().money, repeat_money);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .badges
            .johto
            .iter()
            .any(|b| *b)
    );
    for name in [
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
        "EVENT_MN_ICE_ENFORCER_DEFEATED",
        "EVENT_MN_DISPATCH_NOTE",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!flag(&c, name), "opening supplied later progress: {name}");
    }
    indoor_walk(&mut c, &r, (4, 10));
    travel(&mut c, GameButton::Down, "MnWinona", (18, 20));
    c = continued(c, &root, &r, &format!("{species}-town"));
    indoor_walk(&mut c, &r, (7, 8));
    travel(&mut c, GameButton::Up, "MnWinonaHome1F", (7, 6));
    assert!(!c.snapshot().expect("home repeat").ui.text_window_open);
    (root, r, c)
}

#[test]
#[ignore = "requires the written external Winona opening pack"]
fn winona_new_game_chikorita_decline_retry_real_gift_bundle_repeat_and_continue() {
    opening(0, "CHIKORITA");
}
#[test]
#[ignore = "requires the written external Winona opening pack"]
fn winona_new_game_cyndaquil_decline_retry_real_gift_bundle_repeat_and_continue() {
    opening(1, "CYNDAQUIL");
}
#[test]
#[ignore = "requires the written external Winona opening pack"]
fn winona_new_game_totodile_decline_retry_real_gift_bundle_repeat_and_continue() {
    opening(2, "TOTODILE");
}

#[test]
#[ignore = "requires the written external Winona opening pack"]
fn winona_full_ball_pocket_preserves_reward_then_retries_once_and_continues() {
    let (root, r, mut c) = declined_starter(0, "CHIKORITA");
    // Explicit capacity fixture after earning the home scene and declining.
    // The three opening journeys above contain no inventory/state writes.
    let ball = &r.data().items["POKE_BALL"];
    for _ in 0..BALL_POCKET_CAPACITY {
        assert!(
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_item(ball, MAX_ITEM_STACK)
                .expect("full native ball pocket fixture")
        );
    }
    let full_quantity =
        u16::try_from(BALL_POCKET_CAPACITY).expect("native capacity") * MAX_ITEM_STACK;
    let full = c.shell.shell.session().state().bag.clone();
    mark_runtime_snapshot_dirty(&mut c.shell);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    prompt(&mut c);
    answer(&mut c, true);
    for _ in 0..8 {
        let choice = c
            .shell
            .pending_name_choice
            .as_ref()
            .expect("nickname question");
        if choice.nickname_pages.len() <= 1 {
            break;
        }
        press(&mut c, GameButton::A);
    }
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(flag(&c, STARTER) && flag(&c, PARCEL));
    assert!(!flag(&c, BALLS));
    assert_eq!(c.shell.shell.session().state().bag, full);
    c = continued(c, &root, &r, "full-stack-refused");
    // Free capacity in the fixture; the actual retry is another interaction.
    assert!(
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .remove_item(ball, 5)
            .expect("free five slots in stack")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    indoor_walk(&mut c, &r, (7, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(flag(&c, BALLS));
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(ball),
        full_quantity
    );
    assert_eq!(count(&c), 1);
    c = continued(c, &root, &r, "full-stack-retried");
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(ball),
        full_quantity
    );
    assert_eq!(count(&c), 1);
}
