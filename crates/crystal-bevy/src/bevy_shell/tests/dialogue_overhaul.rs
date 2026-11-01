//! Earned staged opening and text-control timing regressions.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::systems::script_text::{ScriptTextBody, ScriptTextBodyCommand};
use crystal_runtime::CrystalRuntime;

use super::super::field_text_program::{self, PageTransition, TextEffect, TextTiming};
use super::super::{VisibleShellController, field_text_timing};
use super::authored_route_fixture::indoor_walk;
use super::winona::{at, continue_in_place, finish, load, press, travel};

fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("story flag")
}

fn capture_checkpoint(c: &mut VisibleShellController, name: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_WINONA_SAVE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("capture directory");
        c.save(directory.join(format!("{name}.crystalsave")))
            .expect("actual scene checkpoint");
    }
}

fn enter(c: &mut VisibleShellController, button: GameButton, map: &str) {
    for _ in 0..16 {
        if c.snapshot().expect("arrival").overworld.map_name == map {
            return;
        }
        press(c, button);
    }
    panic!("did not enter {map}");
}

fn answer(c: &mut VisibleShellController, yes: bool) {
    for _ in 0..48 {
        if c.snapshot().expect("question").ui.pending_yes_no.is_some() {
            let cursor = c
                .shell
                .yes_no_cursor
                .as_ref()
                .expect("native cursor")
                .option_index;
            if cursor != usize::from(!yes) {
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
            return;
        }
        press(c, GameButton::A);
    }
    panic!("no choice after starter picture");
}

pub(super) fn opening(index: usize) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = VisibleShellController::new_game(root.clone(), r.clone(), "CHRIS", None)
        .expect("production New Game");
    c.set_runtime_journal_enabled(false);
    indoor_walk(&mut c, &r, (7, 1));
    // The stair arrival remains quiet; Mom intercepts at the door.
    enter(&mut c, GameButton::Up, "MnWinonaHome1F");
    at(&mut c, "MnWinonaHome1F", (9, 1));
    assert!(c.snapshot().expect("quiet arrival").ui.text.is_none());
    indoor_walk(&mut c, &r, (7, 6));
    at(&mut c, "MnWinonaHome1F", (7, 6));
    let speaking = c.snapshot().expect("Mom has approached before speaking");
    let mom = speaking.visible_object_runtime_tiles["MN_W_GUARDIAN"];
    assert_eq!((mom.x, mom.y), (7, 5));
    assert_eq!(
        speaking.visible_object_facings["MN_W_GUARDIAN"],
        crystal_core::world::map::Direction::Down
    );
    c.wait_frames(128).expect("Mom's approach animation");
    let snapshot = c.snapshot().expect("Mom approaches");
    let mom = snapshot
        .visible_object_runtime_tiles
        .get("MN_W_GUARDIAN")
        .expect("visible Mom");
    assert_eq!((mom.x, mom.y), (7, 5));
    assert!(!event(&c, "EVENT_MN_HOME_INTRO"));
    press(&mut c, GameButton::Down);
    at(&mut c, "MnWinonaHome1F", (7, 6));
    continue_in_place(&mut c, &root, &r, "overhaul-mom-speaking");
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS_HOME",
        "MnWHomeIntroText",
        "EVENT_MN_HOME_INTRO",
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_POKEGEAR")
            .expect("Pokégear receipt flag")
    );
    capture_checkpoint(&mut c, &format!("mom-gear-receipt-{index}"));
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS_HOME",
        "MnWHomeGearReceivedText",
        "EVENT_MN_HOME_INTRO",
    );
    finish(&mut c);
    let mom = c
        .snapshot()
        .expect("Mom returns")
        .visible_object_runtime_tiles
        .remove("MN_W_GUARDIAN")
        .expect("Mom");
    assert_eq!((mom.x, mom.y), (7, 4));
    assert!(event(&c, "EVENT_MN_HOME_INTRO"));
    indoor_walk(&mut c, &r, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (18, 20));
    enter(&mut c, GameButton::Up, "MnWinonaFieldStation");
    c.wait_frames(256).expect("walk along the lab aisle");
    at(&mut c, "MnWinonaFieldStation", (4, 2));
    assert_eq!(
        c.snapshot()
            .expect("professor")
            .ui
            .text
            .expect("introduction")
            .label,
        "MnOverhaulTamarackIntroText"
    );
    assert!(!event(&c, "EVENT_MN_TAMARACK_INTRO"));
    capture_checkpoint(&mut c, &format!("tamarack-introduction-{index}"));
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS00",
        "MnOverhaulTamarackIntroText",
        "EVENT_MN_TAMARACK_INTRO",
    );
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_TAMARACK_INTRO"));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.snapshot()
            .expect("brief repeat")
            .ui
            .text
            .expect("reminder")
            .asm_text,
        Some("TAMARACK: No rush.\nTake your time.".into())
    );
    capture_checkpoint(&mut c, &format!("tamarack-choice-reminder-{index}"));
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS00",
        "MnWChooseText",
        "EVENT_MN_STARTER_RECEIVED",
    );
    let x = 6 + i16::try_from(index).expect("starter index");
    indoor_walk(&mut c, &r, (x, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    answer(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_STARTER_RECEIVED"));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    answer(&mut c, true);
    for _ in 0..32 {
        if c.shell
            .pending_name_choice
            .as_ref()
            .is_some_and(|choice| choice.nickname_pages.len() == 1)
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(!event(&c, "EVENT_MN_STARTER_RECEIVED"));
    if index == 1 {
        press(&mut c, GameButton::A);
        assert!(c.shell.pending_name_input.is_some());
        assert!(!event(&c, "EVENT_MN_STARTER_RECEIVED"));
        press(&mut c, GameButton::A);
        press(&mut c, GameButton::Start);
        press(&mut c, GameButton::Start);
        assert_eq!(
            c.snapshot().expect("actually named starter").party.slots[0]
                .pokemon
                .nickname,
            "A"
        );
        assert!(!event(&c, "EVENT_MN_STARTER_RECEIVED"));
    } else {
        press(&mut c, GameButton::Down);
        press(&mut c, GameButton::A);
    }
    // The received-partner page precedes completion and automatic repositioning.
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS00",
        "MnOverhaulStarterReceivedText",
        "EVENT_MN_STARTER_RECEIVED",
    );
    at(&mut c, "MnWinonaFieldStation", (4, 2));
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS00",
        "MnWReadingsText",
        "EVENT_MN_R01_PARCEL_RECEIVED",
    );
    super::northstar_dialogue::acknowledge(
        &mut c,
        "MS00",
        "MnWPokedexReceivedText",
        "EVENT_MN_REED_FIRST_MET",
    );
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_STARTER_RECEIVED"));
    assert!(event(&c, "EVENT_MN_R01_PARCEL_RECEIVED"));
    assert!(!event(&c, "EVENT_MN_WINONA_BALLS_RECEIVED"));
    at(&mut c, "MnWinonaFieldStation", (4, 2));
    let before = c.snapshot().expect("partner");
    assert_eq!(before.party.slots.len(), 1);
    for other in 0..3 {
        if other == index {
            continue;
        }
        indoor_walk(&mut c, &r, (6 + other as i16, 4));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        assert_eq!(
            c.snapshot()
                .expect("remaining partner")
                .ui
                .text
                .expect("description")
                .label,
            format!("MnOverhaulOtherStarter{other}Text")
        );
        finish(&mut c);
    }
    assert_eq!(before.party, c.snapshot().expect("no extra gifts").party);
    indoor_walk(&mut c, &r, (4, 8));
    press(&mut c, GameButton::Down);
    at(&mut c, "MnWinonaFieldStation", (4, 9));
    assert!(
        !event(&c, "EVENT_MN_WINONA_BALLS_RECEIVED")
            || c.snapshot().expect("aide").ui.text_window_open
    );
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_WINONA_BALLS_RECEIVED"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["POKE_BALL"]),
        5
    );
    if let Some(directory) = std::env::var_os("CRYSTAL_WINONA_SAVE_DIR") {
        std::fs::write(
            std::path::PathBuf::from(directory).join(format!("opening-{index}.txt")),
            c.shell
                .dialogue_log_events
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .expect("replayed dialogue transcript");
    }
    continue_in_place(&mut c, &root, &r, "overhaul-opening-complete");
    indoor_walk(&mut c, &r, (4, 10));
    travel(&mut c, GameButton::Down, "MnWinona", (18, 20));
    indoor_walk(&mut c, &r, (7, 8));
    travel(&mut c, GameButton::Up, "MnWinonaHome1F", (7, 6));
    (root, r, c)
}

pub(super) fn delivery(index: usize) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = opening(index);
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Left, "MnR01C00R00", (63, 24));
    super::rochester::trail(&mut c, &r, (0, 24));
    travel(&mut c, GameButton::Left, "MnRochester", (39, 18));
    indoor_walk(&mut c, &r, (15, 8));
    travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
    indoor_walk(&mut c, &r, (11, 9));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(!event(&c, "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"));
    let pages = super::northstar_dialogue::canonical_pages("MS01", "MnRCDeliveryText");
    for page in pages {
        assert!(!event(&c, "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"));
        assert_eq!(
            c.snapshot()
                .expect("Ivy page")
                .ui
                .text
                .and_then(|t| t.asm_text),
            Some(page)
        );
        press(&mut c, GameButton::A);
    }
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"));
    continue_in_place(&mut c, &root, &r, "overhaul-originals-delivered");
    (root, r, c)
}

