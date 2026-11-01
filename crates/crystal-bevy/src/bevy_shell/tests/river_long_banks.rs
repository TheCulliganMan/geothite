//! Initial legal Floor/Mew80/Surf/badge/RNG fixtures, then production input only.
//! These local bank cases do not prove earned regional campaign progression.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::r10_gallery::flags;
use super::super::rochester_earned::try_battle;
use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, assert_at, at, collision, fixture_with_items, mount, settle,
    step,
};

struct Bank {
    map: &'static str,
    script: &'static str,
    name: &'static str,
    class: &'static str,
    // Exact frozen course from the trainer's west approach to its real landing.
    approach_to_landing: &'static [Point],
    water: Point,
    party: &'static [(&'static str, u8)],
    gross_prize: u32,
}

const LOWER_FISHER: Bank = Bank {
    map: "MnMississippiLowerWoodsC00R01",
    script: "MN_RIVER_LOWER_BANK_FISHER",
    name: "ELI",
    class: "FISHER",
    approach_to_landing: &[(20, 13), (20, 14), (20, 15)],
    water: (20, 16),
    party: &[("POLIWHIRL", 30), ("GOLDEEN", 29)],
    gross_prize: 1160,
};
const LOWER_KEEPER: Bank = Bank {
    map: "MnMississippiLowerWoodsC01R01",
    script: "MN_RIVER_LOWER_PINE_KEEPER",
    name: "ASH",
    class: "BIRD_KEEPER",
    approach_to_landing: &[(38, 35), (38, 34), (39, 34), (40, 34), (41, 34)],
    water: (42, 34),
    party: &[("NOCTOWL", 31), ("FARFETCH_D", 30)],
    gross_prize: 720,
};
const UPPER_FISHER: Bank = Bank {
    map: "MnMississippiUpstreamWoodsC00R02",
    script: "MN_RIVER_UPSTREAM_BANK_FISHER",
    name: "DALE",
    class: "FISHER",
    approach_to_landing: &[(32, 35), (31, 35), (30, 35), (29, 35), (28, 35)],
    water: (27, 35),
    party: &[("SEAKING", 31), ("QUAGSIRE", 32)],
    gross_prize: 1280,
};
const UPPER_KEEPER: Bank = Bank {
    map: "MnMississippiUpstreamWoodsC01R00",
    script: "MN_RIVER_UPSTREAM_WOODS_KEEPER",
    name: "PINE",
    class: "BIRD_KEEPER",
    approach_to_landing: &[(4, 23), (4, 24), (5, 24), (6, 24), (6, 25)],
    water: (6, 26),
    party: &[("NOCTOWL", 32), ("PIDGEOTTO", 31)],
    gross_prize: 744,
};

fn button(from: Point, to: Point) -> GameButton {
    match (to.0 - from.0, to.1 - from.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("nonadjacent frozen bank course: {other:?}"),
    }
}

fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("existing native bank trainer flag")
}

