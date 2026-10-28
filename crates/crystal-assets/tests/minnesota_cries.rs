//! Full-roster sound, runtime and preservation checks using external packs.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crystal_audio::original_cry::{CryDesignBook, CryScore};
use crystal_audio::pcm::{self, CrySynthesisParameters, ModifiedCryRequest};

#[test]
fn authored_roster_has_prompt_and_structurally_distinct_voices() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
    let book: CryDesignBook =
        serde_json::from_slice(&std::fs::read(root.join("cry-designs.json")).expect("book"))
            .expect("authored designs");
    let roster: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("roster.json")).expect("roster"))
            .expect("roster JSON");
    let programs = book.programs().expect("valid scores");
    let mut structures = BTreeMap::new();
    for family in roster["families"].as_array().expect("families") {
        let id = family["id"].as_str().expect("family id");
        let Some(design) = book.families.get(id) else {
            continue;
        };
        let mut previous = None;
        for (stage, member) in family["members"]
            .as_array()
            .expect("members")
            .iter()
            .enumerate()
        {
            let species = member["id"].as_str().expect("species id");
            let variant = &design.species[species];
            assert!(
                stage == 0 || variant.score.is_some(),
                "{species} needs its own evolved score"
            );
            let score = variant.score.as_ref().unwrap_or(&design.score);
            let structure = phrase_structure(score);
            assert!(
                previous.as_ref() != Some(&structure),
                "{species} only changes pitch"
            );
            previous = Some(structure.clone());
            if let Some(other) = structures.insert(structure, id) {
                assert_eq!(
                    id, other,
                    "unrelated families share articulation: {id}, {other}"
                );
            }
            let rendered = crystal_audio::synth::render(
                &programs[species],
                crystal_audio::synth::SynthContext::cartridge().expect("context"),
            )
            .expect("render authored cry");
            assert!(rendered.loop_samples.is_empty());
            let pcm = rendered.downsample();
            let frames = pcm.len() / 2;
            assert!(
                (3307..=99225).contains(&frames),
                "{species} must last 0.15..=4.5s: {frames}"
            );
            let onset = pcm
                .as_chunks::<2>()
                .0
                .iter()
                .position(|frame| frame.iter().any(|v| i32::from(*v).abs() >= 512))
                .expect("audible voice");
            assert!(
                onset <= 220,
                "{species} attack exceeds 10ms: {onset} frames"
            );
            let attack_rms = (pcm[..882]
                .iter()
                .map(|v| f64::from(*v).powi(2))
                .sum::<f64>()
                / 882.0)
                .sqrt();
            assert!(
                attack_rms >= 400.0,
                "{species} has a weak first 20ms: {attack_rms}"
            );
            assert!(pcm.iter().all(|v| i32::from(*v).abs() <= 24576));
            assert!(
                pcm[pcm.len() - 2..]
                    .iter()
                    .all(|v| i32::from(*v).abs() <= 16)
            );
        }
    }
}

#[test]
fn long_roars_have_a_voiced_middle_and_independently_articulated_parts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
    let book: CryDesignBook =
        serde_json::from_slice(&std::fs::read(root.join("cry-designs.json")).expect("book"))
            .expect("authored designs");
    let programs = book.programs().expect("valid scores");
    for id in ["MN_006", "MN_153", "MN_276"] {
        let program = &programs[id];
        let pcm = crystal_audio::synth::render(
            program,
            crystal_audio::synth::SynthContext::cartridge().expect("context"),
        )
        .expect("render long cry")
        .downsample();
        assert!(
            pcm.len() / 2 >= 26460,
            "{id} must perform for at least 1.2 seconds"
        );
        for ms in [200, 500, 900] {
            let start = ms * 44100 / 1000;
            let energy = pcm[start..start + 882]
                .iter()
                .map(|v| f64::from(*v).powi(2))
                .sum::<f64>()
                / 882.0;
            assert!(energy.sqrt() >= 400.0, "{id} pads its middle at {ms}ms");
        }
        // A connected contour compiles to per-frame register writes. Those
        // writes are not new syllables: compare the authored gesture boundaries.
        let score = book
            .families
            .values()
            .find_map(|family| {
                family
                    .species
                    .get(id)
                    .map(|variant| variant.score.as_ref().unwrap_or(&family.score))
            })
            .expect("scored species");
        let boundaries = |part: &crystal_audio::original_cry::PulsePart| {
            let mut frame = usize::from(part.delay);
            let mut starts = BTreeSet::new();
            for &[frames, _, volume, _] in &part.notes {
                if volume > 0 {
                    starts.insert(frame);
                }
                frame += frames as usize;
            }
            starts
        };
        let lead = boundaries(score.pulse1.as_ref().expect("main throat"));
        let companion = boundaries(score.pulse2.as_ref().expect("second throat"));
        assert!(
            lead.len() >= 4 && companion.len() >= 3,
            "{id} lacks internal development"
        );
        assert!(
            lead.intersection(&companion).count() * 2 < lead.union(&companion).count(),
            "{id} simply doubles the main articulation"
        );
    }
}

