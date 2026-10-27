# All eight current gyms in the regional story candidate

[Complete current rooms](../renders/gyms/regional-candidate/all-gym-interiors.png) ·
[Native Brainerd](../renders/gyms/regional-candidate/native-brainerd.png) ·
[Native Bemidji](../renders/gyms/regional-candidate/native-bemidji.png)

`content-packs/minnesota-northstar-eight-gyms.crystalpack` contains **538 maps**.
It keeps the 526-map Rochester/Winona/Minneapolis/northern-story lineage, refreshes
six superseded gyms, and adds the verified Mankato, St Paul and St Cloud cities.
Every current gym now has its real city facade in this same pack.

| Order | City / original leader | Current room | Puzzle | Safe exterior return |
| --- | --- | --- | --- | --- |
| 1 | Rochester / Varel | 7 × 10 | Planter maze; Tavel, Elvik, then leader | `(30,24)` |
| 2 | Mankato / Torven | 8 × 10 | Rock stairs and one-way drops | `(10,12)` |
| 3 | Minneapolis / Avrin | 5 × 9 | Two trainers and adjacent switch pair | `(42,24)` |
| 4 | St Paul / Brannic | 7 × 10 | Four reciprocal pad pairs and court grille | `(54,16)` |
| 5 | St Cloud / Saven | 9 × 11 | Required trainer wins and opposing wind lanes | `(8,28)` |
| 6 | Brainerd / Sivren | 5 × 8 | Mutually exclusive pool piers | `(12,36)` |
| 7 | Bemidji / Orlis | 6 × 10 | Guide HM, real Pack teaching and two Cut hedges | `(10,10)` |
| 8 | Duluth / Norek | 7 × 10 | Two reversible ice courses and cross-passage | `(16,12)` |

## Explicit gym-generation transition

The accepted source lineage is the Rochester story pack with content identity
`7a136c478074fc03544d89770eb7cae4e9dc83d4e2c8f0fbec6383055ed297e9`.
The reviewed 480-map donor has content identity
`b94a53f5dbe3cf0737658a890025a46eb7747350eea574df3e8a1b54f96734bb`.
These are internal pack content identities, not SHA-256 hashes of the files.
The refresh also pins all eight selected canonical source-object digests.
Unsupported inputs, modified room sources and repeated transitions are rejected.

Rochester and Minneapolis already have their current gyms. They stay exact,
including Rochester's city-specific delivery refusal. Slots2/4/5/6/7/8 receive
only their reviewed modules, private art, native field behavior and owned
trainer definitions. Group/map identities and city return destinations stay
stable; city-bound gyms receive their proper parent landmark. Existing numeric
story events, unrelated trainers and trainer numbers, original maps, art,
audio, HOME and recovery remain unchanged. New puzzle flags receive unused
numbers. The obsolete third Mankato practice trainer is removed; unused old
puzzle event numbers remain reserved. No saved player state is migrated or
awarded.

The review hall receives the current original cast. Seven authored dialogue
labels in five northern city/clinic maps replace superseded leader references;
normal text authoring wraps the longer names to native width. Every other
field in those maps stays exact. No vanilla dialogue or trainer is renamed.

The city builders then add three Mankato maps, five St Paul maps and four
St Cloud maps, with six additive town/clinic recovery records. Their checked
R04/R05/R06 public seams remain present. The output preserves the earned Winona
opening/R01 delivery, Minneapolis ICE/rescue/dispatch scenes and northern plot.

## Native caught-location capacity

The first combined gameplay run exposed St Paul/Cloud landmark IDs128/129,
which cannot fit Crystal's seven-bit caught-location field. The corrected city
publisher allocates an unused native ID in1..127, then uses an explicit existing
geographic area when that table is full. Mankato uses the last free slot;
St Paul shares the existing Twin Cities area and St Cloud shares River Bluffs.
All pre-existing landmark IDs and map assignments stay unchanged. The source
format and native caught-location field are not widened or truncated.

A runtime pack-verification rule now rejects a mapped caught location above127
before play. Its targeted regression rejects128 and accepts the native boundary
127. Asset checks require every mapped regional location to fit. The complete
city course passes after the fix. The standalone St Cloud landmark's copied
city-name typo is also corrected to ST CLOUD.

## Executed Rust publication

All authored strings are captured before construction. The pipeline writes only
a complete verified output; packs and every generated image remain ignored.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_regional_gyms -- \
  content-packs/minnesota-northstar-rochester.crystalpack \
  content-packs/minnesota-gyms-remake.crystalpack \
  planning/minnesota \
  content-packs/minnesota-northstar-eight-gyms.crystalpack

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test regional_gyms -- \
  --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-eight-gyms.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-northstar-eight-gyms.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_regional_gyms_controller_all_eight_actual_facades_and_local_courses \
  -- --ignored --nocapture --test-threads=1

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-northstar-eight-gyms.crystalpack \
  --verify-published --output planning/minnesota/renders/gyms/regional-candidate
```

Three preservation/rejection asset tests pass against the freshly reconstructed
and written identities. The new landmark boundary test passes. One production
suite executes all eight actual facade courses on this exact candidate: required
trainer wins, native puzzles, selected-move PP, badges, one TM each, saves/Continue
and both city returns. It includes real Cut teaching through Pack, opposing wind
input, pad saves, piers, ice and Duluth's separate acknowledged receiver.

All three Rochester chapter regressions also pass on the combined candidate:
an earned New Game/R01 originals delivery and walk home, both unqualified starter
checkpoint lanes, and the separately identified legal Quilava combat fixture.
Complete-room source/native terrain and collision comparisons pass; source and
native player images were inspected and retained. All377 audio PCM hashes, frame
counts and loop ranges pass.

The separate earned Cyndaquil acceptance case also passes on this candidate:
fresh New Game, a partner actually caught during play, native medicine purchases,
delivery, both trainer victories, Varel, first badge/TM and Continue. This run
uses live RNG and does not establish reliable balance for every starter or route.

Changed Rust leaves pass rustfmt. Full workspace formatting still reports
pre-existing differences beginning in `offset-allocator/src/lib.rs:408`, and
full all-target clippy remains blocked by the existing core multiplayer
always-false bit-mask comparison. Scoped affected-crate all-target lint passes
with existing `never_loop` findings allowed and no warnings in changed modules.

## Remaining regional journey

The eight-city suite initializes **eight separate local fixtures** on one pack.
Strong legal partners, initial street positions and prior badges are explicit
setup; Rochester's local gym case also seeds completed delivery. This proves
installation and local play together, not one continuous earned badge journey.
It does not establish ordinary-party balance.

The later [Prairie Road chapter](prairie-story.md) binds R02, and the
[River Valley successor](river-cities.md) binds R03 with a native second-badge
gate. The latter proves a continuous second/third gym course from explicit
prior-badge/strong-party setup. Remaining city/route bindings, courier quest and
progression gates still need integration and an earned sequential journey.
Rail remains unfinished and its public platforms stay sealed. Earlier
opening/story/city packs remain available; saves from a different identity or
from inside superseded gym terrain have no automatic migration promise. No
user visual acceptance is inferred from passing tests.