fn retained_save(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let directory = std::env::var_os("CRYSTAL_RIVER_LONG_BANK_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("geothite-river-long-banks"));
    std::fs::create_dir_all(&directory).expect("retained native bank saves");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save clock")
        .as_nanos();
    let path = directory.join(format!(
        "{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists(), "retain every previous native bank save");
    c.save(&path).expect("production bank Save");
    path
}

fn continue_game(
    root: &AssetRoot,
    r: &CrystalRuntime,
    c: &mut VisibleShellController,
    label: &str,
) {
    let before = c.snapshot().expect("bank Continue preimage");
    let before_flags = flags(c);
    let before_money = c.shell.shell.session().state().money;
    let before_mom = c.shell.shell.session().state().moms_money;
    let path = retained_save(c, label);
    *c = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("native bank Continue");
    let after = c.snapshot().expect("continued bank");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.overworld.facing, after.overworld.facing);
    assert_eq!(before_flags, flags(c));
    assert_eq!(before_money, c.shell.shell.session().state().money);
    assert_eq!(before_mom, c.shell.shell.session().state().moms_money);
}

fn walk(c: &mut VisibleShellController, bank: &Bank, course: impl IntoIterator<Item = Point>) {
    for point in course {
        let before = at(c).1;
        if before == point {
            continue;
        }
        step(
            c,
            button(before, point),
            bank.map,
            point,
            "frozen bank Foot approach",
        );
        let snapshot = c.snapshot().expect("native bank walking");
        assert_eq!(snapshot.overworld.mode, MovementMode::Normal);
        assert!(
            snapshot.battle.is_none(),
            "manual approach avoids trainer sight"
        );
    }
}

fn run_bank(bank: &Bank) {
    let approach = bank.approach_to_landing[0];
    let landing = *bank.approach_to_landing.last().expect("real bank landing");
    let (root, r, mut c) =
        fixture_with_items(bank.map, landing, false, false, false, &[("MAX_REPEL", 2)]);
    assert_eq!(r.data().maps.len(), 683);
    let flag = format!("EVENT_{}_DEFEATED", bank.script);
    assert!(!event(&c, &flag));
    let native = &r.data().trainers.trainers[bank.script];
    assert_eq!(native.name, bank.name);
    assert_eq!(native.trainer_class, bank.class);
    assert_eq!(
        native
            .party
            .iter()
            .map(|p| (p.species.as_str(), p.level))
            .collect::<Vec<_>>(),
        bank.party
    );
    assert_eq!(
        4 * native.base_reward * u32::from(native.party.last().expect("trainer party").level),
        bank.gross_prize
    );
    assert_eq!(collision(&r, &mut c, landing), "FLOOR");
    assert_eq!(collision(&r, &mut c, bank.water), "WATER");
    use_max_repel(&mut c, &r);
    mount(&mut c, button(landing, bank.water), bank.map, bank.water);
    assert_eq!(
        c.snapshot().expect("real launch").overworld.mode,
        MovementMode::Surf
    );
    step(
        &mut c,
        button(bank.water, landing),
        bank.map,
        landing,
        "native bank dismount",
    );
    assert_eq!(
        c.snapshot().expect("real landing").overworld.mode,
        MovementMode::Normal
    );
    walk(&mut c, bank, bank.approach_to_landing.iter().rev().copied());
    c.press(GameButton::Right)
        .expect("face adjacent native trainer");
    c.wait_frames(16).expect("bank facing presentation");
    assert_at(
        &mut c,
        bank.map,
        approach,
        "trainer blocks forward movement",
    );
    assert!(
        c.snapshot()
            .expect("manual bank encounter")
            .battle
            .is_none()
    );
    retained_save(&mut c, &format!("{}-manual-approach", bank.script));
    let before = c.snapshot().expect("before actual trainer");
    let money = c.shell.shell.session().state().money;
    let total_money = money + c.shell.shell.session().state().moms_money;
    c.press(GameButton::A)
        .expect("manual native trainer challenge");
    for _ in 0..128 {
        if c.snapshot().expect("trainer introduction").battle.is_some() {
            break;
        }
        c.press(GameButton::A)
            .expect("native trainer introduction page");
        c.wait_frames(16).expect("trainer presentation");
    }
    let battle = c
        .snapshot()
        .expect("native bank battle")
        .battle
        .expect("manual interaction starts trainer battle");
    assert!(
        matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, event_flag, .. }
        if trainer_id == bank.script && event_flag == &flag)
    );
    assert_eq!(
        battle
            .enemy_party
            .iter()
            .map(|p| (p.species.id.as_str(), p.level))
            .collect::<Vec<_>>(),
        bank.party
    );
    assert!(
        try_battle(&mut c, &r, bank.script, &flag),
        "actual bank trainer victory"
    );
    settle(&mut c);
    let won = c.snapshot().expect("actual bank trainer outcome");
    assert!(won.battle.is_none() && won.party.slots[0].pokemon.hp > 0);
    assert!(event(&c, &flag));
    assert!(
        won.party.slots[0].pokemon.moves[0].current_pp
            < before.party.slots[0].pokemon.moves[0].current_pp
    );
    assert!(won.party.slots[0].pokemon.experience > before.party.slots[0].pokemon.experience);
    assert!(c.shell.shell.session().state().money > money);
    assert_eq!(
        c.shell.shell.session().state().money + c.shell.shell.session().state().moms_money,
        total_money + bank.gross_prize
    );
    continue_game(&root, &r, &mut c, &format!("{}-victory-foot", bank.script));
    let won = c.snapshot().expect("continued bank victory");
    let money = c.shell.shell.session().state().money;
    let mom = c.shell.shell.session().state().moms_money;
    c.press(GameButton::A)
        .expect("already defeated native trainer");
    let text = settle(&mut c).join(" ").to_ascii_lowercase();
    assert!(
        text.contains("safe travels"),
        "original defeated-trainer dialogue: {text:?}"
    );
    let again = c.snapshot().expect("no duplicate trainer reward");
    assert!(again.battle.is_none());
    assert_eq!(again.party, won.party);
    assert_eq!(again.bag, won.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(c.shell.shell.session().state().moms_money, mom);
    assert!(event(&c, &flag));
    walk(&mut c, bank, bank.approach_to_landing.iter().copied());
    mount(&mut c, button(landing, bank.water), bank.map, bank.water);
    continue_game(&root, &r, &mut c, &format!("{}-victory-surf", bank.script));
    assert_at(
        &mut c,
        bank.map,
        bank.water,
        "Continue preserves native bank remount",
    );
    assert_eq!(
        c.snapshot().expect("continued river mode").overworld.mode,
        MovementMode::Surf
    );
    assert!(event(&c, &flag));
}

#[test]
#[ignore = "requires written long-river pack in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn river_long_bank_lower_fisher_mount_manual_battle_reward_once_and_continue() {
    run_bank(&LOWER_FISHER);
}

#[test]
#[ignore = "requires written long-river pack in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn river_long_bank_lower_keeper_mount_manual_battle_reward_once_and_continue() {
    run_bank(&LOWER_KEEPER);
}

#[test]
#[ignore = "requires written long-river pack in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn river_long_bank_upper_fisher_mount_manual_battle_reward_once_and_continue() {
    run_bank(&UPPER_FISHER);
}

#[test]
#[ignore = "requires written long-river pack in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn river_long_bank_upper_keeper_mount_manual_battle_reward_once_and_continue() {
    run_bank(&UPPER_KEEPER);
}
