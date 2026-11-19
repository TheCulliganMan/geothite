use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_audio::synth::{self, SynthContext};
use crystal_core::multiplayer::fnv1a32_hex_bytes;
use crystal_core::save;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::script_audio::{ScriptAudioCue, ScriptAudioKind};
use crystal_core::world::map::{Direction, TilePosition};

#[test]
#[ignore = "requires the external archive in MN_TOWN_RETURNS_PACK"]
fn clinic_return_loads_matching_actor_cry_and_restored_room() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("MN_TOWN_RETURNS_PACK").context("town archive")?);
    let loaded = crate::read_loaded_verified_compiled_game_pack(path)?;
    let pack = loaded.pack();
    let data = pack.data();
    let music = pack.audio_manifest.music.keys().cloned().collect();
    let cries = pack.audio_manifest.cries.keys().cloned().collect();
    const CLINIC: &str = "MnRochesterPokecenter1F";
    const ACTOR: &str = "MN_ARC_GILLPUP";
    for (name, ivy_left, restored) in [
        ("clinic-before", false, false),
        ("clinic-during", true, false),
        ("clinic-restored", true, true),
    ] {
        let (mut state, mut session) = data.start_overworld_session_at_runtime_tile(
            "MnRochester",
            TilePosition::new(15, 8),
            &music,
        )?;
        for (flag, value) in [
            ("EVENT_MN_LIGHTHOUSE_SIGNAL", ivy_left),
            ("EVENT_MN_REED_JOINED_ICE", ivy_left),
            ("EVENT_MN_SURVEY_RESTORED", restored),
        ] {
            state
                .flags
                .set_event_flag(flag, value)
                .map_err(|err| anyhow::anyhow!("clinic flag {flag}: {err:?}"))?;
        }
        data.transition_overworld_session(
            &mut state,
            &mut session,
            CLINIC,
            TilePosition::new(2, 6),
            SpawnMemoryUpdate::Preserve,
            &music,
        )?;
        session.player.facing = Direction::Left;
        let creature = session.visible_object_at(TilePosition::new(1, 6));
        let facing = session
            .check_interaction_checked(1)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        ensure!(
            session
                .visible_object_at(TilePosition::new(8, 6))
                .is_some_and(|(_, o)| o.script == "MN_RC_PATIENT"),
            "aide is missing"
        );

        ensure!(
            session
                .visible_object_at(TilePosition::new(2, 3))
                .is_some_and(|(_, o)| o.script == "MN_ROCHESTER_RECIPIENT")
                != ivy_left,
            "Ivy's clinic presence disagrees with her departure"
        );
        // A native render can resume each independently loaded state.
        if let Some(dir) = std::env::var_os("MN_TOWN_RETURNS_RENDER_DIR") {
            let save_path = PathBuf::from(dir).join(format!("{name}.crystalsave"));
            ensure!(!save_path.try_exists()?, "choose a fresh render directory");
            data.commit_overworld_snapshot(&mut state, &session, SpawnMemoryUpdate::Preserve);
            save::write_save_game_for_modpack(
                save_path,
                state.clone(),
                &loaded.save_modpack_identity()?,
                &loaded.pack_content_hash()?,
            )?;
        }
        if restored {
            ensure!(
                creature.is_none() && facing.is_none(),
                "absent Loobit can still be seen or heard"
            );
            continue;
        }
        ensure!(
            creature.is_some_and(|(_, o)| o.sprite == "SPRITE_LOOBIT" && o.script == ACTOR)
                && facing.as_ref().is_some_and(|hit| hit.script == ACTOR),
            "incorrect basin interaction"
        );
        let cue = data.apply_script_audio_command_in_session(
            &mut state,
            &session,
            CLINIC,
            ACTOR,
            0,
            &music,
            &BTreeSet::new(),
            &cries,
        )?;
        let ScriptAudioCue::Play {
            kind: ScriptAudioKind::Cry,
            audio_id,
            ..
        } = cue
        else {
            anyhow::bail!("the basin did not resolve to a creature cry");
        };
        ensure!(audio_id == "CRY_MN_040", "wrong Loobit cry {audio_id}");
        let asset = data
            .audio
            .iter()
            .find(|asset| asset.id == audio_id)
            .context("Loobit audio")?;
        let program = &asset
            .midi_program
            .as_ref()
            .context("embedded cry")?
            .midi_base64;
        let pcm = synth::render(&synth::decode_midi(program)?, SynthContext::cartridge()?)?;
        let samples = pcm.downsample();
        let bytes: Vec<_> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        ensure!(
            pcm.loop_samples.is_empty()
                && asset.loop_start_sample.is_none()
                && asset.loop_end_sample.is_none()
                && samples.len() / 2 == 20305
                && asset.pcm_frame_count == Some(20305)
                && asset.payload_hash.as_deref() == Some("2c960fb9")
                && fnv1a32_hex_bytes(&bytes) == "2c960fb9",
            "Loobit cry differs from its original recording"
        );
    }
    Ok(())
}
