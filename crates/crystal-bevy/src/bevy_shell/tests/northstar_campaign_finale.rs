//! Continue the same carried party along the coast and through the relay finale.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at, continue_in_place, finish, load, press, travel};
use super::gym_eight_approach::care;
use super::north_shore_journey::road;
use super::north_shore_story::choice;
use super::northstar_campaign_iron::iron;
use super::northstar_campaign_review::boost;
use super::{VisibleShellController, event, interact};

const CAMP: &str = "MnBoundaryWaters";
const ROOM: &str = "MnBoundaryWatersSurveyShelter";
const CHIEF: &str = "EVENT_MN_SURVEY_CHIEF_DEFEATED";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";

pub(super) fn walk_to(c: &mut VisibleShellController, r: &CrystalRuntime, tile: (i16, i16)) {
    let map = c.snapshot().expect("local walking map").overworld.map_name;
    indoor_walk(c, r, tile);
    at(c, &map, tile);
}

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_FINALE_SAVE") else {
        return iron();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual same-pack harbor or Boundary checkpoint");
    c.set_runtime_journal_enabled(false);
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    for flag in [
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
    ] {
        assert!(event(&c, flag), "played prerequisite {flag}");
    }
    let s = c.snapshot().expect("settled northern checkpoint");
    assert!(s.battle.is_none() && s.ui.text.is_none() && s.ui.menu.is_none());
    assert!(matches!(
        s.overworld.map_name.as_str(),
        "MnDuluthHarborWorkshop" | "MnGrandMarais" | "MnBoundaryIntake" | CAMP | ROOM
    ));
    (root, r, c)
}

pub(in crate::bevy_shell::tests) fn walk_cedar(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: (i16, i16),
) {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    let map = &r.data().maps["MnMaraisCedarApproach"];
    let snapshot = c.snapshot().expect("current Cedar approach");
    let start = (snapshot.overworld.tile.x, snapshot.overworld.tile.y);
    let blocked = map
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(start, (start, GameButton::A))]);
    let mut queue = VecDeque::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == target {
            break;
        }
        for (next, button) in [
            ((x - 1, y), GameButton::Left),
            ((x + 1, y), GameButton::Right),
            ((x, y - 1), GameButton::Up),
            ((x, y + 1), GameButton::Down),
        ] {
            let (xx, yy) = next;
            if xx < 0
                || yy < 0
                || xx >= map.attributes.width as i16 * 2
                || yy >= map.attributes.height as i16 * 2
                || blocked.contains(&next)
                || parents.contains_key(&next)
            {
                continue;
            }
            let block =
                map.blocks[yy as usize / 2 * usize::from(map.attributes.width) + xx as usize / 2];
            let kind = &r.data().tilesets[&map.attributes.tileset_name].collision
                [&format!("{block:02x}")][yy as usize % 2 * 2 + xx as usize % 2];
            if !matches!(
                kind.as_str(),
                "FLOOR" | "TALL_GRASS" | "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK"
            ) {
                continue;
            }
            parents.insert(next, ((x, y), button));
            queue.push_back(next);
        }
    }
    assert!(parents.contains_key(&target), "current dry Cedar trail");
    let mut route = Vec::new();
    let mut cursor = target;
    while cursor != start {
        let (from, button) = parents[&cursor];
        route.push(button);
        cursor = from;
    }
    for button in route.into_iter().rev() {
        let before = c.snapshot().expect("Cedar step").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if let Some(battle) = c.snapshot().expect("Cedar encounter").battle {
                match battle.kind {
                    crystal_runtime::RuntimeBattleKind::Wild { .. } => {
                        super::super::rochester::flee(c)
                    }
                    crystal_runtime::RuntimeBattleKind::Trainer { trainer_id, .. } => {
                        assert!(try_battle(
                            c,
                            r,
                            &trainer_id,
                            &format!("EVENT_{trainer_id}_DEFEATED")
                        ));
                    }
                    _ => panic!("unexpected Cedar battle"),
                }
            }
            let after = c.snapshot().expect("Cedar return").overworld;
            assert_eq!(after.map_name, "MnMaraisCedarApproach");
            if after.tile != before {
                break;
            }
        }
        assert_ne!(
            c.snapshot().expect("native Cedar movement").overworld.tile,
            before
        );
    }
    at(c, "MnMaraisCedarApproach", target);
}