#[test]
fn rare_adults_develop_audibly_through_the_extended_performance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
    let book: CryDesignBook =
        serde_json::from_slice(&std::fs::read(root.join("cry-designs.json")).expect("book"))
            .expect("authored designs");
    let programs = book.programs().expect("valid scores");
    for id in [
        "MN_095", "MN_096", "MN_097", "MN_098", "MN_099", "MN_100", "MN_153", "MN_248", "MN_250",
    ] {
        let pcm = crystal_audio::synth::render(
            &programs[id],
            crystal_audio::synth::SynthContext::cartridge().expect("context"),
        )
        .expect("render rare adult")
        .downsample();
        let frames = pcm.len() / 2;
        assert!((52_920..=99_225).contains(&frames), "{id}: {frames} frames");
        // Reject an extended file made by padding a short call with silence.
        // Sample 100ms in each phase, including the late resolution.
        for percent in [10, 35, 60, 85] {
            let start = frames * percent / 100 * 2;
            let energy = pcm[start..start + 4410]
                .iter()
                .map(|sample| f64::from(*sample).powi(2))
                .sum::<f64>()
                / 4410.0;
            assert!(energy.sqrt() >= 400.0, "{id}: silent padding at {percent}%");
        }
    }
}

// Compare articulation and orchestration while deliberately discarding every
// pulse pitch and noise register. Pitch-only variations cannot pass this check.
fn phrase_structure(score: &CryScore) -> Vec<i32> {
    let mut result = Vec::new();
    for part in [&score.pulse1, &score.pulse2] {
        if let Some(part) = part {
            result.push(i32::from(part.delay));
            result.extend(part.duty.map(i32::from));
            for &[frames, _, volume, envelope] in &part.notes {
                result.extend([frames, volume, envelope]);
            }
            result.push(-101);
            for (index, [_, volume]) in &part.contours {
                result.extend([*index as i32, *volume]);
            }
            result.push(-102);
            for (index, duties) in &part.duties {
                result.push(*index as i32);
                result.extend(duties.map(i32::from));
            }
        }
        result.push(-100);
    }
    if let Some(part) = &score.noise {
        result.push(i32::from(part.delay));
        for &[frames, _, volume, envelope] in &part.notes {
            result.extend([frames, volume, envelope]);
        }
    }
    result
}

