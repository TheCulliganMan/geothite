# Cry synthesis research

The Crystal source links below use commit
`3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6`, pinned in this checkout's
`pret-source.lock.json`. Original cry programs are read from the supplied pack.

## Hardware, game engine and authoring are different layers

The Game Boy sound hardware has two pulse generators, one programmable wave
generator and one noise generator. Pulse duty and phase affect timbre. Noise is
an LFSR output: selecting its 7-bit mode makes a much shorter, more periodic
sequence than 15-bit mode. Its divider and shift control the clock; large shifts
do not simply mean more hiss. Envelope updates run at 64 Hz; the first pulse's
hardware sweep runs at 128 Hz. These clocks differ from the game's sound update.
See [Pan Docs: registers](https://raw.githubusercontent.com/gbdev/pandocs/master/src/Audio_Registers.md)
and [APU details](https://raw.githubusercontent.com/gbdev/pandocs/master/src/Audio_details.md).

For a pulse register `r`, frequency is `131072 / (2048 - (r & 2047))` Hz.
Adding a fixed register offset therefore produces a pitch-dependent interval,
not constant transposition in semitones. Equal-tempered pitch is a possible
creative choice, not an inherent property of these cries.

The Crystal driver maintains eight logical channels, with effect channels 5–8
mapped to the same four hardware generators. Its cry loader installs each
species' pitch offset and uses the length word as tonal-channel tempo. Noise
does not receive that tempo. The driver rotates a four-duty pattern each sound
update. Cry priority and stereo routing are handled separately from the voice
program. See [the sound engine](https://raw.githubusercontent.com/pret/pokecrystal/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/audio/engine.asm)
at `_PlayCry`, `HandleTrackVibrato`, `ParseSFXOrCry`, `SetNoteDuration` and
`Music_DutyCyclePattern`, and [the species loader](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/home/pokemon.asm).

The sound-effect note specifies duration, starting volume, envelope and frequency
register. Positive envelope fade decreases volume, negative fade increases it,
and zero holds it; these are not arbitrary linear ramps in milliseconds.
The encoded duration is incremented before timing multiplication. A program's
phrase can use repeated retriggers, local loops and independently timed tracks.
See [the command definitions](https://raw.githubusercontent.com/pret/pokecrystal/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/docs/music_commands.md).

Two useful deductions for original synthesis follow from those clocks. At
`4194304 / 70224` sound updates per second, a four-entry duty pattern repeats
about 14.93 times per second, adding a rapid timbral flutter even during a held
pitch. In ordinary 7-bit noise mode, the 127-clock repeating sequence can become
a pitched buzz: its repetition frequency is the LFSR clock divided by 127.
For example, a 131072 Hz clock repeats about 1032 times per second. The noise
clock is not itself the perceived fundamental. A long-mode sequence, with a much
longer period, behaves differently even at the same timer setting.

Envelope rate also sets a practical articulation scale. At fade pace 1, each
volume step takes about 15.625 ms; a starting level of 12 needs about 187.5 ms
to decay completely. A five-video-frame note lasts only about 83.7 ms, so it
ends before that decay completes. Frequent retriggering, a long natural decay
and a low starting volume can therefore produce very different call shapes
even when the nominal pitches and total duration match.

Yellow's spoken Pikachu is an exception to the ordinary synthesized-cry model.
Masuda describes converting a sampled voice into one-bit data and playing it
through rapid clicks in [his February 29, 2008 account](https://www.gamefreak.co.jp/blog/dir_english/2008/02/index.html).
Do not generalize that method to the ordinary Crystal cry bank.

## How to investigate the local programs

The source pack is the source of truth for the actual programs in this checkout.
Use `study_pokemon_cries` to decode them only in memory and verify synthesized
PCM against the pack's hashes, frame counts and absent cry loop ranges. For a
selected cry, its source-clock summary exposes track onset alignment, retrigger
density, envelope varieties and register-pitch diversity without copying the
program. Encoded duration statistics are not equivalent to elapsed durations
after species tempo, loops and fractional carry.

The source-clock report tracks starting envelope controls, not the hardware's
instantaneous envelope amplitude. A channel can keep running after its envelope
has reached zero. Likewise, a frequency shadow register is not the instantaneous
output of a hardware sweep. Infer audible behavior from an actual render and
listening, not a state-field name.

Compare pulse attack alignment and channel lengths: two channels that share some
attacks can still create substantial internal motion between those attacks.
Look for whether the body is sustained, retriggered or broken into separate
syllables. Check whether noise is synchronized, delayed, independently rhythmic,
tonal or broadband. Examine species variants of the same base cry before choosing
an evolution policy for a new family.

## Findings from this checkout's supplied bank

The Rust inspector was executed against
`content-packs/core-modular.browser.crystalpack` on 2026-10-27. All 68 base-cry
PCM hashes, frame counts and finite loop metadata matched. Base programs span
0.301–1.808 seconds, with 251 species register records. These are observations
of this local bank and renderer, not a substitute for a cartridge comparison.

Selected control-flow-expanded comparisons:

| Base program | Pulse retriggers, ch5 / ch6 | Shared pulse onset frames / union | Construction lesson |
| --- | --- | --- | --- |
| Nidoran M | 3 / 3 | 1 / 5 | Even a short three-part call uses offset pulse articulation and several decay rates. |
| Hoothoot | 8 / 9 | 2 / 15 | Unequal, interleaved tracks carry the phrase; noise uses four separate settings. |
| Marill | 7 / 6 | 5 / 8 | Some attacks reinforce each other while other attacks diverge; rising and falling envelopes coexist. |
| Drowzee | 12 / 11 | 2 / 21 | Close register pitches, loops and a delayed companion create an extended phrase. |
| Paras | 8 / 9 | 1 / 16 | Separate delays and eight different pulse pitches shape the contour. |
| Raichu | 6 / 7 | 1 / 12 | Widely separated pulse registers and changing duty patterns create contrasting registers, rather than simple detuned doubling. |
| Lapras | 5 / 5 | 4 / 6 | Mostly coordinated pulses still use unequal durations and different envelope rates. |

For example, the unmodified Raichu base's pulse-register ranges correspond to
about 1365–3277 Hz on one pulse versus 373–444 Hz on the other. This is a much
larger spectral separation than a universal companion detune of a few cents.
The Drowzee base instead has pulse ranges displaced by only one register unit.
Both relationships belong in a useful authoring vocabulary.

Base-program names are not sufficient to identify a heard species cry. In this
bank, the Paras base also supplies Mew, Mewtwo, Staryu and Starmie; the Raichu base
also supplies Nidoking, Articuno and Moltres. Always inspect the species record
and render its effective registers before using a reference by species name.
The inspector accepts either a base `CRY_` ID or a species token for this reason.

The Hoothoot/Noctowl comparison demonstrates unequal layer scaling: Hoothoot's
tonal tracks run for 31 and 26 source-clock frames, while Noctowl's run for 60
and 50. Their noise track stays at 38 frames. Rendered duration changes from
0.636 to 1.005 seconds, not by uniformly stretching the entire mixed sound.
Nidorino also raises the pitch register relative to Nidoran M, contradicting a
rule that every evolution must lower its voice.

Header inventory and sounding layers differ. Although 62 base programs declare
channels 5/6/8, only 54 produce nonzero kernel output on all three. Some imported
channels contain only setup. The current exporter aliases empty consecutive
labels, but does not explicitly join a nonempty setup prefix to the following
global label. This is a potential import/fallthrough issue to investigate, not
evidence that the original composition intentionally omitted that voice. The
measured channel timelines and the export parser must be read together.

## Current Geothite rendering limits to investigate

`crystal-audio/src/synth/render.rs` gives direct `square_note` events a pulse
kernel and per-event envelope. The `pitch_sweep` handler is gated to logical
channel 1; ordinary cry square notes use effect channels. Do not assume that
adding a sweep command on channel 5 yields a modeled hardware sweep.

The noise rendering branch passes a noise note to `kernels::noise`; unlike the
square-note branch, it does not add `cry_pitch` to that note's register. The
source clock's frequency state does apply the offset. Verify the consequences
before relying on species pitch to modify an authored noise layer. These are
code-path observations, not a completed hardware-fidelity audit or authorization
to alter all existing audio.

The mixer includes filtering and peak scaling. Stopping an oscillator does not
guarantee a zero final PCM sample, because filter state can remain. Measure
release transients and listen for clicks. Do not weaken an output-quality test
merely to label a release as silent. A hash match proves agreement with the
existing renderer and pack; it does not prove the renderer matches a cartridge.

The first authored prototypes demonstrated this problem: the end of Yodeloon's
unmastered filter tail was roughly 6000 PCM units despite oscillator release.
New authored scores now opt into an `output_envelope` after filtering: 2 ms
attack, 45 ms release and 75% linear gain. This leaves waveform construction and
legacy programs unchanged, while giving new voices clean endpoints and output
headroom. The effect is encoded in the program, so native, WASM and WAV exports
use the same mastering rather than a separate exporter-only fade. It is original
sound design, not a change to the historical synthesis reference.

## Original sound-design decisions

Creature anatomy and behavior can suggest a vocal gesture without literally
imitating a machine, food or material in its theme. Design a characteristic
attack, body and ending, then decide how independent sources combine to express
it. A companion pulse can momentarily diverge from the main voice to create
roughness or a second vocal register. An independently articulated noise layer
can produce a throat rasp or chitinous buzz rather than uniform background hiss.

These are design possibilities, not universal historical rules. Let the studied
examples establish what belongs in the authoring vocabulary; let the new
creature determine which subset to use. Revisit a prototype when its description
promises complexity that the actual channel construction does not contain.