fn coast(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if c.snapshot().expect("coastal departure").overworld.map_name == "MnDuluthHarborWorkshop" {
        walk_to(c, r, (8, 10));
        travel(c, GameButton::Down, "MnDuluth", (72, 42));
        care(c, r, "MnDuluth", (69, 32));
        super::northstar_earned_services::duluth(c, r, root);
        let north = r.data().maps["MnDuluth"]
            .attributes
            .connections
            .iter()
            .find(|link| link.direction == "north" && link.target_map == "MnR11C00R03")
            .expect("current Duluth northern port");
        let x = 46 + i16::try_from(north.offset * 2).expect("north port offset");
        walk_to(c, r, (x, 4));
        travel(c, GameButton::Up, "MnR11C00R03", (46, 47));
        // Optional Ryan visit on the real Baptism River bank.
        road(c, r, 11, (23, 24));
        travel(c, GameButton::Up, "MnRyansHouse", (5, 6));
        walk_to(c, r, (5, 3));
        press(c, GameButton::Right);
        press(c, GameButton::A);
        assert_eq!(
            c.snapshot()
                .expect("Ryan dialogue")
                .ui
                .text
                .expect("text")
                .label,
            "MN_RYANText"
        );
        finish(c);
        continue_in_place(c, root, r, "campaign-ryans-house");
        walk_to(c, r, (5, 6));
        travel(c, GameButton::Down, "MnR11C00R00", (23, 22));
        // The accepted compact coast reaches town by its walking seam.
        road(c, r, 11, (71, 8));
        travel(c, GameButton::Right, "MnGrandMarais", (0, 8));
        travel(c, GameButton::Right, "MnGrandMarais", (3, 8));
        continue_in_place(c, root, r, "campaign-grand-marais-arrival");
    }
    if c.snapshot().expect("Gunflint departure").overworld.map_name == "MnGrandMarais" {
        walk_to(c, r, (15, 10));
        travel(c, GameButton::Up, "MnGrandMaraisPokecenter", (3, 6));
        walk_to(c, r, (3, 3));
        press(c, GameButton::Up);
        press(c, GameButton::A);
        let money = c.shell.shell.session().state().money;
        choice(c, true);
        finish(c);
        assert_eq!(c.shell.shell.session().state().money, money);
        walk_to(c, r, (3, 6));
        travel(c, GameButton::Down, "MnGrandMarais", (15, 10));
        walk_to(c, r, (18, 4));
        travel(c, GameButton::Up, "MnMaraisCedarApproach", (26, 23));
        walk_cedar(c, r, (25, 14));
        travel(c, GameButton::Left, "MnGrandMaraisShoreGate", (8, 4));
        walk_to(c, r, (1, 4));
        travel(c, GameButton::Left, "MnMaraisCedarApproach", (18, 14));
        walk_cedar(c, r, (0, 14));
        travel(c, GameButton::Left, "MnR12C00R01", (63, 6));
        road(c, r, 12, (0, 28));
        travel(c, GameButton::Left, CAMP, (63, 24));
        travel(c, GameButton::Left, CAMP, (60, 24));
        continue_in_place(c, root, r, "campaign-boundary-arrival");
    }
    for flag in [
        "EVENT_MN_GRAND_MARAIS_STARTER",
        "EVENT_MN_GRAND_MARAIS_SIGNAL",
        "EVENT_MN_MARAIS_SPARE_PART_RECEIVED",
    ] {
        assert!(
            !event(c, flag),
            "optional coastal progress stays optional: {flag}"
        );
    }
}

fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (43, 20));
    travel(c, GameButton::Up, ROOM, (8, 9));
}

