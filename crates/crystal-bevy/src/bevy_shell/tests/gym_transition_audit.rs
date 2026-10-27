//! Verify native edge collision rather than inferring a defect from a floor strip.
use crystal_core::input::GameButton;
use serde_json::json;

use super::{at, controller, load, press};

#[test]
#[ignore = "requires the current regional gym pack via CRYSTAL_GYM_TEST_PACK"]
fn gym_transition_audit_r05_connected_forest_blocks_entry_without_a_wall_spawn() {
    let (root, r, design) = load();
    for x in 14..=27 {
        let mut initial = design["gyms"][4].clone();
        initial["id"] = "MnR05C00R00".into();
        initial["arrival_core"] = json!([x, 47]);
        let mut c = controller(&root, &r, &initial, false);
        for _ in 0..3 {
            press(&mut c, GameButton::Down);
        }
        assert_eq!(
            c.snapshot()
                .expect("blocked destination")
                .overworld
                .map_name,
            "MnR05C00R00",
            "a neighboring tree must block the connection at x={x}"
        );
        assert_eq!(
            at(&mut c),
            (x, 47),
            "never place a player in the neighbor's forest wall"
        );
    }
}
