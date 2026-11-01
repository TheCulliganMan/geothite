//! Original cry scores: independent pulse and noise performances, not source dumps.
use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::synth::{MusicData, OutputEnvelope, Program};
use crate::{AudioCommand, AudioSource};

/// A finite original voice. Absent parts do not consume a sound channel.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryScore {
    pub pulse1: Option<PulsePart>,
    pub pulse2: Option<PulsePart>,
    pub noise: Option<NoisePart>,
}

/// One independently articulated tonal voice.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PulsePart {
    /// Delay before the voice enters, in video frames.
    pub delay: u16,
    /// Hardware duty settings. A constant array holds the duty; otherwise cycle it.
    pub duty: [u8; 4],
    /// Each entry is [video frames, frequency Hz, volume, envelope pace].
    /// Zero volume is a gap; its frequency may be zero. Positive envelope pace
    /// decays, negative pace grows, and zero holds. Paces range from -7 to 7.
    pub notes: Vec<[i32; 4]>,
    /// Note-indexed [end Hz, end volume] contours. These move in register space
    /// at score-frame resolution without restarting a decaying envelope. The
    /// corresponding note must have at least two frames and envelope pace zero.
    /// This is original composition, not emulation of the hardware sweep unit.
    #[serde(default)]
    pub contours: BTreeMap<usize, [i32; 2]>,
    /// Note-indexed mouth/timbre changes, applied before that note begins.
    #[serde(default)]
    pub duties: BTreeMap<usize, [u8; 4]>,
}

/// Independent breath, rasp or periodic buzz; it need not follow either pulse.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoisePart {
    pub delay: u16,
    /// Each entry is [video frames, NR43 noise register, volume, envelope pace].
    /// Registers select LFSR clock and width, not a frequency in Hz.
    pub notes: Vec<[i32; 4]>,
}

/// Authored families and their explicit species variants, independent of a pack.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryDesignBook {
    pub schema_version: u16,
    pub families: BTreeMap<String, CryFamilyDesign>,
}

/// Shared vocal ancestry, with optional complete replacements for metamorphosis.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryFamilyDesign {
    pub intent: String,
    pub score: CryScore,
    pub species: BTreeMap<String, CryVariant>,
}

/// Explicit register words and vocal intent for one species.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryVariant {
    pub pitch: i16,
    pub length: u16,
    pub character: String,
    pub score: Option<CryScore>,
}

impl CryDesignBook {
    /// Compile each species exactly once. Returns errors for an unsupported
    /// schema, missing intent, duplicate species, or invalid original scores.
    pub fn programs(&self) -> Result<BTreeMap<String, Program>> {
        ensure!(
            self.schema_version == 1 && !self.families.is_empty(),
            "invalid cry design book"
        );
        let mut programs = BTreeMap::new();
        for family in self.families.values() {
            ensure!(
                !family.intent.trim().is_empty() && !family.species.is_empty(),
                "family lacks vocal intent or species"
            );
            for (id, variant) in &family.species {
                ensure!(
                    !variant.character.trim().is_empty(),
                    "species {id} lacks vocal character"
                );
                let score = variant.score.as_ref().unwrap_or(&family.score);
                let program = score.program(variant.pitch, variant.length)?;
                ensure!(
                    programs.insert(id.clone(), program).is_none(),
                    "duplicate cry species {id}"
                );
            }
        }
        Ok(programs)
    }
}

