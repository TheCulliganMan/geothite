// Original anim_sound operands retained independently of canonical PCM assets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BattleSoundArgs {
    packed: u8,
}

impl BattleSoundArgs {
    fn new(duration: u8, tracks: u8) -> Self {
        Self {
            packed: duration.wrapping_shl(2) | tracks,
        }
    }

    fn tracks(self, player_move: bool) -> u8 {
        (self.packed ^ u8::from(!player_move)) & 3
    }

    // Crystal writes duration fields/flag for modes 2/3, but never reads
    // them. All four modes therefore use their initial left/right mask;
    // duration is neither a sweep clock nor a playback cutoff.
    fn initial_mask(self, player_move: bool) -> [bool; 2] {
        let right = self.tracks(player_move) & 1 != 0;
        [!right, right]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VisibleMoveSound {
    id: String,
    args: Option<BattleSoundArgs>,
}

impl From<&str> for VisibleMoveSound {
    fn from(id: &str) -> Self {
        Self {
            id: id.to_string(),
            args: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BattleSoundPlayback {
    args: BattleSoundArgs,
    player_move: bool,
}

fn queue_visible_battle_sound_effect(
    shell: &mut BevyRuntimeShell,
    sound: &VisibleMoveSound,
    player_move: bool,
) -> Result<()> {
    // The existing queue validates the catalog entry. Attach presentation
    // operands to that exact command; no second playback dispatch is needed.
    queue_visible_shell_sound_effect(shell, &sound.id)?;
    let command = shell
        .pending_audio
        .last_mut()
        .context("queued battle sound is missing")?;
    command.battle_sound = sound
        .args
        .map(|args| BattleSoundPlayback { args, player_move });
    Ok(())
}

// Source programs audited against the external Crystal pack: no channel
// panning, restart_channel or new_song can override the initial mask.
// This is a routing eligibility list, not another audio source catalog.
const AUDITED_BATTLE_SOUND_IDS: &[&str] = &[
    "SFX_BUBBLEBEAM",
    "SFX_EMBER",
    "SFX_GIGA_DRAIN",
    "SFX_HYDRO_PUMP",
    "SFX_LEER",
    "SFX_LICK",
    "SFX_MENU",
    "SFX_METRONOME",
    "SFX_PERISH_SONG",
    "SFX_PSYBEAM",
    "SFX_PSYCHIC",
    "SFX_SCREECH",
    "SFX_SHARPEN",
    "SFX_SHINE",
    "SFX_SING",
    "SFX_SLUDGE_BOMB",
    "SFX_SPIDER_WEB",
    "SFX_SUPERSONIC",
    "SFX_SURF",
    "SFX_THROW_BALL",
    "SFX_VINE_WHIP",
    "SFX_WATER_GUN",
    "SFX_WHIRLWIND",
    "SFX_ZAP_CANNON",
];

fn pcm_samples_for_audio_command(
    samples: &Arc<[i16]>,
    sound: Sound,
    command: &BevyAudioCommand,
) -> Arc<[i16]> {
    let output = pcm_samples_for_sound_option(samples, sound);
    if sound != Sound::Stereo || command.kind != ModpackAudioKind::SoundEffect {
        return output;
    }
    // These audited source programs remain bilateral with no subsequent
    // panning or channel restart. A post-cache mask therefore preserves the
    // canonical waveform. Other programs need their own source audit.
    if !AUDITED_BATTLE_SOUND_IDS.contains(&command.audio_id.as_str()) {
        return output;
    }
    let Some(playback) = command.battle_sound else {
        return output;
    };
    let mask = playback.args.initial_mask(playback.player_move);
    Arc::from(
        output
            .chunks_exact(2)
            .flat_map(|frame| {
                [
                    if mask[0] { frame[0] } else { 0 },
                    if mask[1] { frame[1] } else { 0 },
                ]
            })
            .collect::<Vec<_>>(),
    )
}

// Explicit pilot for the verified Surf program. Other moves keep their
// existing wrapper until their source-channel arbitration has been audited.
fn visible_surf_source_audio_wait(
    shell: &BevyRuntimeShell,
    animation: &VisibleMoveAnimation,
) -> Result<u16> {
    if animation.move_id != "SURF" || animation.animation_label != "BattleAnim_Surf" {
        return Ok(0);
    }
    anyhow::ensure!(
        !animation.sound_events.is_empty(),
        "Surf source sound timeline is empty"
    );
    anyhow::ensure!(
        animation.sound_events.iter().all(|(_, sound)| {
            sound.id == "SFX_SURF" && sound.args == Some(BattleSoundArgs::new(0, 1))
        }),
        "Surf sound wrapper contains unaudited cues or routing"
    );
    let sound = shell
        .shell
        .runtime()
        .audio()
        .require_sound_effect("SFX_SURF")?;
    let AudioProgramSource::Midi { midi_base64, .. } = &sound.source else {
        anyhow::bail!("Surf source-channel timing requires its bundled MIDI command program");
    };
    let program = crystal_audio::synth::decode_midi(midi_base64)?;
    let timing = crystal_audio::synth::source_channel_timing(&program)?;
    anyhow::ensure!(
        timing.looping_channels.is_empty(),
        "Surf WaitSFX cannot complete an infinite source loop"
    );
    anyhow::ensure!(
        timing.channel_frames.keys().copied().collect::<Vec<_>>() == [5, 6, 8],
        "Surf source channel ownership is unaudited"
    );
    let duration = *timing
        .channel_frames
        .values()
        .max()
        .context("Surf has no active source channels")?;
    let duration =
        u16::try_from(duration).context("Surf source busy duration exceeds presentation clock")?;
    let end = animation
        .sound_events
        .iter()
        .try_fold(0_u16, |_, (frame, _)| {
            // Repeated equal-priority Surf requests replace/restart the same
            // channels. They do not overlap four independent PCM voices.
            frame
                .checked_add(duration)
                .context("Surf source busy deadline overflow")
        })?;
    Ok(end.saturating_sub(animation.total_frames))
}

#[cfg(test)]
mod battle_sound_tests {
    use super::*;

    fn surf_command(player_move: bool, duration: u8, tracks: u8) -> BevyAudioCommand {
        BevyAudioCommand {
            battle_sound: Some(BattleSoundPlayback {
                args: BattleSoundArgs::new(duration, tracks),
                player_move,
            }),
            cry_parameters: None,
            audio_id: "SFX_SURF".into(),
            kind: ModpackAudioKind::SoundEffect,
            mode: ModpackAudioPlaybackMode::RawPcm,
            looped: false,
        }
    }

    #[test]
    fn surf_source_stereo_uses_actor_side_and_retains_canonical_pcm_and_cache_key() {
        let samples: Arc<[i16]> = Arc::from([120, 120, -90, -90, 300, 300]);
        let before = Arc::clone(&samples);
        let player = surf_command(true, 0, 1);
        let enemy = surf_command(false, 0, 1);
        assert_eq!(
            BevyAudioCacheKey::from_command(&player),
            BevyAudioCacheKey::from_command(&enemy)
        );
        let right = pcm_samples_for_audio_command(&samples, Sound::Stereo, &player);
        let left = pcm_samples_for_audio_command(&samples, Sound::Stereo, &enemy);
        assert_eq!(right.as_ref(), &[0, 120, 0, -90, 0, 300]);
        assert_eq!(left.as_ref(), &[120, 0, -90, 0, 300, 0]);
        assert_eq!(right.len(), samples.len(), "duration zero is not a cutoff");
        assert_eq!(left.len(), samples.len());
        assert_eq!(samples, before);
        assert!(Arc::ptr_eq(&samples, &before));
    }

    #[test]
    fn all_source_track_modes_use_the_original_actor_mask_at_every_duration() {
        // Original GetPanning table is f0, 0f, f0, 0f. Enemy XORs bit 0.
        for duration in 0..64 {
            for tracks in 0..4 {
                let args = BattleSoundArgs::new(duration, tracks);
                assert_eq!(args.packed >> 2, duration);
                let right = tracks == 1 || tracks == 3;
                assert_eq!(args.initial_mask(true), [!right, right]);
                assert_eq!(args.initial_mask(false), [right, !right]);
            }
        }
    }

    #[test]
    fn audited_battle_pcm_keeps_mono_cache_identity_and_every_selected_sample() {
        let samples: Arc<[i16]> = Arc::from([i16::MIN, 123, -90, i16::MAX, 300, -25]);
        let before = samples.to_vec();
        for &audio_id in AUDITED_BATTLE_SOUND_IDS {
            for player_move in [true, false] {
                for duration in [0, 6, 63] {
                    for tracks in 0..4 {
                        let mut command = surf_command(player_move, duration, tracks);
                        command.audio_id = audio_id.into();
                        assert_eq!(
                            pcm_samples_for_audio_command(&samples, Sound::Mono, &command),
                            pcm_samples_for_sound_option(&samples, Sound::Mono)
                        );
                        let mut other_side = command.clone();
                        other_side.battle_sound.as_mut().unwrap().player_move = !player_move;
                        assert_eq!(
                            BevyAudioCacheKey::from_command(&command),
                            BevyAudioCacheKey::from_command(&other_side)
                        );
                        let masked =
                            pcm_samples_for_audio_command(&samples, Sound::Stereo, &command);
                        let right = ((tracks & 1) != 0) == player_move;
                        let expected: Vec<i16> = samples
                            .chunks_exact(2)
                            .flat_map(|frame| if right { [0, frame[1]] } else { [frame[0], 0] })
                            .collect();
                        assert_eq!(masked.as_ref(), expected.as_slice());
                        assert_eq!(masked.len(), samples.len(), "duration cannot cut off PCM");
                        assert_eq!(samples.as_ref(), before.as_slice());
                    }
                }
            }
        }
    }

    #[test]
    fn unannotated_unreviewed_and_non_sfx_audio_keep_the_existing_stereo_path() {
        let samples: Arc<[i16]> = Arc::from([120, -80, -40, 100]);
        let mut ordinary = surf_command(true, 0, 1);
        ordinary.battle_sound = None;
        assert!(Arc::ptr_eq(
            &samples,
            &pcm_samples_for_audio_command(&samples, Sound::Stereo, &ordinary)
        ));
        ordinary.battle_sound = Some(BattleSoundPlayback {
            args: BattleSoundArgs::new(6, 2),
            player_move: true,
        });
        ordinary.audio_id = "SFX_DOUBLE_KICK".into();
        assert!(Arc::ptr_eq(
            &samples,
            &pcm_samples_for_audio_command(&samples, Sound::Stereo, &ordinary)
        ));
        ordinary.audio_id = "SFX_PSYCHIC".into();
        ordinary.kind = ModpackAudioKind::Music;
        assert!(Arc::ptr_eq(
            &samples,
            &pcm_samples_for_audio_command(&samples, Sound::Stereo, &ordinary)
        ));
    }
}