fn refusal(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (32, 41));
    for _ in 0..8 {
        if c.snapshot().expect("League denial").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot()
            .expect("restoration prerequisite")
            .ui
            .text
            .expect("shutdown directions")
            .label,
        "MnBoundaryLeagueCheckRestoredText"
    );
    press(c, GameButton::Down);
    at(c, CAMP, (32, 42));
    finish(c);
    at(c, CAMP, (32, 41));
    assert!(!event(c, RESTORED));
}

fn pages(c: &mut VisibleShellController, label: &str) -> String {
    let mut result = String::new();
    for _ in 0..64 {
        let s = c.snapshot().expect("native story page");
        let Some(text) = s.ui.text else {
            return result;
        };
        if text.label != label {
            return result;
        }
        let before = s.overworld;
        result.push_str(
            &text
                .asm_text
                .expect("visible story text")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        );
        result.push(' ');
        press(c, GameButton::Down);
        let after = c.snapshot().expect("text owns movement").overworld;
        assert_eq!((before.map_name, before.tile), (after.map_name, after.tile));
        press(c, GameButton::A);
    }
    panic!("native story text never settled");
}

fn receiver(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (12, 4));
    press(c, GameButton::Up);
    press(c, GameButton::A);
}

#[test]
#[ignore = "user-authorized boosted story review from the actual harbor or Boundary save"]
fn northstar_campaign_boosted_coast_chief_relay_and_ely_trail() {
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some());
    let _ = finale();
}

fn departure(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    receiver(c, r);
    assert_eq!(
        c.snapshot()
            .expect("repeat receiver")
            .ui
            .text
            .expect("text")
            .label,
        "MnBoundarySurveyReceiverAlreadyText"
    );
    finish(c);
    interact(c, r, (3, 7));
    finish(c);
    walk_to(c, r, (9, 10));
    travel(c, GameButton::Down, CAMP, (43, 20));
    walk_to(c, r, (32, 41));
    // Walk the authored clearance trigger through input before planning any
    // road travel. The local planner deliberately avoids coordinate events.
    travel(c, GameButton::Down, CAMP, (32, 44));
    let offset = r.data().maps[CAMP]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction == "south" && link.target_map == "MnR13C00R00")
        .expect("published League exit")
        .offset;
    // Frozen earned saves retain their historical pack. New candidates use
    // the unshifted turnout, without migrating those saves to a new identity.
    let landing_x = i16::try_from(32 - offset * 2).expect("native League landing extent");
    travel(c, GameButton::Down, "MnR13C00R00", (landing_x, 0));
    continue_in_place(c, root, r, "campaign-league-road");
    road(c, r, 13, (34, 126));
    at(c, "MnR13C00R02", (34, 30));
    continue_in_place(c, root, r, "campaign-ely-frontier");
}