impl CryScore {
    /// Compile an authored score into the existing finite cartridge program.
    ///
    /// Pitch and length are explicit species register words; length affects only
    /// the tonal parts. Noise texture is authored directly because the current
    /// PCM renderer does not apply the species pitch offset to noise.
    ///
    /// Returns errors for an empty score, invalid or out-of-range note controls,
    /// contours on gaps or decaying notes, frequencies outside 70..=6000 Hz,
    /// excessive delays, or more than 240 score frames per part. The extended
    /// budget permits developed rare-species calls without stretching every
    /// gesture through the species length register.
    /// Pitch arithmetic follows the existing renderer's 11-bit register wrapping.
    pub fn program(&self, pitch: i16, length: u16) -> Result<Program> {
        ensure!(
            (64..=640).contains(&length),
            "cry length register outside 64..=640"
        );
        let mut channels = BTreeMap::new();
        for (number, part) in [(5, &self.pulse1), (6, &self.pulse2)] {
            let Some(part) = part else { continue };
            validate_notes(part.delay, &part.notes)?;
            ensure!(
                part.duty.iter().all(|duty| *duty <= 3),
                "invalid pulse duty"
            );
            let mut commands = Vec::new();
            if part.duty.iter().all(|duty| *duty == part.duty[0]) {
                commands.push(command("duty_cycle", [i32::from(part.duty[0])]));
            } else {
                commands.push(command("duty_cycle_pattern", part.duty.map(i32::from)));
            }
            silence(&mut commands, "square_note", i32::from(part.delay));
            ensure!(
                part.contours
                    .keys()
                    .chain(part.duties.keys())
                    .all(|index| *index < part.notes.len()),
                "pulse control refers to a missing note"
            );
            for (index, &[frames, hz, volume, fade]) in part.notes.iter().enumerate() {
                if let Some(duties) = part.duties.get(&index) {
                    ensure!(
                        duties.iter().all(|duty| *duty <= 3),
                        "invalid pulse duty change"
                    );
                    // Even a constant pattern replaces any earlier cycling
                    // pattern, on both the renderer and source-clock paths.
                    commands.push(command("duty_cycle_pattern", duties.map(i32::from)));
                }
                ensure!(
                    volume == 0 || (70..=6000).contains(&hz),
                    "invalid pulse frequency {hz}"
                );
                let register = if volume == 0 {
                    0
                } else {
                    frequency_register(hz)
                };
                if let Some(&[end_hz, end_volume]) = part.contours.get(&index) {
                    ensure!(
                        frames >= 2
                            && volume > 0
                            && fade == 0
                            && (70..=6000).contains(&end_hz)
                            && (0..=15).contains(&end_volume),
                        "invalid connected pulse contour"
                    );
                    let target = frequency_register(end_hz);
                    for frame in 0..frames {
                        let interpolate = |start, end| start + (end - start) * frame / (frames - 1);
                        commands.push(command(
                            "square_note",
                            [
                                0,
                                interpolate(volume, end_volume),
                                0,
                                interpolate(register, target),
                            ],
                        ));
                    }
                } else {
                    commands.push(command("square_note", [frames - 1, volume, fade, register]));
                }
            }
            finish(&mut commands, "square_note");
            channels.insert(
                format!("OriginalCry_Pulse{number}"),
                AudioSource {
                    number: Some(number),
                    commands,
                },
            );
        }
        if let Some(part) = &self.noise {
            validate_notes(part.delay, &part.notes)?;
            let mut commands = Vec::new();
            silence(&mut commands, "noise_note", i32::from(part.delay));
            for &[frames, register, volume, fade] in &part.notes {
                ensure!(
                    (0..=223).contains(&register),
                    "invalid or stopped noise clock {register}"
                );
                commands.push(command("noise_note", [frames - 1, volume, fade, register]));
            }
            finish(&mut commands, "noise_note");
            channels.insert(
                "OriginalCry_Noise".into(),
                AudioSource {
                    number: Some(8),
                    commands,
                },
            );
        }
        ensure!(!channels.is_empty(), "cry score has no parts");
        Ok(Program {
            profile: "pokecrystal-midi-v1".into(),
            cry_pitch: Some(i64::from(pitch)),
            cry_length: Some(i64::from(length)),
            output_envelope: Some(OutputEnvelope {
                attack_ms: 2,
                release_ms: 45,
                gain_percent: 75,
            }),
            music_data: MusicData {
                channel_count: u8::try_from(channels.len())?,
                channels,
                subroutines: BTreeMap::new(),
                shared_sources: BTreeMap::new(),
            },
        })
    }
}

fn frequency_register(hz: i32) -> i32 {
    (2048.0 - 131072.0 / f64::from(hz)).round() as i32
}