#[test]
#[ignore = "requires revised external pack via CRYSTAL_WINONA_TEST_PACK"]
fn winona_wardens_explain_partner_requirement_and_remain_after_starter() {
    let (root, r) = load();
    let mut c = VisibleShellController::new_game(root, r.clone(), "CHRIS", None)
        .expect("production New Game");
    c.set_runtime_journal_enabled(false);
    indoor_walk(&mut c, &r, (7, 1));
    enter(&mut c, GameButton::Up, "MnWinonaHome1F");
    indoor_walk(&mut c, &r, (7, 6));
    finish(&mut c);
    indoor_walk(&mut c, &r, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (4, 18));
    let before = c.snapshot().expect("wardens before the starter");
    assert!(!event(&c, "EVENT_MN_STARTER_RECEIVED"));
    for id in ["MN_W_TRAIL_WARDEN_NORTH", "MN_W_TRAIL_WARDEN_SOUTH"] {
        assert!(before.visible_object_runtime_tiles.contains_key(id), "{id}");
    }
    for _ in 0..4 {
        if c.snapshot().expect("warden check").ui.text.is_some() {
            break;
        }
        press(&mut c, GameButton::Left);
    }
    for (index, expected) in [
        "Hold up! You need\na POKEMON partner",
        "for this trail.\nAsk TAMARACK at",
        "the field station\nby the river.",
    ]
    .into_iter()
    .enumerate()
    {
        let page = c
            .snapshot()
            .expect("warden warning")
            .ui
            .text
            .expect("warning page");
        assert_eq!(page.label, "MnWTrailWardenText");
        assert_eq!(page.asm_text.as_deref(), Some(expected));
        capture_checkpoint(&mut c, &format!("warden-before-{index}"));
        press(&mut c, GameButton::A);
    }
    finish(&mut c);
    at(&mut c, "MnWinona", (4, 18));

    let (root, r, mut c) = opening(0);
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    continue_in_place(&mut c, &root, &r, "warden-after-starter");
    indoor_walk(&mut c, &r, (5, 18));
    let after = c.snapshot().expect("wardens after receiving a starter");
    assert!(event(&c, "EVENT_MN_STARTER_RECEIVED"));
    for (id, position) in [
        ("MN_W_TRAIL_WARDEN_NORTH", (5, 17)),
        ("MN_W_TRAIL_WARDEN_SOUTH", (6, 17)),
    ] {
        let tile = after
            .visible_object_runtime_tiles
            .get(id)
            .expect("visible warden");
        assert_eq!((tile.x, tile.y), position);
    }
    for x in [5, 6] {
        indoor_walk(&mut c, &r, (x, 18));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        for (index, expected) in ["ROCHESTER is west.\nRest at home if", "your team tires."]
            .into_iter()
            .enumerate()
        {
            let page = c
                .snapshot()
                .expect("warden directions")
                .ui
                .text
                .expect("direction page");
            assert_eq!(page.label, "MnWTrailWardenClearedText");
            assert_eq!(page.asm_text.as_deref(), Some(expected));
            capture_checkpoint(&mut c, &format!("warden-after-{x}-{index}"));
            press(&mut c, GameButton::A);
        }
        finish(&mut c);
    }
    indoor_walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Left, "MnR01C00R00", (63, 24));
    press(&mut c, GameButton::Start);
    assert!(
        c.snapshot()
            .expect("Start after the trail")
            .ui
            .menu
            .is_some()
    );
}