pub(super) fn finale() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = arrival();
    if c.snapshot()
        .expect("actual intake Continue")
        .overworld
        .map_name
        == "MnBoundaryIntake"
    {
        assert!(event(&c, CHIEF) && event(&c, "EVENT_MN_ARC_LINK_SENT"));
        super::northstar_arc_campaign::intake(&mut c, &root, &r);
        departure(&mut c, &r, &root);
        return (root, r, c);
    }
    if event(&c, RESTORED) {
        departure(&mut c, &r, &root);
        return (root, r, c);
    }
    boost(&mut c, &r);
    coast(&mut c, &r, &root);
    if c.snapshot().expect("shelter approach").overworld.map_name == CAMP {
        refusal(&mut c, &r);
        enter(&mut c, &r);
    }
    if !event(&c, CHIEF) {
        receiver(&mut c, &r);
        assert_eq!(
            c.snapshot()
                .expect("locked receiver")
                .ui
                .text
                .expect("text")
                .label,
            "MnBoundarySurveyReceiverLockedText"
        );
        finish(&mut c);
        let money = c.shell.shell.session().state().money;
        interact(&mut c, &r, (3, 7));
        finish(&mut c);
        assert_eq!(c.shell.shell.session().state().money, money);
        assert!(
            c.snapshot()
                .expect("free field care")
                .party
                .slots
                .iter()
                .all(|p| p.pokemon.hp == p.pokemon.max_hp
                    && p.pokemon.status.is_none()
                    && p.pokemon
                        .moves
                        .iter()
                        .all(|m| m.current_pp == r.data().moves[&m.name].pp))
        );
        walk_to(&mut c, &r, (8, 5));
        continue_in_place(&mut c, &root, &r, "campaign-chief-ready");
        press(&mut c, GameButton::Right);
        press(&mut c, GameButton::A);
        assert!(try_battle(&mut c, &r, "MN_LODE_CHIEF", CHIEF));
        assert!(!event(&c, RESTORED));
        continue_in_place(&mut c, &root, &r, "campaign-chief-victory");
    }
    walk_to(&mut c, &r, (8, 5));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    let chief_reply = pages(&mut c, "MnBoundarySurveyChiefAfterText");
    if r.data()
        .story_event_script_constants
        .global
        .contains_key("EVENT_MN_TAMARACK_INTRO")
    {
        let expected = super::super::northstar_dialogue::canonical_pages(
            "MS11",
            "MnBoundarySurveyChiefAfterText",
        )
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
        assert_eq!(chief_reply.trim(), expected);
    } else {
        assert!(chief_reply.contains("have not changed my mind"));
    }
    finish(&mut c);
    walk_to(&mut c, &r, (8, 10));
    travel(&mut c, GameButton::Down, CAMP, (43, 20));
    refusal(&mut c, &r);
    enter(&mut c, &r);
    if r.data().maps.contains_key("MnBoundaryIntake") {
        super::northstar_arc_campaign::intake(&mut c, &root, &r);
    } else {
        receiver(&mut c, &r);
        // Ordinary poison damage on the walk to the console precedes this scene.
        let before = c.shell.shell.session().state().clone();
        choice(&mut c, false);
        assert!(pages(&mut c, "MnBoundarySurveyReceiverDeclinedText").contains("another time"));
        finish(&mut c);
        assert!(!event(&c, RESTORED));
        receiver(&mut c, &r);
        choice(&mut c, true);
        assert!(pages(&mut c, "MnBoundarySurveyReceiverRestoreText").contains("align the dial"));
        let mut picture = false;
        for _ in 0..96 {
            assert!(
                !event(&c, RESTORED),
                "restoration must wait for the feed and cut-link acknowledgment"
            );
            if c.snapshot()
                .expect("feed")
                .ui
                .active_pokemon_picture
                .as_deref()
                == Some("KETTLURK")
            {
                picture = true;
                break;
            }
            press(&mut c, GameButton::A);
        }
        assert!(picture, "native Kettlurk remote-feed picture");
        press(&mut c, GameButton::Down);
        at(&mut c, ROOM, (12, 4));
        assert_eq!(
            c.snapshot()
                .expect("picture owns input")
                .ui
                .active_pokemon_picture
                .as_deref(),
            Some("KETTLURK")
        );
        continue_in_place(&mut c, &root, &r, "campaign-kettlurk-feed");
        press(&mut c, GameButton::A);
        for _ in 0..96 {
            if c.snapshot()
                .expect("cut-link text")
                .ui
                .text
                .is_some_and(|t| t.label == "MnNorthstarRelayCutText")
            {
                break;
            }
            assert!(!event(&c, RESTORED));
            press(&mut c, GameButton::A);
        }
        assert!(!event(&c, RESTORED));
        let text = pages(&mut c, "MnNorthstarRelayCutText");
        assert!(text.contains("motors coast") && text.contains("cage opens"));
        finish(&mut c);
        assert!(event(&c, RESTORED));
        let off = *c.shell.shell.session().state().map_block_overrides[ROOM]
            .get(&(6, 1))
            .expect("native console switched off");
        assert_ne!(off, r.data().maps[ROOM].blocks[14]);
        let after = c.shell.shell.session().state();
        assert_eq!(after.storage, before.storage);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.money, before.money);
        assert_eq!(after.badges, before.badges);
        assert!(
            !after
                .flags
                .is_engine_flag_set("ENGINE_SAFARI_ZONE")
                .expect("optional Safari admission")
        );
    }
    continue_in_place(&mut c, &root, &r, "campaign-survey-restored");
    departure(&mut c, &r, &root);
    (root, r, c)
}
