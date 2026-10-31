//! Explicit placement/wallet fixtures; all game actions use production input.

use crystal_core::input::GameButton;
use crystal_core::state::SlotSymbol;

use super::super::{
    VisibleCardFlipPhase, VisibleShellController, VisibleSlotMachinePhase,
    load_card_flip_render_sources, mark_runtime_snapshot_dirty,
};
use super::interior_life::{fixture, load};

const ROOM: &str = "MnMinneapolisCornerHome";

fn ready(at: (i16, i16)) -> VisibleShellController {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, ROOM, at);
    c.shell.shell.session_mut().state_mut().coins = 20;
    c.shell
        .shell
        .add_bag_item("COIN_CASE", 1)
        .expect("initial Coin Case fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production Game Corner input");
    c.wait_frames(120).expect("native minigame clock");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the Minnesota Game Corner"]
fn minnesota_game_corner_controller_stops_all_reels_and_commits_payout() {
    let mut c = ready((5, 6));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell
            .visible_slot_machine
            .as_ref()
            .expect("real machine")
            .phase,
        VisibleSlotMachinePhase::Betting
    );
    press(&mut c, GameButton::A);
    assert_eq!(c.snapshot().expect("stake snapshot").trainer.coins, 17);
    for _ in 0..3 {
        press(&mut c, GameButton::A);
    }
    c.wait_frames(1600)
        .expect("finish native result and coin payout");
    let machine = c
        .shell
        .visible_slot_machine
        .as_ref()
        .expect("result remains visible");
    assert_eq!(machine.phase, VisibleSlotMachinePhase::Result);
    let core = c
        .shell
        .shell
        .session()
        .state()
        .script_runtime
        .slot_machine
        .as_ref()
        .expect("core machine");
    assert_eq!(core.next_reel, 4);
    assert_eq!(core.payout_remaining, 0);
    let payout = match core.matched_symbol {
        None => 0,
        Some(SlotSymbol::Seven) => 300,
        Some(SlotSymbol::Pokeball) => 50,
        Some(SlotSymbol::Cherry) => 6,
        Some(SlotSymbol::Pikachu) => 8,
        Some(SlotSymbol::Squirtle) => 10,
        Some(SlotSymbol::Staryu) => 15,
    };
    assert_eq!(
        c.snapshot().expect("paid snapshot").trainer.coins,
        17 + payout
    );
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::B);
    assert!(c.shell.visible_slot_machine.is_none());
    press(&mut c, GameButton::Down);
    if c.snapshot().expect("native facing turn").overworld.tile.y == 6 {
        press(&mut c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot().expect("movement after quit").overworld.tile.y,
        7
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the Minnesota Game Corner"]
fn minnesota_game_corner_controller_card_flip_wager_pays_and_continues() {
    let (root, runtime, _) = load();
    let mut c = ready((14, 6));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell
            .visible_card_flip
            .as_ref()
            .expect("real cards")
            .phase,
        VisibleCardFlipPhase::AskPlay
    );
    press(&mut c, GameButton::A);
    assert_eq!(c.snapshot().expect("three-coin stake").trainer.coins, 17);
    press(&mut c, GameButton::A);
    let game = c.shell.visible_card_flip.as_ref().expect("native bet grid");
    assert_eq!(game.phase, VisibleCardFlipPhase::PlaceBet);
    // Read this test fixture's selected card to exercise the 72-coin exact-match
    // branch deterministically. No live state is edited after input begins.
    let face = c
        .shell
        .shell
        .session()
        .state()
        .script_runtime
        .card_flip
        .as_ref()
        .expect("core shuffled deck")
        .deck[game.which_card];
    for _ in 0..(face & 3) {
        press(&mut c, GameButton::Right);
    }
    for _ in 0..(face >> 2) {
        press(&mut c, GameButton::Down);
    }
    press(&mut c, GameButton::A);
    c.wait_frames(1000)
        .expect("native reveal and all 72 payout frames");
    let game = c.shell.visible_card_flip.as_ref().expect("visible result");
    assert_eq!(game.phase, VisibleCardFlipPhase::Result);
    assert_eq!(game.payout, 72);
    assert_eq!(c.snapshot().expect("paid Card Flip").trainer.coins, 89);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::B);
    assert!(c.shell.visible_card_flip.is_none());
    press(&mut c, GameButton::Down);
    if c.snapshot().expect("native facing turn").overworld.tile.y == 6 {
        press(&mut c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot().expect("movement after cards").overworld.tile.y,
        7
    );
    let path = std::env::temp_dir().join(format!(
        "geothite-corner-{}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("save earned test payout");
    let mut continued =
        VisibleShellController::load_save(root, runtime, path.clone(), None).expect("Continue");
    assert_eq!(
        continued.snapshot().expect("continued coins").trainer.coins,
        89
    );
    std::fs::remove_file(path).expect("remove temporary test save");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with corrected native Card Flip banks"]
fn minnesota_game_corner_card_flip_board_does_not_wrap_into_card_area() {
    let (_root, runtime, _) = load();
    let root = runtime
        .materialize_runtime_files()
        .expect("native packed art root");
    let sources = load_card_flip_render_sources(&root).expect("packed Card Flip art");
    let green = &sources.base[..4];
    for y in 0..96 {
        for x in 0..72 {
            let offset = (y * 160 + x) * 4;
            assert_eq!(
                &sources.base[offset..offset + 4],
                green,
                "card-area pixel {x},{y}"
            );
        }
    }
}
