# Short coastal arrival and compact Gunflint

The straight approach into Grand Marais is removed. A four-metatile bend at
North Shore's northern end now meets the town's west side, twenty metatiles
closer. North Shore remains north–south and warm; town shares its connected
Superior water. Gunflint is recomposed at32×32 instead of64×56, about71% less
area. The separate65-metatile inland Neck is retired, with its BERRY×3 pickup
and persistent flag2387 moved to Gunflint's ranger junction.

## Verified native result

- Pack: `content-packs/minnesota-superior-compact-v03.crystalpack`,689maps.
- SHA256: `032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899`.
- Content identity: `f2ac93f14af05a2935947e1696385a26a16b2d2bd509e5da3e9223af02f98d63`.
- Exact parent: `content-packs/minnesota-superior-inland-winter-v01.crystalpack`.
- Parent SHA256: `65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17`.
- Reviewed aggregate contract: [native.compact.contract.json](native.compact.contract.json).
- Contract SHA256: `fec56ec1141f91bb3d3ae7d371990ba8bf0b4ea22cc6505e5b0e58c3a636e135`.

Strict native write/readback, whole-pack inverse preservation, uncached native
behavior, all96connected exterior footprints, reciprocal offsets, cycle
consistency and reachable water boundaries pass. Five maps are removed and
nine retained modules change; all680other original modules remain exact.
Existing audio, catalog, story, trainer parties, event identities and recovery
data are preserved. One native snowy-pine recipe is appended privately without
changing existing glyph recipes. Final audio verification passes all377programs'
PCM hashes, frame counts and loop ranges. A fresh Rust replay is byte-identical.

| Exterior | Origin, metatiles | Size |
| --- | --- | --- |
| R11 compact terminal |222,−400|4×24|
| Grand Marais |226,−400|20×19|
| Cedar snowline |222,−412|20×12|
| Gunflint aggregate |190,−432|32×32|
| Boundary Waters |158,−430|32×24|

The seventeen-map western exterior component translates rigidly by92,−3;
its internal geometry remains intact. Other exterior footprints remain fixed.
R11 retains its56×96source canvas, with480explicitly unpublished cells where
the former town approach stood. Its lower neighbor receives twenty visible
native rock cells to close the cropped northern overhang. The dark gap below
town in the actual atlas is genuinely unmapped; no terrain is invented there.

Cedar WEST connects to retained lower `MnR12C00R01` with +4/−4 offsets.
Cedar SOUTH connects to town NORTH +4/−4. Gunflint's upper western port meets
Boundary Waters with +2/−2. The compact source preserves rooms42:7/8, trainers,
residents, rewards, encounters, native Cut behavior and useful ice stops.

## Production input verification

Thirteen controller cases pass against the exact final pack:

- Both North Shore walking lanes cross the compact bend and town seam, in
  both directions. Shared Superior Surf lanes, Save/Continue and dismount pass.
- Every lower-north overhang water approach is blocked by actual native rocks;
  the real upper/lower water passage remains traversable.
- North departure denies missing badge/signal prerequisites, retains dialogue
  input ownership, and permits qualified access on both lanes.
- Both Gunflint endpoint lanes, every ordinary actor approach and the formerly
  isolated bog sign are reachable through production walking input.
- Both original Gunflint rooms work, including the cave's actual MAX_ETHER
  consumption/PP restoration and both ranger-hut return carpets.
- Cut denial, successful native removal, gated reward, Continue and the native
  tree reset after map return pass. The reward flag remains persistent.
- BERRY×3 is collected once, survives Continue and map reentry, and does not
  duplicate. Three actual frost courses reach their dry/wall stops, reject
  mid-slide direction changes and allow the intended return.
- Required encounter grass starts an authoritative wild battle. Cedar's ledge
  jumps downward, rejects reversal and has a short ordinary grassy return.
- Grand Marais's six original room pairs return correctly, including the actual
  Field Station welcome/gift scene. Both incoming roads permit an unqualified
  injured party to reach native nurse care without granting the Duluth signal.

Two asset cases pass: exact frozen source bindings and guarded publication,
including stale-input and unrelated-mutation rejection. The canonical Rust
source/native parity test passes for every published terrain cell. Twelve
canonical files were promoted only after exact before-byte guards and native
terrain/anchor/paired-warp checks; see [promotion manifest](canonical-promotion-v01/manifest.json).

These are explicit local party/position/badge fixtures followed by production
`VisibleShellController` input. They establish this geography's local behavior;
they do not establish an earned statewide campaign or compatibility with old
saves positioned inside retired maps.

