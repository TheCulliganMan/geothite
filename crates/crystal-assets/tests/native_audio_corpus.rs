//! Verify the actual shipped programs without bundling a second source catalog.
use crystal_assets::{ModpackAudioKind, read_verified_compiled_game_pack};
use crystal_audio::synth::{SynthContext, decode_midi, render};

#[test]
fn bundled_audio_matches_native_pcm_and_music_loops() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/core-modular.browser.crystalpack")
        .canonicalize()
        .unwrap();
    let pack = read_verified_compiled_game_pack(&path).unwrap();
    let context = SynthContext::cartridge().unwrap();
    let mut music_count = 0;
    for asset in &pack.data().audio {
        let program = decode_midi(&asset.midi_program.as_ref().unwrap().midi_base64).unwrap();
        let rendered =
            render(&program, context.clone()).unwrap_or_else(|e| panic!("{}: {e:#}", asset.id));
        let pcm = rendered.downsample();
        let hash = pcm
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .fold(2166136261_u32, |hash, byte| {
                (hash ^ u32::from(byte)).wrapping_mul(16777619)
            });
        assert_eq!(
            Some(format!("{hash:08x}")),
            asset.payload_hash,
            "{} PCM",
            asset.id
        );
        assert_eq!(
            Some(pcm.len() / 2),
            asset.pcm_frame_count,
            "{} frames",
            asset.id
        );
        if asset.kind == ModpackAudioKind::Music {
            music_count += 1;
            let primary = program
                .music_data
                .channels
                .values()
                .filter_map(|c| c.number)
                .min()
                .unwrap();
            let start = rendered.loop_samples.get(&primary).map(|s| s.div_ceil(2));
            assert_eq!(start, asset.loop_start_sample, "{} loop start", asset.id);
            assert_eq!(
                start.map(|_| pcm.len() / 2),
                asset.loop_end_sample,
                "{} loop end",
                asset.id
            );
            if matches!(
                asset.id.as_str(),
                "MUSIC_JOHTO_WILD_BATTLE_NIGHT" | "MUSIC_RIVAL_AFTER"
            ) {
                assert!(start.is_some());
                assert!(rendered.channel_nonzero.values().all(|n| *n > 0));
                assert_eq!(
                    rendered.loop_samples.len(),
                    program.music_data.channels.len()
                );
                assert_eq!(
                    format!("{hash:08x}"),
                    if asset.id.ends_with("NIGHT") {
                        "d5be03ed"
                    } else {
                        "610d66a9"
                    }
                );
            }
        }
    }
    assert_eq!(music_count, 102);
    assert_eq!(pack.data().audio.len(), 1130);
}
