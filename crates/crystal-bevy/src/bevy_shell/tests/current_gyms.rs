//! All current city doors on a selected regional pack; no earned-travel claim.
use crystal_assets::read_minnesota_gym_design;
use serde_json::Value;

use super::{continue_save, controller, exit_and_reenter, load, point};

#[test]
#[ignore = "requires a published current-gym regional pack via CRYSTAL_GYM_TEST_PACK"]
fn current_gyms_controller_all_eight_actual_city_doors_both_carpets_and_continue() {
    let (root, runtime, _) = load();
    let source = read_minnesota_gym_design(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/gyms/current.json"),
    )
    .expect("Rust current selection");
    let design: Value = serde_json::from_str(&source).expect("current rooms");
    for gym in design["gyms"].as_array().expect("eight rooms") {
        // Each room is a separate entrance/prior-badge/party fixture. Travel,
        // both native carpets, real city reentry and Continue use only input.
        let mut c = controller(&root, &runtime, gym, false);
        for carpet in gym["exit_tiles_core"].as_array().expect("paired carpets") {
            exit_and_reenter(&mut c, &runtime, gym, point(carpet));
            c = continue_save(&mut c, &root, &runtime);
        }
    }
}