## Visual evidence and retained history

- [Actual connected native region](../../renders/superior-gunflint-quality-v01/native-connected-v03/connected-review.png).
- [Actual source-pixel connected original](../../renders/superior-gunflint-quality-v01/native-connected-v03/connected-original-32px-per-metatile.png).
- [Twenty headless native player fixtures](../../renders/superior-gunflint-quality-v01/native-player-cameras-v04/receipt.json).
- [Matched source/native classic-camera comparisons](../../renders/superior-gunflint-quality-v01/native-player-pairs-v03/receipt.json).
- [Seven actual reached-Save captures](../../renders/superior-gunflint-quality-v01/native-reached-cameras-v01/receipt.json), with no position, time or story overrides.
- [All35 verification logs](../../renders/superior-gunflint-quality-v01/native-publication-v01/logs/).
- [Visual critique and iteration history](VISUAL_REVIEW.md).

Originals, crops, rejected designs, rejected capture epochs and reached Saves
remain retained under ignored `planning/minnesota/renders/`. The final fixture
plan is [native-camera-plan-v08.json](native-camera-plan-v08.json). All capture
starts use unoccupied native FLOOR/TALL_GRASS. Cedar's corrected source cameras
include actual connected terrain without edge clamping. Coastal source frames
are terrain-only; the production frames additionally show actual player/NPCs.
Native water/grass animation phases may differ from the source-art still.
Forty-one final-identity Saves are retained, along with earlier-identity history.
The seven selected reached views preserve their saved night clock and palette;
blue night tint on the warm coast is not winter terrain.

The first live room test found that a DOOR on a chunk connection could not be
entered from its legitimate southern approach. The complete hut moved one
metatile north in source-v10, retaining its second warp identity and existing
interior. Production entry and both returns then passed. A separate native
visual pass replaced three repeated west-bank corners with the correct native
horizontal bank. Earlier packs v01/v02 remain immutable history.

## Rebuild and inspect

From the repository root, the executed Rust build uses the pinned toolchain:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 RUST_MIN_STACK=33554432 \
cargo build --locked --profile wasm-dev -p crystal-assets --bin build_compact_superior \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0'
```

The verified replay command below requires a fresh output path. The preserved
`-replay` output already exists from verification; use `-rebuild` for another run:

```sh
RUST_MIN_STACK=33554432 target/runtime-fps/wasm-dev/build_compact_superior \
  content-packs/minnesota-superior-inland-winter-v01.crystalpack \
  planning/minnesota/revisions/superior-gunflint-quality-v01/native.compact.contract.json \
  fec56ec1141f91bb3d3ae7d371990ba8bf0b4ea22cc6505e5b0e58c3a636e135 \
  planning/minnesota/revisions/north-shore-ely-geography-v01/coast-quality-v01/source-v04/compact-bend.contract.json \
  planning/minnesota/revisions/gunflint-quality-v01/source-v10/MN_R12.candidate.layout.json \
  planning/minnesota/revisions/gunflint-quality-v01/source-v10/MN_R12.candidate.json \
  planning/minnesota/revisions/superior-gunflint-quality-v01/cedar-source-v02/contract.json \
  content-packs/minnesota-superior-compact-v03-rebuild.crystalpack

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-superior-compact-v03.crystalpack --audio --desktop
```

Controller selectors use `CRYSTAL_COMPACT_SUPERIOR_TEST_PACK`; the two retained
town cases use `CRYSTAL_MARAIS_SUPERIOR_CONTROLLER_PACK`; source parity uses
`CRYSTAL_SUPERIOR_SOURCE_TEST_PACK`. Bind each to the absolute final pack path.
Asset publication uses `CRYSTAL_COMPACT_SUPERIOR_PARENT_PACK` bound to the exact
parent above. Preserve saves by setting the documented map/compact/town save
directory variables to a fresh ignored evidence directory. Production captures
always set `CRYSTAL_RENDER_TEST_HEADLESS=1` and
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`.

Workspace formatting and normal affected-crate Clippy remain blocked by
pre-existing unrelated differences/lints, including core multiplayer's denied
`bad_bit_mask`. Changed Rust leaves pass scoped rustfmt. No unrelated fixes,
staging, commits or generated-pack publication to Git were performed.
Supplemental lint passes with known existing lint allowances on its first run;
the final rerun has no compact-module warnings but encounters another chat's
concurrent `international_falls.rs:82` inaccessible `north_shore_story` reference.
