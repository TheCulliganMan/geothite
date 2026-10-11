use std::{collections::BTreeSet, path::PathBuf};

use crystal_assets::{GameDataSet, read_loaded_verified_compiled_game_pack};
use crystal_core::{
    input::GameButton,
    random::ReplayDivider,
    state::GameState,
    world::{
        map::{Direction, TilePosition},
        movement::{MovementMode, StepOutcome},
    },
};

#[test]
fn running_shoes_external_pack_default_hold_release_and_opt_out() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("content-packs/text-tui.crystalpack"));
    if !path.exists() {
        return;
    }
    let loaded = read_loaded_verified_compiled_game_pack(&path).expect("verified external pack");
    let mut data = loaded.pack().data().clone();
    assert_eq!(
        data.running_shoes, None,
        "legacy pack should inherit core default"
    );
    let mut state = GameState::default();
    let mut session = data
        .overworld_session("PlayersHouse2F", TilePosition::new(3, 3), 0)
        .unwrap();
    session.player.facing = Direction::Right;
    for (buttons, speed, x) in [
        (vec![GameButton::B, GameButton::Right], 2, 4),
        (vec![GameButton::B, GameButton::Right], 2, 5),
        (vec![GameButton::Right], 1, 6),
    ] {
        let frame = data
            .apply_overworld_input(
                &mut state,
                &mut session,
                buttons,
                &BTreeSet::new(),
                &mut ReplayDivider::new(vec![0; 128]),
            )
            .unwrap();
        assert!(
            matches!(frame.movement, Some(StepOutcome::Moved { speed_multiplier, .. }) if speed_multiplier == speed)
        );
        assert_eq!(frame.snapshot.tile, TilePosition::new(x, 3));
        assert_eq!(session.player.mode, MovementMode::Normal);
        assert!(
            frame.step_events.is_some(),
            "running must process ordinary step events"
        );
    }
    let idle = data
        .apply_overworld_input(
            &mut state,
            &mut session,
            [GameButton::B],
            &BTreeSet::new(),
            &mut ReplayDivider::new(vec![0; 128]),
        )
        .unwrap();
    assert_eq!(idle.movement, None);
    data.running_shoes = Some(false);
    let frame = data
        .apply_overworld_input(
            &mut state,
            &mut session,
            [GameButton::B, GameButton::Right],
            &BTreeSet::new(),
            &mut ReplayDivider::new(vec![0; 128]),
        )
        .unwrap();
    assert!(matches!(
        frame.movement,
        Some(StepOutcome::Moved {
            speed_multiplier: 1,
            ..
        })
    ));
    let serialized = serde_json::to_value(&data).unwrap();
    assert_eq!(serialized["running_shoes"], false);
    let roundtrip: GameDataSet = serde_json::from_value(serialized).unwrap();
    assert_eq!(roundtrip.running_shoes, Some(false));
}

#[test]
fn running_shoes_default_does_not_add_a_legacy_pack_field() {
    let value = serde_json::to_value(GameDataSet::default()).unwrap();
    assert!(value.get("running_shoes").is_none());
    let mut manifest = crystal_assets::ModpackManifest::default();
    assert!(
        serde_json::to_value(&manifest.payload)
            .unwrap()
            .get("running_shoes")
            .is_none()
    );
    manifest.payload.running_shoes = Some(false);
    assert_eq!(
        serde_json::to_value(&manifest.payload).unwrap()["running_shoes"],
        false
    );
}