#[test]
#[ignore = "requires revised external pack via CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_three_earned_starters_and_ivy_delivery() {
    for index in 0..3 {
        delivery(index);
    }
}

#[test]
#[ignore = "requires revised external pack via CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_v28_story_scenes_pokegear_wardens_cousin_and_rochester_reed() {
    let (root, r) = load();
    let mut c = VisibleShellController::new_game(root.clone(), r.clone(), "CHRIS", None)
        .expect("production New Game");
    c.set_runtime_journal_enabled(false);

    // 1. Check Home 2F cousin before starter (cousin is at (1, 3)):
    indoor_walk(&mut c, &r, (2, 3));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.snapshot()
            .expect("cousin greeting")
            .ui
            .text
            .expect("cousin text")
            .label,
        "MN_SMALL_INTERIOR_WINONA_COUSINInteriorText"
    );
    finish(&mut c);

    // 2. Step down stairs to Home 1F (9, 1): Mom walks to (9, 2) BEFORE opening dialogue,
    // then hands over Pokegear and returns to (7, 4).
    indoor_walk(&mut c, &r, (7, 1));
    enter(&mut c, GameButton::Up, "MnWinonaHome1F");
    at(&mut c, "MnWinonaHome1F", (9, 1));
    c.wait_frames(128).expect("Mom's approach animation");
    let snapshot = c.snapshot().expect("Mom approaches");
    let mom = snapshot
        .visible_object_runtime_tiles
        .get("MN_W_GUARDIAN")
        .expect("visible Mom");
    assert_eq!((mom.x, mom.y), (9, 2));
    assert_eq!(
        snapshot.ui.text.expect("Mom intro").label,
        "MnWHomeIntroText"
    );
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_HOME_INTRO"));
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_POKEGEAR")
            .expect("ENGINE_POKEGEAR")
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_MAP_CARD")
            .expect("ENGINE_MAP_CARD")
    );

    // 3. Before starter: Winona trail wardens block Driftless Trail at (2, 18) and (2, 19)
    // and explain that wild Pokemon live in the tall grass.
    indoor_walk(&mut c, &r, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (4, 18));
    for _ in 0..4 {
        if c.snapshot().expect("warden check").ui.text.is_some() {
            break;
        }
        press(&mut c, GameButton::Left);
    }
    assert_eq!(
        c.snapshot()
            .expect("trail warden warning")
            .ui
            .text
            .expect("warden text")
            .label,
        "MnWTrailWardenText"
    );
    finish(&mut c);
    at(&mut c, "MnWinona", (4, 18));
    drop(c);

    // 4. Complete delivery(0) which verifies Field Station intro, starter, Pokedex,
    // Reed's approach & post-battle position, Aide's 5 Poke Balls, open Winona trail,
    // and Ivy delivery in Rochester.
    let (_root, r, mut c) = delivery(0);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_POKEDEX")
            .expect("ENGINE_POKEDEX")
    );

    // 5. In MnRochesterPokecenter1F after delivering the parcel: step onto (11, 12)
    // and verify Reed walks from (9, 10) to (11, 11) (adjacent to player at (11, 12))
    // before opening MnArcRCPathsText.
    indoor_walk(&mut c, &r, (11, 11));
    for _ in 0..4 {
        if c.snapshot().expect("Reed clinic check").ui.text.is_some() || c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    c.wait_frames(128).expect("Reed walks to (11, 11)");
    let snapshot = c.snapshot().expect("Reed at clinic");
    let reed = snapshot
        .visible_object_runtime_tiles
        .get("MN_ARC_RC_REED")
        .expect("visible Reed in clinic");
    assert_eq!((reed.x, reed.y), (11, 11));
    assert_eq!(
        snapshot.ui.text.expect("Reed clinic offer").label,
        "MnArcRCPathsText"
    );
}

