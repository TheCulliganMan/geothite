# Rochester: readable planter course

**Rejected by the user.** This record describes the former maze. The user found
its walls unclear and its design poor. The current revision is
[the two-switch lane course](rochester-gates.md); solving this historical maze
did not establish its quality.

The first gym now has three continuous planter beds with visible openings on
the **right, left, right**. This replaces the earlier short stubs and awkward
pockets. Tavel, Elvik and Varel retain their original parties and identities.
Both trainer victories are required before Varel accepts the challenge.

![Recorded normal-control solution](../renders/gyms/rochester-solvability/recorded-solution.png)

The diagram records actual positions from `VisibleShellController::press`,
rather than a predicted path. The regression uses literal directions and no
path finder for walking the room. Its initial fixture is the gym arrival,
delivered parcel, Totodile16 and Hoothoot14 with native level-up moves. No gym
victory or badge is supplied. It wins all three consecutive battles, spends
actual PP, earns Care and one Headbutt TM, checks Continue and repeat reward
protection, then exits through both real city carpets. This is a local course
proof for that party, rather than an earned opening chapter or every starter.
The existing fixture supplies a deterministic divider replay before input;
there are no RNG, party or progress writes during the walk or battles.

## Exact solution

Coordinates are 16px walking tiles; directions count actual tile steps, not
key repetitions used to turn or settle presentation.

1. From `(6,17)`, go up1. Face left and battle Tavel at `(5,16)`.
2. Right4, up4, left7, up3, right2. Face up and battle Elvik at `(5,8)`.
3. Right5, up6, left3. Face left and battle Varel at `(6,3)`.
4. Return right3, down6, left7, down3, right7, down4, left4, down2.
   Down once more exits to Rochester `(30,24)`. Re-enter and use the adjacent
   carpet to verify the second exit.

Elvik refuses before Tavel. Varel refuses before either required victory or
without the parcel. Losing grants no trainer victory, badge or TM.

## Publication and source generations

Published ignored pack: `content-packs/minnesota-rochester-readable.crystalpack`,
541 maps, content identity
`471572f0929249c5c924a35e4b260bfafda96c53d31ceeb24cf67edf43e3cdef`.
It extends the completed Leech shore parent. This is a separate gym revision;
later story and city expansion successors remain separate integration work.

[current.json](current.json) selects [the revision](rochester-candidate.json)
over the historical [eight-room source](interiors.json). Keeping that historical
generation intact preserves earlier publishers' pinned references. Default
`review_gyms.py` reviews the current selection; pass `--design` to review a
historical source explicitly. The Rust publisher replaces only Rochester's
terrain and two hints and rejects changes to battles, rewards or gates.
It grants no progress. Existing saves standing inside the old room need a
position review before migration.

Executed publication:

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_rochester_maze -- \
  content-packs/minnesota-northstar-leech-shore.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/rochester-candidate.json \
  content-packs/minnesota-rochester-readable.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-rochester-readable.crystalpack \
  --verify-published \
  --output planning/minnesota/renders/gyms/rochester-solvability/current-circuit

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-rochester-readable.crystalpack --audio --desktop
```

Executed literal control proof:

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-rochester-readable.crystalpack" \
CRYSTAL_GYM_WALK_PROOF=/tmp/geothite-rochester-walk-proof.json \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  rochester_readable_literal_walkthrough -- --ignored --nocapture --test-threads=1
```

The external proof JSON is temporary verification output, not game content.
`review_gym_walk.py` draws its recorded points over the verified published
whole-room image. It retains originals, conversions and all diagram versions.

## Review evidence and remaining scope

The ordinary-party literal course passes. Three additional focused controller
cases pass: required trainer sequence/Continue/reward persistence, missing
parcel/leader loss, and trainer loss. Two asset cases prove unrelated room,
actor, battle, warp, art, runtime-file and audio preservation, uncached native
module reconstruction, rejection of altered inputs and repeat installation.
All377 audio PCM hashes, frame counts and loop ranges match.

Every current circuit terrain pixel and collision quadrant matches source.
The native entrance and first-gap captures were visually inspected at noon,
facing down. Their independent source cameras use the same location, native
border and actor origin; all368640 pixels match in each capture. The preview
renderer's old four-pixel upward actor offset was corrected. Earlier previews,
failed comparisons and rejected layouts remain archived.

Changed Rust leaves pass rustfmt. Scoped all-target assets/Bevy Clippy passes
with existing warnings and the pre-existing `never_loop` allowance. Workspace
`cargo fmt --all -- --check` still fails first at the unrelated
`crates/offset-allocator/src/lib.rs:408`. An intervening shared story test used
`PokemonSpecies.as_str()`; its compile fix reads the species `id` field.

The other seven gyms retain their previous designs and need this same
readability and explicit-solution review. This first revision does not complete
the active gym improvement request or establish final user visual approval.