fn validate_notes(delay: u16, notes: &[[i32; 4]]) -> Result<()> {
    ensure!(
        delay <= 60 && !notes.is_empty() && notes.len() <= 96,
        "invalid cry part length"
    );
    let mut total = i32::from(delay);
    let mut audible = false;
    for &[frames, _, volume, fade] in notes {
        ensure!((1..=60).contains(&frames), "invalid cry note duration");
        ensure!((0..=15).contains(&volume), "invalid cry volume");
        ensure!((-7..=7).contains(&fade), "invalid cry envelope");
        total += frames;
        audible |= volume > 0;
    }
    ensure!(
        total <= 240 && audible,
        "cry part is too long or entirely silent"
    );
    Ok(())
}

fn command<const N: usize>(name: &str, args: [i32; N]) -> AudioCommand {
    AudioCommand {
        command: name.into(),
        args: args.into_iter().map(|value| value.to_string()).collect(),
    }
}

fn silence(commands: &mut Vec<AudioCommand>, name: &str, frames: i32) {
    if frames > 0 {
        commands.push(command(name, [frames - 1, 0, 0, 0]));
    }
}

fn finish(commands: &mut Vec<AudioCommand>, name: &str) {
    // Release generators before ending. Output filters may retain a DC transient.
    silence(commands, name, 3);
    commands.push(command("sound_ret", []));
}

#[cfg(test)]
mod tests {
    use super::{CryScore, NoisePart, PulsePart};
    use crate::synth::{self, SourceAudioClock, SynthContext};

    #[test]
    fn independent_parts_keep_their_timing_and_runtime_registers() {
        let score = CryScore {
            pulse1: Some(PulsePart {
                delay: 0,
                duty: [2; 4],
                notes: vec![[4, 390, 12, 1], [7, 450, 10, 2]],
                contours: Default::default(),
                duties: Default::default(),
            }),
            pulse2: Some(PulsePart {
                delay: 3,
                duty: [0, 1, 0, 2],
                notes: vec![[3, 810, 7, 0], [9, 730, 9, -2]],
                contours: Default::default(),
                duties: Default::default(),
            }),
            noise: Some(NoisePart {
                delay: 1,
                notes: vec![[2, 53, 9, 1], [12, 72, 5, 2]],
            }),
        };
        let program = score.program(-20, 320).expect("valid score");
        let midi = synth::encode_midi(&program).expect("encode");
        assert_eq!(synth::decode_midi(&midi).expect("decode"), program);
        let mut clock = SourceAudioClock::new().expect("clock");
        clock.play(&program).expect("play");
        let mut first = std::collections::BTreeMap::new();
        for frame in 0..64 {
            for step in clock.update_vblank().expect("advance") {
                if step.commands.iter().any(|command| {
                    matches!(command.command.as_str(), "square_note" | "noise_note")
                        && command.args[1] != "0"
                }) {
                    first.entry(step.after.channel).or_insert(frame);
                }
            }
        }
        assert_eq!(first[&5], 0);
        assert_eq!(first[&6], 3); // floor(3 * 320 / 256)
        assert_eq!(first[&8], 1); // noise is not scaled by cry length
        let rendered =
            synth::render(&program, SynthContext::cartridge().expect("context")).expect("render");
        assert!(rendered.loop_samples.is_empty());
        assert!(rendered.channel_nonzero.values().all(|count| *count > 0));
    }

    #[test]
    fn invalid_and_silent_scores_are_rejected() {
        let mut score = CryScore {
            pulse1: None,
            pulse2: None,
            noise: None,
        };
        assert!(score.program(0, 256).is_err());
        score.pulse1 = Some(PulsePart {
            delay: 0,
            duty: [2; 4],
            notes: vec![[5, 0, 10, 1]],
            contours: Default::default(),
            duties: Default::default(),
        });
        assert!(score.program(0, 256).is_err());
        score.pulse1.as_mut().expect("part").notes = vec![[5, 400, 0, 1]];
        assert!(score.program(0, 256).is_err());
    }