#[test]
#[ignore = "requires revised pack and an earned campaign; no fixture progression or level boosts"]
fn overhaul_earned_campaign_through_hall_of_fame() {
    assert!(
        std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR").is_some(),
        "set CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR for actual in-scene review captures"
    );
    assert!(
        std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_none(),
        "earned campaign cannot use review boosts"
    );
    for variable in [
        "CRYSTAL_NORTHSTAR_EARNED_SAVE",
        "CRYSTAL_NORTHSTAR_PRAIRIE_EXIT_SAVE",
        "CRYSTAL_NORTHSTAR_CURRENT_SAVE",
        "CRYSTAL_NORTHSTAR_CURRENT_EXIT_SAVE",
        "CRYSTAL_NORTHSTAR_COURIER_SAVE",
        "CRYSTAL_NORTHSTAR_ICE_SAVE",
        "CRYSTAL_NORTHSTAR_RELAY_SAVE",
        "CRYSTAL_NORTHSTAR_WIND_SAVE",
        "CRYSTAL_NORTHSTAR_LAKE_SAVE",
        "CRYSTAL_NORTHSTAR_PINE_SAVE",
        "CRYSTAL_NORTHSTAR_IRON_SAVE",
        "CRYSTAL_NORTHSTAR_FINALE_SAVE",
        "CRYSTAL_NORTHSTAR_LEAGUE_SAVE",
    ] {
        assert!(
            std::env::var_os(variable).is_none(),
            "earned campaign must start at New Game: {variable}"
        );
    }
    super::minnesota_gyms::northstar_arc_unboosted_new_game_through_legendary_release_and_hall_of_fame();
}

