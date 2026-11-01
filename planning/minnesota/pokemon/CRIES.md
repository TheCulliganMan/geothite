# Minnesota cry performances — revision 8

The [score book](cry-designs.json) covers **224 active Pokémon in 101 lines**,
including explicit replacement scores for every evolved form. Revision 8 rewrites
all six active legendaries, both mythicals, and the three-stage pseudo-legendary
line. These are eleven individual scores; the other **213 species retain exact
revision-7 WAV bytes**, including the loon and noise-only Velcro refinements.

[Open the current listening gallery](http://127.0.0.1:8937/crafted-v8/listen/).
The existing user-visible tab now shows revision 8. The opening comparisons are
Wrigglet → Pupump → Mosquitan, legendary land voices, and water/ice/memory voices.
Every species and evolution sequence remains available. Previous version now
means revision 7, held in an external snapshot. Nothing autoplays.

**The agent has not heard these renders.** Authored construction and verified
playback do not establish artistic acceptance. Earlier structural and numerical
checks did not resolve the user's perceptual complaint. Listening is the test.

## Developed rare voices

The rare adults last **2.71–3.48 seconds**. Longer duration comes from individually
scored development, rather than a global tempo change, repeated loop, or silent
padding. All new species length registers remain 256. Early forms stay short;
the larval feeding gulp becomes the final creature's abdominal swallow.

| Pokémon | Duration | Authored performance |
| --- | ---: | --- |
| Wrigglet | 0.452 s | Wet pinched feeding gulp, trapped needle squeal, swallowed click. |
| Pupump | 1.205 s | The gulp loads a ribbed bellows; pressure gathers below a needle whine, then a valve blows open. |
| Mosquitan | 3.148 s | Unequal tearing serrations above a separately swelling tank drone; wing overload, recovery, then the old feeding gulp swallows the drill. |
| Babehemoth | 3.164 s | One immense gentle nasal ox bellow; separating horn resonances remain over a deep descending chest. |
| Kettlurk | 3.248 s | A falling current crosses a rising return current; a dry torso cavity opens and leaves a remote vortex. |
| Pepivern | 3.482 s | Submerged foghorn opens a strained split steam whistle; hoarse paddle shudders subside while the water pressure continues. |
| Bunjant | 2.712 s | Wooden KRAH, uneven enormous open-throated laugh, rough oath and splintered pine-beard exhale. |
| Vergloom | 2.963 s | Nearly stationary moss-choked scrape; branch catch, hollow break, remote narrow scream over a dragging hidden chest. |
| Icemyth | 3.265 s | A suspended frozen face and a lower misremembered face tighten into dissonance, fracture in unequal cuts, then leave a displaced face. |
| Strandrak | 3.349 s | Stone impact unlocks three differently voiced shoreline chambers; a basalt bellow resolves through a diverging split outlet. |
| Flarkindle | 2.763 s | A fragile questioning breath folds time: its answer arrives early, leaves a gap, then returns lower after the first voice has faded. |

The previous full-roster revision replaced the shared bouncing articulation.
Gillpup has a steady closed-mouth coo; Mudmender has a noise-led nursery fan
breath; Velvawn has a forward nasal bleat; Shedstag has a hoarse flat rut-call.
Bristub and Latchquill are entirely unpitched irregular Velcro peels. These and
the refined loon wails/yodels are unchanged in revision 8.

## Crystal study

Original assembly was inspected at the commit pinned in `pret-source.lock.json`:
[cry programs](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/audio/cries.asm),
[species registers](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/data/pokemon/cries.asm),
and [audio engine](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/audio/engine.asm).
The Rust inspector reads programs directly from the user-supplied local pack.
No disassembly, copied commands, or second source catalog enters this repository.

Effective species renders of Lugia, Ho-Oh, Tyranitar and Suicune are included in
the gallery. They are synthesized references, not hardware recordings. Their
PCM hashes, frame counts and finite loop ranges were verified against the pack.

| Crystal reference | Effective duration | Useful construction |
| --- | ---: | --- |
| Lugia / Typhlosion base | 1.808 s | 41 main-pulse attacks against 7 companion attacks; only 3 shared onset frames. A fractured foreground can live over an independently sustained voice. |
| Ho-Oh / Aipom base | 1.122 s | Bright response phrases, cycling duty, independently ending voices; effective length 384. |
| Tyranitar / Raikou base | 1.122 s | Contrasting registers, growing envelopes and hard decays; only 3 shared pulse onsets across 12 distinct onsets. |
| Suicune / Magcargo base | 0.871 s | Nearly fixed main register against a more mobile companion; only 2 shared pulse onsets across 12 distinct onsets. |

Crystal's legendary cries are not uniformly its longest. Internal articulation,
register contrast, independent noise and different roles for the two pulse
voices create scale. Our longer rare performances use those principles without
copying the reference programs or assigning every rare creature a bass roar.

## Implementation and reproduction

Synthesis, score compilation, WAV export and pack publication use Rust. The
compiler is `crystal-audio/src/original_cry.rs`. Revision 8 extends its finite
part budget from 120 to **240 score frames**; notes remain limited to 60 frames,
entry delays to 60, and note lists to 96 entries. Exported cries must last
0.15–4.5 seconds. A boundary test renders and round-trips a full 240-frame
performance and rejects 241 frames. Shared synthesis kernels are unchanged.

Pulse notes are `[frames, Hz, volume, envelope pace]`; noise uses an NR43 register
instead of Hz. Optional note-indexed `contours` hold an envelope while changing
register and level; `duties` change mouth texture at a note boundary. These are
original authoring controls, not a hardware-sweep emulation claim. Original
scores retain their opt-in 2 ms attack, 45 ms final release and 75% output gain.
Legacy source programs retain exact PCM.

WAVs and comparison books stay outside the checkout; packs remain ignored. The
completed revision-7 donor already contains all active species and source art.

```sh
CARGO_TARGET_DIR=target/cry-audit-gallery cargo run --locked -p crystal-audio \
  --example cry_gallery -- planning/minnesota/pokemon/cry-designs.json \
  "$HOME/Music/Geothite/minnesota-cries/crafted-v8/listen" \
  planning/minnesota/pokemon/roster.json \
  "$HOME/Music/Geothite/minnesota-cries/crafted-v8/before-designs.json" \
  'Rare voices and Mosquitan line · revision 8'

CARGO_TARGET_DIR=target/cry-audit cargo build --locked -p crystal-assets \
  --bin pack_minnesota_cries --bin verify_crystalpack

target/cry-audit/debug/pack_minnesota_cries \
  content-packs/minnesota-cries-v7.crystalpack \
  planning/minnesota/pokemon content-packs/minnesota-cries-v8.crystalpack

target/cry-audit/debug/verify_crystalpack \
  content-packs/minnesota-cries-v8.crystalpack --audio
```

Reference WAVs are exported with, for example:

```sh
target/cry-audit/debug/study_pokemon_cries \
  content-packs/core-modular.browser.crystalpack LUGIA \
  "$HOME/Music/Geothite/minnesota-cries/crafted-v8/listen/references"
```

Export the other three species before rendering the gallery to include their
players. The local static server uses Python only to serve Rust-exported files:

```sh
python3 -m http.server 8937 --bind 127.0.0.1 \
  --directory "$HOME/Music/Geothite/minnesota-cries"
```

The native campaign packs remain separate from the review candidate. All 224
revision-8 WAVs are now published in web release `20261027-cries-v8-20`; see the
[publication runbook](../../../docs/pokedex-web.md). No new runtime opcode
requires rebuilding a client.

## Verification

The 37 audio library tests pass, including the extended finite-score boundary.
The unchanged 213 WAVs match revision 7 byte for byte. Each of the nine rare
adults has audible energy in 100 ms samples across its opening, development,
climax and late resolution. This rejects silent padding without pretending to
measure musical character.

The external-pack regression checks exact gallery/runtime PCM for every species,
metadata hashes and frame counts, finite loop ranges, clean endpoints, headroom,
species-register decoding, corruption rejection, slow playback, idempotent
publication and exact preservation of unrelated donor content. Run it with:

```sh
CRYSTAL_MN_CRY_BASE="$PWD/content-packs/minnesota-cries-v7.crystalpack" \
CRYSTAL_MN_CRY_PACK="$PWD/content-packs/minnesota-cries-v8.crystalpack" \
CRYSTAL_MN_CRY_WAV_DIR="$HOME/Music/Geothite/minnesota-cries/crafted-v8/listen" \
CARGO_TARGET_DIR=target/cry-audit cargo test --locked -p crystal-assets \
  --test minnesota_cries -- --include-ignored --nocapture
```

Scoped rustfmt passes. Workspace `cargo fmt --all -- --check` reports existing
unrelated formatting differences beginning in `crates/offset-allocator`.
Audio all-target Clippy passes with nine existing library warnings.

Full revision-8 pack verification passes: **693 maps, 475 species, 251 moves,
601 audio programs**, with every PCM hash, frame count and loop range matching.
Assets all-target Clippy is blocked by the existing denied `bad_bit_mask` at
`crystal-core/src/multiplayer/mod.rs:1332`; it does not reach the affected assets
targets. No new audio Clippy warnings were introduced. Logs and comparison
snapshots remain outside the repository under the revision-8 listening workspace.

All **four focused cry regressions pass** (6.33 seconds), including the full
224-species runtime/WAV comparison and unrelated-content preservation. Cargo
compiled the current libraries, then began rebuilding unrelated auto-discovered
exporter binaries. That invocation was interrupted. The focused test source was
compiled with the pinned `rustc --test`, linking those fresh Cargo libraries, and
executed as `target/cry-audit/debug/minnesota-cries-v8-tests --include-ignored
--nocapture` with the three pack/gallery variables above. The successful build
and test logs are `crafted-v8/focused-test-build.log` and
`crafted-v8/runtime-tests.log` in the external listening workspace. The normal
Cargo command above remains the reproducible frontend; its binary discovery can
make the build much broader than this test target.

## Web publication

All 224 revision-8 WAVs are live in `20261027-cries-v8-20` under
`/pokedex/media/cries/20261027-crafted-v8/`. The Rust publication exporter decoded
every cry through the runtime integrity path and produced exact gallery bytes.
All 224 public HTTPS downloads and all 2,192 remote static files match the local
outputs. The prior site's profiles, sprite/animation bytes, TM/HM learnsets and
starter signature content remain exact after URL normalization. Full app/media
regressions and public root/health checks pass. See the
[web runbook](../../../docs/pokedex-web.md) for deployment commands and build/lint
limitations.
