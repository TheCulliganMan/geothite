//! Local fixture parties; the approach, encounter, capture and Continue use input.
use crystal_core::input::GameButton;

use super::{checkpoint, encounter_and_capture, fixture, verify_capture};
use crate::bevy_shell::tests::winona::press;

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with landmark habitats"]
fn landmark_oatlet_harvest_field_capture_and_continue() {
    let (root, runtime, mut controller) = fixture("MnR02C00R00", [40, 8], false, false);
    checkpoint(&mut controller, "oatlet-field-approach");
    for _ in 0..4 {
        let tile = controller
            .snapshot()
            .expect("field approach")
            .overworld
            .tile;
        if (tile.x, tile.y) == (40, 7) {
            break;
        }
        press(&mut controller, GameButton::Up);
    }
    crate::bevy_shell::tests::winona::at(&mut controller, "MnR02C00R00", (40, 7));
    encounter_and_capture(&mut controller, &runtime, "OATLET", [40, 7], [41, 7]);
    verify_capture(&controller, &runtime, "OATLET");
    controller = crate::bevy_shell::tests::r10_gallery::continue_save(
        root,
        runtime.clone(),
        &mut controller,
    );
    verify_capture(&controller, &runtime, "OATLET");
    checkpoint(&mut controller, "oatlet-field-caught");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with landmark habitats"]
fn landmark_cinderfin_smokehouse_grass_capture_and_continue() {
    let (root, runtime, mut controller) = fixture("MnR11C00R03", [40, 5], false, false);
    checkpoint(&mut controller, "cinderfin-smokehouse-approach");
    press(&mut controller, GameButton::Down);
    encounter_and_capture(&mut controller, &runtime, "CINDERFIN", [40, 6], [41, 6]);
    verify_capture(&controller, &runtime, "CINDERFIN");
    controller = crate::bevy_shell::tests::r10_gallery::continue_save(
        root,
        runtime.clone(),
        &mut controller,
    );
    verify_capture(&controller, &runtime, "CINDERFIN");
    checkpoint(&mut controller, "cinderfin-smokehouse-caught");
}