#[test]
#[ignore = "requires the same candidate's actual earned chapter save; no fixture progression or boosts"]
fn overhaul_earned_continue_through_hall_of_fame() {
    earned_continue(true);
}

#[test]
#[ignore = "legacy V16 earned story checkpoint only; does not verify final V18 service acceptance"]
fn overhaul_earned_legacy_v16_continue_through_hall_of_fame() {
    let path = std::env::var_os("CRYSTAL_WINONA_TEST_PACK").expect("exact legacy pack path");
    let pack = crystal_assets::read_verified_compiled_game_pack(std::path::PathBuf::from(path))
        .expect("verified legacy pack");
    assert_eq!(
        pack.identity()
            .expect("legacy content identity")
            .content_hash,
        "c6feb99daa624a3772c5e76cfc8878b0760d4ce9b498b47eacf145b5156f1d22",
        "only the retained V16 story lineage may use this legacy course"
    );
    drop(pack);
    earned_continue(false);
}

fn earned_continue(verify_service_return: bool) {
    assert!(
        std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR").is_some(),
        "set CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR for actual in-scene review captures"
    );
    assert!(
        std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_none(),
        "earned Continue cannot use review boosts"
    );
    let checkpoints = [
        "CRYSTAL_NORTHSTAR_EARNED_SAVE",
        "CRYSTAL_NORTHSTAR_PRAIRIE_EXIT_SAVE",
        "CRYSTAL_NORTHSTAR_CURRENT_SAVE",
        "CRYSTAL_NORTHSTAR_CURRENT_EXIT_SAVE",
        "CRYSTAL_NORTHSTAR_COURIER_SAVE",
        "CRYSTAL_NORTHSTAR_ICE_SAVE",
        "CRYSTAL_NORTHSTAR_RELAY_SAVE",
        "CRYSTAL_NORTHSTAR_WIND_SAVE",
        "CRYSTAL_NORTHSTAR_LAKE_SAVE",
        "CRYSTAL_NORTHSTAR_PINE_SAVE",
        "CRYSTAL_NORTHSTAR_IRON_SAVE",
        "CRYSTAL_NORTHSTAR_FINALE_SAVE",
        "CRYSTAL_NORTHSTAR_LEAGUE_SAVE",
    ];
    assert_eq!(
        checkpoints
            .iter()
            .filter(|name| std::env::var_os(name).is_some())
            .count(),
        1,
        "Continue requires exactly one actual New Game campaign checkpoint"
    );
    // Native loaders retain the actual checkpoint's identity and prerequisites.
    // The pinned legacy V16 course is story evidence, never final V18 service acceptance.
    super::minnesota_gyms::earned_arc_through_hall_of_fame(verify_service_return);
}

