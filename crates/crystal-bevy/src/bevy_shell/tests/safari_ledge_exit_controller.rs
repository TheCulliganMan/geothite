//! Paid one-way cliff traversal and a real walking exit that closes the permit.

use crystal_core::input::GameButton;
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;

use super::{
    LakeGrid, Point, VisibleShellController, active, admit, button, continue_save,
    controller_fixture, global, launch, press, release_wild, save, step, travel,
};

fn blocked(c: &mut VisibleShellController, direction: GameButton) {
    release_wild(c);
    let position = global(c);
    let budget = active(c).steps_remaining;
    for _ in 0..4 {
        press(c, direction);
        assert_eq!(
            global(c),
            position,
            "native terrain must block the reverse/boundary"
        );
        assert_eq!(
            active(c).steps_remaining,
            budget,
            "turns and bumps are not travel"
        );
    }
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_palisade_hop_grassy_climb_and_real_buoy_boundary() {
    let (root, runtime, mut c) = controller_fixture();
    let grid = LakeGrid::new(&runtime);
    // Setup ends at the public ranger; the entire approach uses native input.
    admit(&mut c);
    travel(&mut c, &grid, (3, 90), false);
    launch(&mut c, (3, 89));
    travel(&mut c, &grid, (36, 68), true);
    step(&mut c, (36, 67));
    assert!(travel(&mut c, &grid, (35, 57), false) >= 3);
    step(&mut c, (35, 58));
    release_wild(&mut c);
    let before_hop = active(&mut c).steps_remaining;
    for _ in 0..8 {
        if global(&mut c) == (35, 60) {
            break;
        }
        press(&mut c, GameButton::Down);
        assert_ne!(
            global(&mut c),
            (35, 59),
            "a ledge jumps over its blocked quadrant"
        );
    }
    assert_eq!(global(&mut c), (35, 60));
    assert_eq!(
        active(&mut c).steps_remaining,
        before_hop - 1,
        "one native two-tile hop is one successful travel commit"
    );
    assert_eq!(
        c.snapshot().expect("landed ledge").overworld.mode,
        MovementMode::Normal
    );
    save(&mut c, "paid-palisade-landing");
    // Landing in habitat can legitimately open wildlife immediately. Retain
    // that checkpoint, then use native Run for a separate visible cliff view.
    release_wild(&mut c);
    save(&mut c, "paid-palisade-landing-field");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    blocked(&mut c, GameButton::Up);
    let return_path = grid.path((35, 60), (35, 57), false);
    assert!(
        return_path.len() > 9,
        "the uphill return must be a real detour"
    );
    let before_return = active(&mut c).steps_remaining;
    assert!(
        travel(&mut c, &grid, (35, 57), false) >= 3,
        "the ordinary uphill return must cross the grassy shoulder"
    );
    assert_eq!(
        active(&mut c).steps_remaining,
        before_return - u16::try_from(return_path.len() - 1).expect("bounded return path")
    );
    save(&mut c, "paid-palisade-grassy-return");
    assert!(travel(&mut c, &grid, (36, 42), false) >= 6);
    launch(&mut c, (36, 41));
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    travel(&mut c, &grid, (1, 92), true);
    assert_eq!(grid.kind((0, 92)), "BUOY");
    blocked(&mut c, GameButton::Left);
    assert_eq!(
        c.snapshot().expect("blocked water edge").overworld.mode,
        MovementMode::Surf
    );
    assert_eq!(active(&mut c).balls_remaining, 30);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    save(&mut c, "paid-west-buoy-blocked");
    let mut continued = continue_save(root, runtime, &mut c);
    assert_eq!(global(&mut continued), (1, 92));
}

fn walk_free_court(c: &mut VisibleShellController, grid: &LakeGrid, target: Point) {
    for tile in grid.path(global(c), target, false).into_iter().skip(1) {
        let from = global(c);
        for _ in 0..4 {
            if global(c) == tile {
                break;
            }
            press(c, button(from, tile));
        }
        assert_eq!(global(c), tile, "actual unpaid court walking");
        assert!(c.snapshot().expect("no free permit").safari.is_none());
    }
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_walking_exit_ends_permit_and_reentry_restores_both_guards() {
    let (root, runtime, mut c) = controller_fixture();
    let grid = LakeGrid::new(&runtime);
    admit(&mut c);
    travel(&mut c, &grid, (14, 111), false);
    let party = c.snapshot().expect("pre-exit regional partner").party;
    for _ in 0..4 {
        press(&mut c, GameButton::Down);
        if c.snapshot()
            .expect("native outward seam")
            .overworld
            .map_name
            == "MnPreservePineEntry"
        {
            break;
        }
    }
    let outside = c.snapshot().expect("actual walking exit");
    assert_eq!(outside.overworld.map_name, "MnPreservePineEntry");
    assert_eq!(outside.overworld.tile, TilePosition::new(16, 0));
    assert_eq!(outside.overworld.mode, MovementMode::Normal);
    assert!(
        outside.safari.is_none(),
        "outward travel cannot leave a live permit outside"
    );
    assert_eq!(outside.party, party);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    save(&mut c, "paid-walked-out");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
        if c.snapshot().expect("unpaid return seam").overworld.map_name == "MnBoundarySafariC00R02"
        {
            break;
        }
    }
    assert_eq!(global(&mut c), (14, 111));
    assert!(c.snapshot().expect("free court reentry").safari.is_none());
    for x in [12, 11] {
        walk_free_court(&mut c, &grid, (x, 102));
        assert!(
            c.shell
                .shell
                .session()
                .overworld()
                .visible_object_at_checked(TilePosition::new(x, 5))
                .expect("actual restored guard")
                .is_some(),
            "the real guard body must return, not just a logical denial"
        );
        for _ in 0..4 {
            press(&mut c, GameButton::Up);
            assert_eq!(global(&mut c), (x, 102));
            assert!(c.snapshot().expect("closed unpaid gate").safari.is_none());
        }
    }
    assert_eq!(c.shell.shell.session().state().money, 1000);
    save(&mut c, "paid-reentry-guards-closed");
    press(&mut c, GameButton::A);
    let before_text = c.snapshot().expect("native guard directions");
    let pages = super::super::super::super::visible_field_dialog_pages(&before_text, &c.shell)
        .expect("authored gate dialogue")
        .join("\n");
    assert!(pages.contains("Permit required."));
    assert!(pages.contains("Ask the ranger"));
    assert!(!pages.contains("EXPEDITIONS CLOSED"));
    let position = global(&mut c);
    press(&mut c, GameButton::Down);
    assert_eq!(
        global(&mut c),
        position,
        "native permit directions own input"
    );
    save(&mut c, "paid-gate-permit-directions");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert_eq!(
        c.snapshot()
            .expect("guard text Continue")
            .ui
            .pending_text_wait,
        before_text.ui.pending_text_wait
    );
    press(&mut c, GameButton::A);
    let second_text = c.snapshot().expect("next native directions page");
    let pages = super::super::super::super::visible_field_dialog_pages(&second_text, &c.shell)
        .expect("second authored gate page")
        .join("\n");
    assert!(pages.contains("at basecamp to"));
    assert!(pages.contains("start a trip."));
    assert!(!pages.contains("EXPEDITIONS CLOSED"));
    let live_memory = c
        .shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory
        .clone();
    save(&mut c, "paid-gate-permit-next-page");
    assert_eq!(
        c.shell.shell.session().state().script_runtime.memory,
        live_memory
    );
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert_eq!(
        c.snapshot()
            .expect("second guard page Continue")
            .ui
            .pending_text_wait,
        second_text.ui.pending_text_wait
    );
    let resumed = c
        .snapshot()
        .expect("same visible second page after Continue");
    let resumed_pages = super::super::super::super::visible_field_dialog_pages(&resumed, &c.shell)
        .expect("continued native gate page")
        .join("\n");
    assert!(resumed_pages.contains("at basecamp to"));
    assert!(resumed_pages.contains("start a trip."));
    assert!(!resumed_pages.contains("Permit required."));
    save(&mut c, "paid-gate-permit-next-page-continued");
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("guard dialogue settling");
        if snapshot.ui.pending_text_wait.is_none()
            && !snapshot.ui.text_window_open
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.snapshot()
            .expect("guard dialogue closed")
            .ui
            .pending_text_wait
            .is_none()
    );
    assert!(!c.has_pending_script_work());
    save(&mut c, "paid-guard-dialogue-closed");
    let mut continued = continue_save(root, runtime, &mut c);
    assert!(
        continued
            .snapshot()
            .expect("closed gate Continue")
            .safari
            .is_none()
    );
    let closed = continued
        .snapshot()
        .expect("closed dialogue after Continue");
    assert!(
        super::super::super::super::visible_field_dialog_pages(&closed, &continued.shell).is_none()
    );
}