    #[test]
    fn extended_performance_is_finite_and_keeps_its_frame_budget() {
        let mut score = CryScore {
            pulse1: Some(PulsePart {
                delay: 0,
                duty: [2; 4],
                notes: vec![[60, 180, 10, 0]; 4],
                contours: Default::default(),
                duties: Default::default(),
            }),
            pulse2: None,
            noise: None,
        };
        let program = score.program(0, 256).expect("240-frame performance");
        let encoded = synth::encode_midi(&program).expect("encode");
        let decoded = synth::decode_midi(&encoded).expect("runtime decode");
        assert_eq!(decoded, program);
        let rendered = synth::render(&decoded, SynthContext::cartridge().expect("context"))
            .expect("render extended performance");
        assert!(rendered.loop_samples.is_empty());
        let pcm = rendered.downsample();
        assert!((88_200..=99_225).contains(&(pcm.len() / 2)));
        assert!(
            pcm[176_400..177_282]
                .iter()
                .any(|sample| sample.abs() > 1000)
        );
        assert!(pcm[pcm.len() - 2..].iter().all(|sample| sample.abs() <= 16));
        score
            .pulse1
            .as_mut()
            .expect("part")
            .notes
            .push([1, 180, 10, 0]);
        assert!(
            score.program(0, 256).is_err(),
            "241 frames exceed the budget"
        );
    }

    #[test]
    fn contours_keep_level_connected_and_round_trip_through_runtime() {
        let score: CryScore = serde_json::from_str(
            r#"{
            "pulse1": {"delay": 0, "duty": [0,2,0,2],
                "notes": [[9,400,12,0],[6,800,6,0]],
                "contours": {"0":[800,6],"1":[200,1]},
                "duties": {"1":[2,2,2,2]}},
            "pulse2": null, "noise": null
        }"#,
        )
        .expect("original contour score");
        let program = score.program(0, 256).expect("compile");
        let commands = &program.music_data.channels["OriginalCry_Pulse5"].commands;
        let notes = commands
            .iter()
            .filter(|c| c.command == "square_note")
            .collect::<Vec<_>>();
        assert_eq!(notes.len(), 16); // Fifteen contour frames and final silence.
        assert_eq!(&notes[0].args[..3], &["0", "12", "0"]);
        assert_eq!(
            notes[8].args, notes[9].args,
            "the join preserves pitch and level"
        );
        assert_eq!(notes[14].args[1], "1");
        let encoded = synth::encode_midi(&program).expect("encode");
        let decoded = synth::decode_midi(&encoded).expect("runtime decode");
        assert_eq!(decoded, program);
        let mut clock = SourceAudioClock::new().expect("clock");
        clock.play(&decoded).expect("play");
        for frame in 0..15 {
            let steps = clock.update_vblank().expect("advance");
            let state = &steps.last().expect("active pulse").after;
            assert_eq!(state.envelope & 7, 0, "held level at {frame}");
        }
        let pcm = synth::render(&decoded, SynthContext::cartridge().expect("context"))
            .expect("render")
            .downsample();
        assert!(pcm.iter().any(|sample| sample.abs() > 1000));
        assert!(pcm[pcm.len() - 2..].iter().all(|sample| sample.abs() <= 16));
    }

    #[test]
    fn invalid_contour_controls_are_rejected() {
        for controls in [
            r#""contours":{"1":[900,8]}"#,
            r#""contours":{"0":[60,8]}"#,
            r#""contours":{"0":[900,16]}"#,
            r#""duties":{"0":[0,1,2,4]}"#,
            r#""duties":{"1":[0,1,2,3]}"#,
        ] {
            let json = format!(
                r#"{{"pulse1":{{"delay":0,"duty":[2,2,2,2],
                "notes":[[8,400,12,0]],{controls}}},"pulse2":null,"noise":null}}"#
            );
            let score: CryScore = serde_json::from_str(&json).expect("JSON");
            assert!(score.program(0, 256).is_err(), "{controls}");
        }
        let score: CryScore = serde_json::from_str(
            r#"{
            "pulse1":{"delay":0,"duty":[2,2,2,2],"notes":[[8,400,12,1]],
                "contours":{"0":[800,8]}},"pulse2":null,"noise":null
        }"#,
        )
        .expect("JSON");
        assert!(
            score.program(0, 256).is_err(),
            "a contour cannot restart decay every frame"
        );
    }
}
