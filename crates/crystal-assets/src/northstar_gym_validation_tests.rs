//! Static regressions for actual trainer approaches and optional routes.

use std::path::PathBuf;

use crate::gym_design::GymDesign;
use crate::northstar_gym_design::RoomContract;

#[test]
#[ignore = "requires external CRYSTAL_GYM_PARENT; static pack geometry only"]
fn native_sightline_cannot_turn_an_optional_battle_into_a_required_one() {
    let parent = crate::read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_GYM_PARENT").expect("external unchanged Gym parent"),
    ))
    .expect("verified Gym parent");
    let design: GymDesign = serde_json::from_str(include_str!(
        "../../../planning/minnesota/gyms/overhaul-20261117/brainerd.json"
    ))
    .expect("authored dock room");
    let mut contract: RoomContract = serde_json::from_str(include_str!(
        "../../../planning/minnesota/gyms/continuation-20261117/brainerd-contract.json"
    ))
    .expect("authored dock contract");
    let story = include_str!("../../../planning/minnesota/story.json");
    let gym = &design.gyms[0];
    crate::northstar_gym_rooms::install(&mut parent.clone(), gym, &contract, story)
        .expect("right-facing trainer preserves the optional LEFT bypass");
    let bravel = contract
        .sightlines
        .get_mut("MN_GYM_6_TRAINER_2")
        .expect("Bravel sightline");
    bravel.direction = "down".into();
    bravel.radius = 2;
    let error = crate::northstar_gym_rooms::install(&mut parent.clone(), gym, &contract, story)
        .expect_err("Bravel must not intercept the LEFT route's shared lower pier");
    assert!(
        format!("{error:#}").contains("optional trainer MN_GYM_6_TRAINER_2"),
        "{error:#}"
    );
}
