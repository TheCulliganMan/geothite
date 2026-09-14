use crystal_runtime::{
    CrystalRuntime, RuntimeGameShell,
    assets::{AssetRoot, read_loaded_verified_compiled_game_pack},
};
use std::path::PathBuf;

fn game() -> RuntimeGameShell {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let loaded = read_loaded_verified_compiled_game_pack(
        root.join("content-packs/core-modular.browser.crystalpack"),
    )
    .expect("read the shipped pack");
    let assets = AssetRoot::new(&root);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&assets, loaded).unwrap();
    let spawn = runtime.title_new_game_spawn_identifier().unwrap();
    RuntimeGameShell::new_game(assets, runtime, spawn).unwrap()
}

#[test]
fn renderer_free_sessions_replay_identically_and_share_audio() {
    // The real pack exercises the public API from a different crate, without
    // importing Bevy or writing generated pack/PCM artifacts.
    let mut first = game();
    let mut second = first.clone();
    for _ in 0..3 {
        first.tick([]).unwrap();
        second.tick([]).unwrap();
        assert_eq!(first.snapshot().unwrap(), second.snapshot().unwrap());
        assert_eq!(
            first.drain_resolved_audio_events().unwrap(),
            second.drain_resolved_audio_events().unwrap()
        );
    }
    let failed = first.try_session_update(|game| {
        let player_id = game.session().state().player_id;
        game.set_trainer_identity("TEST".to_string(), player_id)?;
        anyhow::bail!("simulated persistence failure")
    });
    let failed: anyhow::Result<()> = failed;
    assert!(
        failed
            .unwrap_err()
            .to_string()
            .contains("simulated persistence failure")
    );
    assert_eq!(
        first, second,
        "failed updates restore state and command journals"
    );
    let catalog = first.runtime().audio();
    for programs in [catalog.music(), catalog.sound_effects(), catalog.cries()] {
        let (id, program) = programs.first_key_value().expect("pack audio category");
        let pcm = crystal_runtime::audio::pcm::decode_program_source(program.source.clone())
            .unwrap_or_else(|error| panic!("{id}: {error:#}"));
        assert!(!pcm.samples.is_empty());
        assert_eq!(pcm.samples.len() * 2, pcm.bytes.len());
        assert_eq!(pcm.format.channels, 2);
        assert_eq!(pcm.format.sample_rate_hz, 22_050);
    }
}