#[test]
fn overhaul_text_program_retains_source_control_boundaries() {
    let body = ScriptTextBody {
        label: "TimingText".into(),
        commands: [
            ("text", "First"),
            ("text_pause", ""),
            ("text", " sentence."),
            ("sound_item", ""),
            ("para", "Next page."),
            ("line", "Second line."),
            ("cont", "New bottom line."),
            ("done", ""),
            ("text", "Unused bytes."),
        ]
        .into_iter()
        .enumerate()
        .map(|(command_index, (command, text))| ScriptTextBodyCommand {
            command: command.into(),
            args: if text.is_empty() {
                vec![]
            } else {
                vec![text.into()]
            },
            command_index,
        })
        .collect(),
    };
    let program = field_text_program::compile(&body, &BTreeMap::new(), "CHRIS", "REED", 0);
    assert_eq!(program.pages.len(), 3);
    assert_eq!(program.pages[0].text, "First sentence.");
    assert_eq!(program.pages[0].cues[0].after_chars, 5);
    assert_eq!(program.pages[0].cues[0].effect, TextEffect::Pause);
    assert_eq!(
        program.pages[0].cues[1].effect,
        TextEffect::Sound("SFX_ITEM")
    );
    assert_eq!(program.pages[1].transition, PageTransition::Paragraph);
    assert_eq!(program.pages[2].transition, PageTransition::Scroll);
    assert_eq!(program.pages[2].text, "Second line.\nNew bottom line.");
}

#[test]
fn overhaul_inline_chunks_preserve_spaces_and_sound_only_bodies() {
    let compile = |commands: &[(&str, &[&str])]| {
        field_text_program::compile(
            &ScriptTextBody {
                label: "ChunkText".into(),
                commands: commands
                    .iter()
                    .enumerate()
                    .map(|(command_index, (command, args))| ScriptTextBodyCommand {
                        command: (*command).into(),
                        args: args.iter().map(|arg| (*arg).into()).collect(),
                        command_index,
                    })
                    .collect(),
            },
            &BTreeMap::new(),
            "CHRIS",
            "REED",
            0,
        )
    };
    let program = compile(&[
        ("text", &["\"Take \""]),
        ("text_pause", &[]),
        ("text", &["\"your time.\""]),
        ("done", &[]),
    ]);
    assert_eq!(program.pages[0].text, "Take your time.");
    let sound = compile(&[("sound_item", &[]), ("done", &[])]);
    assert_eq!(sound.pages.len(), 1);
    assert!(sound.pages[0].text.is_empty());
    assert_eq!(sound.pages[0].cues[0].after_chars, 0);
    assert_eq!(sound.pages[0].cues[0].effect, TextEffect::Sound("SFX_ITEM"));
}

