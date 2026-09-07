//! Cartridge-style synthesis shared by the native exporter and browser WASM.
use crate::{AudioCommand, AudioSource};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
mod control;
mod kernels;
mod midi;
mod render;
use control::expand;
pub use control::{link, validate_references};
use kernels::*;
pub use midi::{decode_midi, encode_midi};

pub const SAMPLE_RATE: usize = 44_100;
const FRAME_NUM: usize = SAMPLE_RATE * 70_224;
const FRAME_DEN: usize = 4_194_304;
const PREC: f64 = 281_474_976_710_656.0;
const TABLE: [i32; 25] = [
    0, 0xf82c, 0xf89d, 0xf907, 0xf96b, 0xf9ca, 0xfa23, 0xfa77, 0xfac7, 0xfb12, 0xfb58, 0xfb9b,
    0xfbda, 0xfc16, 0xfc4e, 0xfc83, 0xfcb5, 0xfce5, 0xfd11, 0xfd3b, 0xfd63, 0xfd89, 0xfdac, 0xfdcd,
    0xfded,
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub profile: String,
    pub music_data: MusicData,
    pub cry_pitch: Option<i64>,
    pub cry_length: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MusicData {
    pub channel_count: u8,
    pub channels: BTreeMap<String, AudioSource>,
    pub subroutines: BTreeMap<String, AudioSource>,
    #[serde(default)]
    pub shared_sources: BTreeMap<String, AudioSource>,
}
impl MusicData {
    fn sources(&self) -> BTreeMap<String, AudioSource> {
        let mut sources = self.channels.clone();
        sources.extend(self.subroutines.clone());
        sources.extend(self.shared_sources.clone());
        sources
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NoiseNote {
    pub length: i32,
    pub volume: i32,
    pub fade: i32,
    pub frequency: i32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthContext {
    pub drumkits: BTreeMap<i32, BTreeMap<i32, Vec<NoiseNote>>>,
    pub wave_samples: BTreeMap<i32, Vec<i32>>,
    pub wave_instrument_map: BTreeMap<i32, i32>,
}
impl SynthContext {
    pub fn cartridge() -> Result<Self> {
        let mut context: Self = serde_json::from_str(include_str!("context.json"))?;
        for index in 0..16 {
            context
                .wave_samples
                .entry(index)
                .or_insert_with(|| vec![0; 32]);
        }
        Ok(context)
    }
}
#[derive(Debug, Serialize)]
pub struct Rendered {
    #[serde(skip)]
    pub samples: Vec<i16>,
    pub sample_rate: usize,
    pub loop_samples: BTreeMap<u8, usize>,
    pub channel_frames: BTreeMap<u8, usize>,
    pub channel_samples: BTreeMap<u8, usize>,
    pub channel_nonzero: BTreeMap<u8, usize>,
}
impl Rendered {
    pub fn downsample(&self) -> Vec<i16> {
        self.samples
            .chunks_exact(4)
            .flat_map(|s| [s[0], s[1]])
            .chain(if self.samples.len() % 4 == 2 {
                self.samples[self.samples.len() - 2..].to_vec()
            } else {
                Vec::new()
            })
            .collect()
    }
}
#[derive(Clone, Debug)]
struct State {
    tempo: i32,
    note_length: i32,
    remainder: i32,
    default_length: Option<i32>,
    envelope: Option<(i32, i32)>,
    octave: i32,
    duty: i32,
    duty_pattern: Option<i32>,
    transpose_octaves: i32,
    transpose_pitches: i32,
    wave_instrument: i32,
    wave_volume: i32,
    instrument_pitch: i32,
    pitch_offset: i32,
    pan: [bool; 2],
    vib_delay: i32,
    vib_delay_count: i32,
    vib_extent: i32,
    vib_rate: i32,
    vib_counter: i32,
    vib_up: bool,
    vib_latched: i32,
    sweep: i32,
    sweep_enabled: bool,
    sweep_shadow: i32,
    pulse_active: bool,
    slide: Option<(i32, usize)>,
    noise_enabled: bool,
    lfsr: i32,
    noise_acc: f64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            tempo: 256,
            note_length: 1,
            remainder: 0,
            default_length: None,
            envelope: None,
            octave: 4,
            duty: 2,
            duty_pattern: None,
            transpose_octaves: 0,
            transpose_pitches: 0,
            wave_instrument: 4,
            wave_volume: 0,
            instrument_pitch: 0,
            pitch_offset: 0,
            pan: [true, true],
            vib_delay: 0,
            vib_delay_count: 0,
            vib_extent: 0,
            vib_rate: 0,
            vib_counter: 0,
            vib_up: false,
            vib_latched: 0,
            sweep: 0,
            sweep_enabled: false,
            sweep_shadow: 0,
            pulse_active: true,
            slide: None,
            noise_enabled: false,
            lfsr: 0x7fff,
            noise_acc: 0.0,
        }
    }
}
#[derive(Clone)]
struct Segment {
    audio: Vec<i16>,
    pan: [bool; 2],
}
struct Renderer<'a> {
    program: &'a Program,
    context: SynthContext,
    tempos: BTreeMap<usize, (i32, i32)>,
    tempo_channel: Option<u8>,
    primary: u8,
    sample_remainder: usize,
    perfect_pitch: bool,
    volume: BTreeMap<usize, [i32; 2]>,
    loops: BTreeMap<u8, (usize, usize)>,
    frames: BTreeMap<u8, usize>,
    samples: BTreeMap<u8, usize>,
    channels: BTreeMap<u8, Vec<Segment>>,
}
pub fn render(program: &Program, context: SynthContext) -> Result<Rendered> {
    ensure!(
        program.profile == "pokecrystal-midi-v1",
        "unsupported audio program profile"
    );
    // Exported command graphs can contain several named passes for one hardware
    // channel. Preserve their ordering and final channel output, including the
    // shared fractional sample clock, as the previous renderer does.
    for source in program.music_data.channels.values() {
        let number = source.number.context("channel number missing")?;
        ensure!((1..=8).contains(&number), "invalid audio channel {number}");
    }
    validate_references(&program.music_data)?;
    let mut channels = program.music_data.channels.iter().collect::<Vec<_>>();
    channels.sort_by_key(|(_, s)| s.number);
    let primary = channels
        .first()
        .and_then(|(_, s)| s.number)
        .context("audio program has no channels")?;
    let mut r = Renderer {
        program,
        context,
        tempos: BTreeMap::new(),
        tempo_channel: None,
        primary,
        sample_remainder: 0,
        perfect_pitch: false,
        volume: BTreeMap::from([(0, [7, 7])]),
        loops: BTreeMap::new(),
        frames: BTreeMap::new(),
        samples: BTreeMap::new(),
        channels: BTreeMap::new(),
    };
    let streams = channels
        .iter()
        .map(|(name, source)| {
            Ok((
                source.number.context("channel number missing")?,
                expand(&program.music_data, name)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    for (number, stream) in &streams {
        r.scan_tempo(stream)?;
        if !r.tempos.is_empty() {
            r.tempo_channel = Some(*number);
            break;
        }
    }
    for (number, stream) in streams {
        let out = r.channel(number, &stream)?;
        r.channels.insert(number, out);
    }
    let nonzero = r
        .channels
        .iter()
        .map(|(n, segs)| {
            (
                *n,
                segs.iter()
                    .flat_map(|s| &s.audio)
                    .filter(|s| **s != 0)
                    .count(),
            )
        })
        .collect();
    r.sync();
    Ok(Rendered {
        samples: r.mix(),
        sample_rate: SAMPLE_RATE,
        loop_samples: r.loops.iter().map(|(n, (_, s))| (*n, *s)).collect(),
        channel_frames: r.frames,
        channel_samples: r.samples,
        channel_nonzero: nonzero,
    })
}
fn number(s: &str) -> Result<i32> {
    let s = s.trim();
    let (sign, s) = if let Some(s) = s.strip_prefix('-') {
        (-1, s)
    } else {
        (1, s)
    };
    let (radix, s) = if let Some(s) = s.strip_prefix("0x").or_else(|| s.strip_prefix('$')) {
        (16, s)
    } else if let Some(s) = s.strip_prefix("0b").or_else(|| s.strip_prefix('%')) {
        (2, s)
    } else {
        (10, s)
    };
    Ok(sign
        * i32::from_str_radix(s, radix).with_context(|| format!("invalid audio integer {s}"))?)
}
fn arg(c: &AudioCommand, n: usize) -> Result<i32> {
    number(
        c.args
            .get(n)
            .with_context(|| format!("{} missing argument {n}", c.command))?,
    )
}
fn note_frames(ticks: i32, st: &mut State, tempo: i32) -> usize {
    let total = st.note_length.max(1) * ticks.max(1) * tempo.max(1) + st.remainder;
    st.remainder = total & 255;
    (total >> 8).max(1) as usize
}
fn sample_count(frames: usize, remainder: &mut usize) -> usize {
    let total = frames as u64 * FRAME_NUM as u64 + *remainder as u64;
    *remainder = (total % FRAME_DEN as u64) as usize;
    (total / FRAME_DEN as u64) as usize
}
fn ratio_floor(a: usize, b: usize, divisor: usize) -> usize {
    ((a as u64 * b as u64) / divisor as u64) as usize
}
fn ratio_rounded(a: usize, b: usize, divisor: usize) -> usize {
    ((a as u64 * b as u64 + (divisor / 2) as u64) / divisor as u64) as usize
}
fn ratio_ceil(a: usize, b: usize, divisor: usize) -> usize {
    (a as u64 * b as u64).div_ceil(divisor as u64) as usize
}
fn pitch(name: &str, s: &State) -> Result<i32> {
    let notes = [
        "C_", "C#", "D_", "D#", "E_", "F_", "F#", "G_", "G#", "A_", "A#", "B_",
    ];
    let index = notes
        .iter()
        .position(|n| *n == name)
        .with_context(|| format!("invalid note {name}"))? as i32
        + 1
        + s.transpose_pitches;
    ensure!(
        (1..TABLE.len() as i32).contains(&index),
        "note index out of bounds"
    );
    let mut reg = TABLE[index as usize];
    if reg >= 0x8000 {
        reg -= 0x10000;
    }
    let octave = s.octave + s.transpose_octaves;
    ensure!((0..=15).contains(&octave), "octave out of bounds");
    for _ in octave..7 {
        reg >>= 1;
    }
    Ok((reg + s.pitch_offset + s.instrument_pitch) & 2047)
}
fn frequency(reg: i32, wave: bool) -> f64 {
    (if wave { 65536.0 } else { 131072.0 }) / (2048 - reg.clamp(0, 2047)) as f64
}
