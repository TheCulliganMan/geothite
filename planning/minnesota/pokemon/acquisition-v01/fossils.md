# Driftless fossil research

The selected local review pack is
`content-packs/minnesota-current-fossils-repaired-r15-v01.crystalpack`, built from
`content-packs/minnesota-scripts-repaired-r15-v01.crystalpack`. It inherits
the accepted HM/habitat/cyclist stages and all 18 repaired cyclist battle-return
commands. Pack content hash:
`53917ea37309b35a3eff89340608f9e118dc568346a4fc258ce328a092e883af`.

A scientist beside the existing Driftless bluff grass at `(39,21)` offers one
specimen from a paired slab. The player can decline the study or the final
choice. LENS FOSSIL restores TRILUME; SPIRE FOSSIL restores SIPHLET. Choosing
commits only after actual key-item delivery. The courtyard scientist east of
Rochester's clinic, at `(20,10)`, restores the specimen at level 10 after the
Care Badge. A full party retains both the item and the restoration claim.

After defeating the regional Champion, the Driftless researcher offers the
unchosen specimen through a free, once-only research exchange. Both living
lines can therefore be collected in one run. The separate prehistoric beaver
line is still awaiting its own discovery path.

Three native controller cases pass choice/refusal/persistence/exchange,
badge/full-party/restoration, and both level-34 evolutions. The evolution case
declares level 33 and one Rare Candy, then uses actual PACK input, resolves
move-learning and evolution dialogue, checks the caught Pokédex entry and
continues the saved evolved party. These fixtures do not establish earned
statewide collection. The key-pocket failure branch is present; the current catalog has
only 22 other key items, so a full 25-slot pocket cannot be constructed from
those items for this test. No native full-key-pocket failure claim is made.

The strict comparison preserves 728 unrelated maps exactly, all terrain,
warps, existing actors, encounters, teams, field rules, rods, original items,
source art and audio. Both researcher approaches are structurally connected to
their route/city anchors; existing spawn landings remain unobstructed. All 67
new text rows fit eighteen columns. Two native-art planning views were
inspected; this does not establish new 3D visual acceptance.

All 377 audio PCM hashes, frame counts and loop ranges pass. The current
production client builds and loads the candidate. Its stdio client loads a
same-pack saved fixture, exposes the real Start/Pack/Save menu and moves through
the production controller with byte-empty stderr.

The acquisition audit now reports 135 potential direct species and a generous
upper bound of 214 after all evolution edges. Ten species in nine families still
lack sources. Other evolution conditions and earned access remain open.

Verified publisher commands, from the repository root:

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
RUST_MIN_STACK=67108864 cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin pack_minnesota_fossils
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_minnesota_fossils \
  content-packs/minnesota-scripts-repaired-r15-v01.crystalpack \
  content-packs/minnesota-current-fossils-repaired-r15-v01.crystalpack
```

The second command requires a fresh ignored output. Keep existing packs and
identity-bound saves; never overwrite an earlier publication or migrate a save
by rewriting its pack identity. Read the current `world.json` selector before
composing a later stage.

Scoped Rust formatting passes. Clippy for both affected crates/all targets passes
with the four recorded pre-existing lint allowances and existing warnings.
Whole-workspace formatting still has the pre-existing offset-allocator diff.