#[test]
#[ignore = "requires verified external pack via CRYSTAL_RENDER_TEST_PACK"]
fn overhaul_printer_waits_thirty_frames_at_pause_and_preserves_scroll_delays() {
    let mut shell = super::core_modular_title_shell_for_test();
    let mut timing = TextTiming::new(
        "test".into(),
        field_text_program::TextProgram::plain(vec!["First".into(), "Next".into()]),
        0,
    );
    timing.program.pages[0]
        .cues
        .push(field_text_program::TextCue {
            after_chars: 0,
            effect: TextEffect::Pause,
        });
    shell.field_text_timing = Some(timing);
    for _ in 0..30 {
        assert!(field_text_timing::advance(&mut shell, false).expect("pause frame"));
    }
    assert!(!field_text_timing::advance(&mut shell, false).expect("resume printing"));
    shell.field_text_timing.as_mut().expect("timing").enter(1);
    for _ in 0..20 {
        assert!(field_text_timing::advance(&mut shell, false).expect("paragraph frame"));
    }
    assert!(!field_text_timing::advance(&mut shell, false).expect("paragraph complete"));
    let timing = shell.field_text_timing.as_mut().expect("timing");
    timing.program.pages[1].transition = PageTransition::Scroll;
    timing.enter(1);
    for frame in 0..10 {
        assert_eq!(
            field_text_timing::scroll_offset(&shell),
            if frame < 5 { 1.0 } else { 0.0 }
        );
        assert!(field_text_timing::advance(&mut shell, false).expect("scroll frame"));
    }
    assert!(!field_text_timing::advance(&mut shell, false).expect("scroll complete"));
    let timing = shell.field_text_timing.as_mut().expect("timing");
    timing.enter(0);
    assert!(!field_text_timing::advance(&mut shell, true).expect("held A skips inline pause"));
    assert!(field_text_timing::complete(&shell));
}

#[test]
#[ignore = "requires verified external pack via CRYSTAL_RENDER_TEST_PACK"]
fn overhaul_text_confirmation_and_receipt_cues_reach_non_silent_playback() {
    use bevy::prelude::{App, Update};

    let mut shell = super::core_modular_title_shell_for_test();
    shell.pending_audio.clear();
    shell.pending_full_audio_reset = false;
    shell.pending_music_stop = false;
    let ids = ["SFX_READ_TEXT_2", "SFX_ITEM", "SFX_CAUGHT_MON"];
    let mut program = field_text_program::TextProgram::plain(vec![String::new()]);
    program.pages[0].cues = ids
        .iter()
        .map(|id| field_text_program::TextCue {
            after_chars: 0,
            effect: TextEffect::Sound(id),
        })
        .collect();
    shell.field_text_timing = Some(TextTiming::new(String::new(), program, 0));
    shell.field_text_reveal = Some(super::super::VisibleFieldTextReveal {
        text: String::new(),
        page_index: 0,
        visible_chars: 0,
        frames_until_next_char: 0,
    });
    let mut app = App::new();
    app.insert_resource(shell)
        .add_systems(Update, super::super::play_pending_audio);
    for (index, id) in ids.into_iter().enumerate() {
        {
            let mut shell = app
                .world_mut()
                .resource_mut::<super::super::BevyRuntimeShell>();
            assert!(
                field_text_timing::advance(&mut shell, false).expect("dispatch authored sound")
            );
            assert!(!field_text_timing::complete(&shell));
            assert_eq!(
                shell
                    .pending_audio
                    .last()
                    .expect("one ordered cue")
                    .audio_id,
                id
            );
            assert!(field_text_timing::advance(&mut shell, false).expect("wait for playback pump"));
        }
        for _ in 0..4 {
            app.update();
        }
        let shell = app.world().resource::<super::super::BevyRuntimeShell>();
        assert!(shell.pending_audio.is_empty());
        assert_eq!(shell.audio_source_cache.len(), index + 1);
        for pcm in shell.audio_source_cache.values() {
            assert!(
                pcm.bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .any(|s| i16::from_le_bytes([s[0], s[1]]).unsigned_abs() > 32),
                "cue must reach non-silent native PCM"
            );
        }
        // This App has no audio device or elapsed playback clock. Complete
        // the just-inspected transient before testing the next cue, matching
        // the production backend's completion notification after WaitSFX.
        let mut shell = app
            .world_mut()
            .resource_mut::<super::super::BevyRuntimeShell>();
        shell.transient_audio_playing = false;
        shell.active_transient_kind = None;
    }
    let mut shell = app
        .world_mut()
        .resource_mut::<super::super::BevyRuntimeShell>();
    assert!(!field_text_timing::advance(&mut shell, false).expect("all cues completed"));
    assert!(field_text_timing::complete(&shell));
}

