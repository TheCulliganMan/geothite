//! Nurse and battle recovery using a declared location fixture and real inputs.
#![cfg(feature = "bevy-shell")]

use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::state::OverworldMemory;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the current Minnesota region"]
fn nurse_save_and_actual_battle_loss_return_to_brainerd() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_WINONA_TEST_PACK").expect("review pack"))
        .canonicalize()
        .expect("external pack path");
    let root = AssetRoot::new(pack.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack).expect("verified review pack"),
    )
    .expect("runtime");
    let save_dir =
        std::env::temp_dir().join(format!("geothite-clinic-flow-{}", std::process::id()));
    std::fs::create_dir_all(&save_dir).expect("temporary save directory");
    let save = save_dir.join("fixture.crystalsave");
    let mut game = RuntimeGameShell::new_game_at_runtime_tile(
        root.clone(),
        runtime.clone(),
        runtime
            .title_new_game_spawn_identifier()
            .expect("home spawn"),
        "MnBrainerdPokecenter1F",
        7,
        5,
    )
    .expect("clinic fixture");
    {
        let state = game.session_mut().state_mut();
        let mut partner = runtime
            .data()
            .create_pokemon("BIRCHIT", 5, Dv::default())
            .expect("regional partner");
        partner.hp = 1;
        partner.moves = vec![LearnedMove {
            name: "SPLASH".into(),
            current_pp: 0,
            pp_ups: 0,
        }];
        state.storage.party.pokemon[0] = Some(partner);
        state.last_spawn_map_constant = Some("MN_WINONA_POKECENTER_1F".into());
        state.money = 1200;
        state.sync_party_from_storage();
    }
    game.save(&save).expect("write clinic fixture");
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), save.clone(), None)
            .expect("load clinic fixture");
    controller.press(GameButton::Up).expect("face nurse");
    controller.wait_frames(1).expect("release input");
    controller.press(GameButton::A).expect("speak to nurse");
    controller.wait_frames(1).expect("release input");
    for _ in 0..128 {
        let snapshot = controller.snapshot().expect("nurse dialogue");
        if snapshot.party.slots[0].pokemon.hp == snapshot.party.slots[0].pokemon.max_hp
            && snapshot.ui.text.is_none()
        {
            break;
        }
        controller
            .press(GameButton::A)
            .expect("advance nurse dialogue");
        controller.wait_frames(1).expect("release input");
    }
    let healed = controller.snapshot().expect("healed party");
    assert_eq!(
        healed.party.slots[0].pokemon.hp,
        healed.party.slots[0].pokemon.max_hp
    );
    controller.save(&save).expect("save after nurse");
    let mut state = runtime.load_save(&save).expect("read healed save");
    assert_eq!(
        state.last_spawn_map_constant.as_deref(),
        Some("MN_BRAINERD_POKECENTER_1F")
    );

    // Explicit location fixture: retain the real nurse save, then place the
    // player beside a gym trainer. Battle and whiteout use production input.
    state.overworld = OverworldMemory::Active {
        map_name: "MnBrainerdGym".into(),
        tile: TilePosition::new(2, 11),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    runtime
        .save_game(&save, state)
        .expect("write gym location fixture");
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), save.clone(), None)
            .expect("continue at trainer");
    controller.press(GameButton::A).expect("challenge trainer");
    controller.wait_frames(1).expect("release input");
    let mut fought = false;
    for _ in 0..4096 {
        let snapshot = controller.snapshot().expect("battle or recovery");
        fought |= snapshot.battle.is_some();
        if fought
            && snapshot.battle.is_none()
            && snapshot.overworld.map_name == "MnBrainerdPokecenter1F"
        {
            break;
        }
        controller
            .press(GameButton::A)
            .expect("advance actual battle");
        controller.wait_frames(1).expect("release input");
    }
    let recovered = controller.snapshot().expect("whiteout recovery");
    assert!(fought, "trainer battle must actually start");
    assert_eq!(recovered.overworld.map_name, "MnBrainerdPokecenter1F");
    assert_eq!(recovered.overworld.tile, TilePosition::new(10, 13));
    assert_eq!(
        recovered.party.slots[0].pokemon.hp,
        recovered.party.slots[0].pokemon.max_hp
    );
    controller.save(&save).expect("save after whiteout");
    let state = runtime.load_save(&save).expect("read recovery save");
    assert_eq!(
        state.last_spawn_map_constant.as_deref(),
        Some("MN_BRAINERD_POKECENTER_1F")
    );
    assert_eq!(state.money, 600);
    std::fs::remove_dir_all(save_dir).expect("remove temporary saves");
}
