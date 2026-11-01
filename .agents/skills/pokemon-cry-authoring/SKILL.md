---
name: pokemon-cry-authoring
description: Study and synthesize original Game Boy style Pokémon or creature cries from sound-channel primitives. Use for cry design, evolution-family vocal identity, and Rust audio authoring in Geothite; not music composition or sampled voice imitation.
---

# Pokémon cry authoring

Treat a cry as a short performance with independent sound layers. Before
designing a roster, study real programs in the supplied local content pack and
explain which observed techniques the new design needs. Read
[the synthesis research](references/synthesis.md) for engine mechanics,
measurement limits and source-backed distinctions.

## Study before implementing

In Geothite, use the Rust inspector to verify and summarize the supplied cries:

```sh
cargo run --locked -p crystal-assets --bin study_pokemon_cries -- \
  content-packs/core-modular.browser.crystalpack
cargo run --locked -p crystal-assets --bin study_pokemon_cries -- \
  content-packs/core-modular.browser.crystalpack CRY_CHIKORITA
```

The selected-program report measures channel timing, note retriggers, duty and
envelope variety, pulse onset alignment and species pitch/length variants.
It does not export the source commands. Inspect further examples that challenge
your initial model: a short peep, a raspy call, an extended phrase, an insect-like
texture and related species. Do not infer a design grammar from one example,
channel counts or a list of supported commands alone.

Use a species token such as `NOCTOWL` to include the effective species-register
render. A final external directory argument exports a listening WAV; keep that
directory outside the checkout. Base `CRY_` labels can supply several unrelated
species, so a base program is not automatically the species named by its label.

Distinguish what the cartridge engine does, what Geothite currently renders, and
what you are choosing creatively. A PCM hash verifies repeatability, not faithful
hardware emulation, recognizability or artistic quality. Numerical summaries and
spectrograms cannot substitute for hearing. State explicitly when you have not
listened; do not describe an unperformed auditory review as passed.

## Design original voices

Read the creature's anatomy, temperament, size and evolution changes. Choose its
vocal character before its oscillator settings: questioning, hoarse, breathy,
urgent, mournful, guttural or playful. A furnace creature can have a voice with a
hot, rough throat without sounding like a generic furnace sound effect. Preserve
the creature's personality as well as its material theme.

Author the main pulse, companion pulse and noise as independent timelines when
needed. The second pulse can answer, interrupt, diverge, beat against, or briefly
reinforce the first; it need not be a detuned duplicate. Noise can provide an
attack, a separately timed growl, a periodic buzz or a changing breath texture.
Use envelopes and retriggering to shape articulation. Frequency movement need
not be a sequence of equal-tempered notes. Choose static duty, changing duty or
a frame-cycled pattern deliberately rather than applying one universal recipe.

Build and compare a varied set of prototypes before expanding to the roster.
Keep notes on the intended identity, the actual construction, and what a revision
changed. Related species may retain rhythm or timbral ancestry while changing
other features; do not apply one mandatory pitch-down/length-up rule to every
evolution. Similar types and neighboring IDs need not sound alike. Never generate
voices from name hashes, arbitrary ID offsets or a generic type-to-sound table.

For Minnesota work, use `planning/minnesota/pokemon/roster.json`,
`roster-curation.json` and `species-profiles.json` for active identities and
biology. Source `MN_` IDs differ from displayed Pokédex numbers. Preserve retired
species and their art. Track active coverage against those files rather than
hard-coding a remembered roster count.

## Implement and verify

The reusable compiler is `crystal-audio/src/original_cry.rs`. Author
`planning/minnesota/pokemon/cry-designs.json`: each family has a vocal `intent`,
a `score`, and explicit species register/character records. Scores contain
optional `pulse1`, `pulse2` and `noise` parts with independent delays and notes.
Pulse notes are `[video_frames, Hz, volume, envelope_pace]`; noise notes replace
Hz with an NR43 register byte. A zero-volume note is a gap. Constant four-entry
duty arrays select static duty; nonconstant arrays select frame-cycled duty.
A species can supply a complete replacement `score` when anatomy changes.

The compiler gives new voices a 2 ms attack, 45 ms final release and 75% output
gain after filtering. This opt-in mastering prevents truncating the current
mixer's residual DC tail and supplies headroom. Legacy programs omit the optional
`output_envelope` field and preserve their exact PCM. Keep this distinction clear:
the new release is an authoring choice, not a claimed cartridge behavior.

Render an external listening gallery and publish a separate ignored pack:

```sh
cargo run --locked -p crystal-audio --example cry_gallery -- \
  planning/minnesota/pokemon/cry-designs.json \
  "$HOME/Music/Geothite/minnesota-cries/roster-v1" \
  planning/minnesota/pokemon/roster.json
cargo run --locked -p crystal-assets --bin pack_minnesota_cries -- \
  content-packs/minnesota-city-life-fair-v3.crystalpack \
  planning/minnesota/pokemon content-packs/minnesota-cries.crystalpack
```

Supplying the roster to the gallery preserves evolution order even where stable
source IDs run in a different order. The publisher requires exact active-family
and species coverage; it rejects missing designs rather than filling them with
fallbacks. The input base must already contain the active Minnesota species.

Follow the checkout's AGENTS.md. Keep synthesis and export in Rust inside the
Geothite repository, using the pinned toolchain. Read original game programs
directly from the local pack. Keep disassembly, ROMs, copied program catalogs,
raw command dumps and generated audit manifests out of source and shipped assets.

Prefer the existing `crystal-audio` renderer and pack infrastructure. Investigate
whether a proposed effect is implemented on the actual channel and command path
being used; accepting a command does not prove the effect is rendered. Avoid
changing shared kernels merely to make a new authoring scheme convenient.

Publish to a separate ignored content pack. Export listening WAVs outside the
checkout and use minimal browser playback integration when useful. Verify the
new cries through the same register and integrity-decoding path as runtime
playback. Check termination, duration, frame counts, hashes, loop absence,
coverage, unwanted silence, clipping and output transients. Preserve every
unaffected audio asset, runtime file and unrelated game-data field. Run the
repository's relevant tests, formatting check and clippy.

```sh
cargo test --locked -p crystal-audio --lib
CRYSTAL_MN_CRY_BASE="$PWD/content-packs/minnesota-city-life-fair-v3.crystalpack" \
CRYSTAL_MN_CRY_PACK="$PWD/content-packs/minnesota-cries.crystalpack" \
CRYSTAL_MN_CRY_WAV_DIR="$HOME/Music/Geothite/minnesota-cries/roster-v1" \
  cargo test --locked -p crystal-assets --test minnesota_cries -- --ignored --nocapture
cargo run --locked -p crystal-assets --bin verify_crystalpack -- \
  content-packs/minnesota-cries.crystalpack --audio
```

Deliver an accessible listening result with the reusable design skill and
reproducible Rust commands. Keep any aesthetic uncertainty explicit. Completion
requires the requested roster's actual cries, not just a generator, a voice table,
a passing hash check or a handful of prototypes.