#[test]
#[ignore = "requires the external core text pack and native art for the production 2D schedule"]
fn overhaul_held_acceleration_cannot_acknowledge_the_next_page() {
    use super::super::BevyRuntimeShell;
    use bevy::prelude::{ButtonInput, KeyCode};

    let mut shell = super::initialized_mail_reader_shell("FLOWER_MAIL");
    shell.field_notice = Some("First sentence.\n\nNext page.".into());
    shell.shell.session_mut().state_mut().options.text_speed = crystal_core::state::TextSpeed::Slow;
    super::super::mark_runtime_snapshot_dirty(&mut shell);
    let mut app = super::integrated_shell_test_app(shell);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyZ);
    for _ in 0..120 {
        app.update();
        // Clear only input edges; retain the actual held key.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    let reveal = shell.field_text_reveal.as_ref().expect("held printer");
    assert_eq!(reveal.page_index, 0);
    assert_eq!(reveal.visible_chars, "First sentence.".chars().count());
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset(KeyCode::KeyZ);
    app.update();
    super::press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let reveal = shell.field_text_reveal.as_ref().expect("new paragraph");
    assert_eq!(reveal.page_index, 1);
    assert_eq!(reveal.visible_chars, 0, "paragraph clears before its delay");
    assert!(!super::super::field_text_timing::complete(shell));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyZ);
    for _ in 0..120 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(
        shell
            .field_text_reveal
            .as_ref()
            .expect("last page")
            .page_index,
        1
    );
    assert!(
        shell.field_notice.is_some(),
        "held acceleration cannot dismiss the following page"
    );
}

#[test]
#[ignore = "declared five-badge/level-65 late-story fixture; supplementary to the earned New Game course"]
fn overhaul_declared_late_story_through_hall_of_fame() {
    let checkpoints = [
        "CRYSTAL_NORTHSTAR_LAKE_SAVE",
        "CRYSTAL_NORTHSTAR_PINE_SAVE",
        "CRYSTAL_NORTHSTAR_IRON_SAVE",
        "CRYSTAL_NORTHSTAR_FINALE_SAVE",
        "CRYSTAL_NORTHSTAR_LEAGUE_SAVE",
    ]
    .into_iter()
    .filter_map(std::env::var_os)
    .collect::<Vec<_>>();
    assert_eq!(
        checkpoints.len(),
        1,
        "one explicitly declared late-story checkpoint"
    );
    let path = std::path::PathBuf::from(&checkpoints[0]);
    assert!(path.to_string_lossy().contains("declared-late-story"));
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_none());
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR").is_some());
    super::minnesota_gyms::northstar_arc_unboosted_new_game_through_legendary_release_and_hall_of_fame();
}