#[test]
#[ignore = "requires CRYSTAL_MN_CRY_BASE, CRYSTAL_MN_CRY_PACK and CRYSTAL_MN_CRY_WAV_DIR"]
fn original_cries_cover_roster_play_in_runtime_and_preserve_other_content() {
    let base_path = std::env::var_os("CRYSTAL_MN_CRY_BASE").expect("base pack");
    let pack_path = std::env::var_os("CRYSTAL_MN_CRY_PACK").expect("authored pack");
    let wav_directory = std::env::var_os("CRYSTAL_MN_CRY_WAV_DIR").expect("external WAV gallery");
    let base =
        crystal_assets::read_verified_compiled_game_pack(Path::new(&base_path)).expect("base");
    let pack = crystal_assets::read_verified_compiled_game_pack(Path::new(&pack_path))
        .expect("published pack");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
    let book: CryDesignBook =
        serde_json::from_slice(&std::fs::read(root.join("cry-designs.json")).expect("book file"))
            .expect("book");
    let curation: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("roster-curation.json")).expect("curation file"),
    )
    .expect("curation");
    let expected = curation["active_species_count"]
        .as_u64()
        .expect("species count") as usize;
    assert_eq!(
        book.families.len(),
        curation["active_family_count"].as_u64().expect("families") as usize
    );
    let programs = book.programs().expect("complete book");
    assert_eq!(programs.len(), expected);
    let assets = pack
        .data()
        .audio
        .iter()
        .filter(|asset| programs.contains_key(asset.id.strip_prefix("CRY_").unwrap_or("")))
        .collect::<Vec<_>>();
    assert_eq!(assets.len(), expected);
    assert_eq!(
        assets
            .iter()
            .map(|asset| &asset.payload_hash)
            .collect::<BTreeSet<_>>()
            .len(),
        expected
    );
    let replaced = base
        .data()
        .audio
        .iter()
        .filter(|asset| programs.contains_key(asset.id.strip_prefix("CRY_").unwrap_or("")))
        .count();
    assert_eq!(
        pack.data().audio.len(),
        base.data().audio.len() + expected - replaced
    );
    for (species_id, program) in &programs {
        let id = format!("CRY_{species_id}");
        let asset = assets
            .iter()
            .find(|asset| asset.id == id)
            .expect("every scored species published");
        let (token, meta) = pack
            .data()
            .pokemon_cries
            .iter()
            .find(|(_, meta)| meta.cry == id)
            .expect("runtime species reference");
        assert!(pack.data().pokemon.contains_key(token));
        assert_eq!(
            (asset.loop_start_sample, asset.loop_end_sample),
            (None, None)
        );
        assert!(program.output_envelope.is_some());
        let midi = &asset
            .midi_program
            .as_ref()
            .expect("MIDI program")
            .midi_base64;
        let mut request = ModifiedCryRequest {
            format: crystal_audio::AudioPcmFormat {
                sample_rate_hz: 22050,
                channels: 2,
                bits_per_sample: 16,
            },
            byte_len: asset.pcm_frame_count.expect("frame count") * 4,
            payload_hash: asset.payload_hash.clone().expect("hash"),
            parameters: CrySynthesisParameters {
                pitch: meta.pitch as u16,
                length: meta.length as u16,
            },
        };
        let decoded =
            pcm::decode_modified_cry(midi, &request).expect("runtime variant and integrity path");
        let wav = std::fs::read(Path::new(&wav_directory).join(format!("{species_id}.wav")))
            .expect("listening WAV");
        assert!(wav.starts_with(b"RIFF") && &wav[8..16] == b"WAVEfmt ");
        assert_eq!(&wav[22..24], &[2, 0]);
        assert_eq!(&wav[24..28], &22050_u32.to_le_bytes());
        assert_eq!(&wav[34..36], &[16, 0]);
        assert_eq!(
            &wav[44..],
            &*decoded.bytes,
            "gallery and runtime PCM agree for {id}"
        );
        assert_eq!(
            decoded.samples.len() / 2,
            asset.pcm_frame_count.expect("frames")
        );
        assert!(
            decoded
                .samples
                .iter()
                .any(|sample| i32::from(*sample).abs() > 1000)
        );
        assert!(
            decoded
                .samples
                .iter()
                .all(|sample| i32::from(*sample).abs() <= 24576)
        );
        assert!(
            decoded.samples[decoded.samples.len() - 2..]
                .iter()
                .all(|sample| i32::from(*sample).abs() <= 16)
        );
        let canonical = crystal_audio::synth::render(
            program,
            crystal_audio::synth::SynthContext::cartridge().expect("context"),
        )
        .expect("canonical score")
        .downsample();
        assert_eq!(
            &*decoded.samples, canonical,
            "runtime matches authored score {id}"
        );
        if Some(species_id) == programs.keys().next() {
            request.parameters = CrySynthesisParameters::slow(meta.pitch, meta.length);
            let slow = pcm::decode_modified_cry(midi, &request).expect("runtime slow cry variant");
            assert_ne!(&*slow.samples, canonical);
            assert!(
                slow.samples
                    .iter()
                    .any(|sample| i32::from(*sample).abs() > 1000)
            );
            assert!(
                slow.samples[slow.samples.len() - 2..]
                    .iter()
                    .all(|sample| i32::from(*sample).abs() <= 16)
            );
        }
        request.payload_hash = "00000000".into();
        assert!(
            pcm::decode_modified_cry(midi, &request).is_err(),
            "integrity rejection {id}"
        );
    }
    for original in &base.data().audio {
        if programs.contains_key(original.id.strip_prefix("CRY_").unwrap_or("")) {
            continue;
        }
        assert_eq!(
            pack.data()
                .audio
                .iter()
                .find(|asset| asset.id == original.id),
            Some(original),
            "preserve source audio {}",
            original.id
        );
    }
    for (species, original) in &base.data().pokemon_cries {
        if !programs.contains_key(
            pack.data().pokemon_cries[species]
                .cry
                .strip_prefix("CRY_")
                .unwrap_or(""),
        ) {
            assert_eq!(&pack.data().pokemon_cries[species], original);
        }
    }
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    let mut before = serde_json::to_value(base.data()).expect("base data");
    let mut after = serde_json::to_value(pack.data()).expect("authored data");
    for value in [&mut before, &mut after] {
        let data = value.as_object_mut().expect("game data object");
        data.remove("audio");
        data.remove("pokemon_cries");
    }
    assert_eq!(
        before, after,
        "every unrelated game-data field must be preserved"
    );
    let rebuilt = crystal_assets::build_minnesota_cries(&pack, &root).expect("idempotent rebuild");
    assert_eq!(
        rebuilt.identity().expect("identity"),
        pack.identity().expect("published identity")
    );
    println!(
        "verified {expected} distinct original voices, runtime decoding, integrity rejection and complete preservation"
    );
}
