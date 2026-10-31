//! Preserve native R01 return-walk blackouts instead of following a stale path.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::rochester::{flee, try_trail_with};
use super::super::winona::{at, continue_in_place, travel};
use super::VisibleShellController;

pub(super) fn play(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    root: &AssetRoot,
    checkpoint: &str,
) {
    if controller
        .snapshot()
        .expect("native training return")
        .overworld
        .map_name
        == "MnR01C00R00"
        && try_trail_with(controller, runtime, (0, 24), flee)
    {
        travel(controller, GameButton::Left, "MnRochester", (39, 18));
    }
    let snapshot = controller
        .snapshot()
        .expect("actual return or clinic recovery");
    match snapshot.overworld.map_name.as_str() {
        "MnRochester" => {}
        "MnRochesterPokecenter1F" => {
            at(controller, "MnRochesterPokecenter1F", (10, 13));
            continue_in_place(controller, root, runtime, checkpoint);
        }
        other => panic!("unexpected native training recovery map {other}"),
    }
}

#[test]
#[ignore = "requires CRYSTAL_R01_NATIVE_BLACKOUT_SAVE and its exact CRYSTAL_WINONA_TEST_PACK"]
fn r01_native_saved_blackout_recovery_preserves_earned_state_and_continues() {
    let (root, runtime) = super::super::winona::load();
    let path = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_R01_NATIVE_BLACKOUT_SAVE")
            .expect("actual native early-wild blackout checkpoint"),
    );
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
            .expect("unchanged native blackout save with matching pack identity");
    controller.set_runtime_journal_enabled(false);
    let before = controller.snapshot().expect("native clinic recovery");
    assert_eq!(before.overworld.map_name, "MnRochesterPokecenter1F");
    assert_eq!(
        controller.shell.shell.session().state().badges.johto,
        [false; 8]
    );
    let flags = controller.shell.shell.session().state().flags.clone();
    let money = controller.shell.shell.session().state().money;
    let badges = controller.shell.shell.session().state().badges.clone();
    // No state writes: only the production Save/Continue recovery helper.
    play(
        &mut controller,
        &runtime,
        &root,
        "native-r01-blackout-recovery",
    );
    let after = controller
        .snapshot()
        .expect("continued earned clinic state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(controller.shell.shell.session().state().flags, flags);
    assert_eq!(controller.shell.shell.session().state().money, money);
    assert_eq!(controller.shell.shell.session().state().badges, badges);
}
