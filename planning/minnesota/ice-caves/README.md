# Current Voyageurs ice source

The selected terrain now uses the [turning-chambers revision](turning-chambers-v01/README.md). Resolve `planning/minnesota/world.json` for its current pack and inherited source index. The canonical source contains the current complete floorplans; frozen previous rows and all renders remain retained.

The initial-publication commands and 683-map results below are historical evidence. They do not establish the current turning layout; use the linked current runbook for new work.

# Initial Voyageurs Ice Cavern publication

The requested Minnesota ice dungeon is a new, optional two-floor destination
beneath the Voyageurs ridge. It uses the supplied pack's original `ice_path`
metatiles, collisions, special blue palette and native forced sliding. Crystal's
five original Ice Path maps and Hall of Fame remain unchanged.

The entrance is `MnR20C01R00 (6,33)`, global R20 `(70,33)`. A short dry trail
branches north from `(6,36)`; the safe exterior return is `(6,34)`. Existing R20
regional badge/story access is retained. The dungeon adds no new HM or fee gate.

The [explicit authoring source](voyageurs-ice.json) owns both complete floorplans,
actors, pickups, paired warps and the two exterior metatile edits. Source renders
and all rejected passes remain in the ignored
`planning/minnesota/renders/ice-caves/voyageurs/` directory.

| Floor | Size, metatiles | Play |
| --- | --- | --- |
| MnVoyageursIce1F | 14×12 | Sheltered arrival, guide, teaching slide, separate Ice Heal pocket, downward ladder |
| MnVoyageursIceB1F | 16×12 | Rock-stop puzzle to the northern NeverMeltIce shelf, optional Max Revive pocket, return ladder |

The lower chamber's west pocket is separated from the dry return ledge. Its
southeastern rock shoulder and central buttress shape the sliding routes. There
is no walkable southern band that bypasses the ice. Rewards are ordinary native
item balls with independent persistent event flags. No Strength boulders or pits
are used, so leaving a puzzle cannot strand a required movable object.

## Build and review

Executed Rust publication command, from this checkout:

```sh
target/ice-dungeon-publish/wasm-dev/pack_voyageurs_ice \
  content-packs/minnesota-cave-quality-v01-09.crystalpack \
  planning/minnesota/ice-caves/voyageurs-ice.json \
  content-packs/minnesota-voyageurs-ice-v02.crystalpack
```

Build that executable using the pinned toolchain:

```sh
CARGO_TARGET_DIR=target/ice-dungeon-publish CARGO_BUILD_JOBS=2 cargo build \
  --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  -p crystal-assets --features test-fixtures --bin pack_voyageurs_ice
```

The builder checks the exact exterior preimage, native actor terrain, slide
reachability, safe returns, unique identities and strict pack readback. It adds
maps through the existing Rust publisher and synchronizes the changed R20 split
payload without reparsing or replacing its story scripts. Generated packs stay
ignored.

Source review, including slide-input planning rather than ordinary floor fill:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_voyageurs_ice.py \
  --pack content-packs/minnesota-voyageurs-ice-v02.crystalpack \
  --base content-packs/minnesota-cave-quality-v01-09.crystalpack \
  --published --output planning/minnesota/renders/ice-caves/voyageurs/published-source-v03
```

The native capture script `tools/review_voyageurs_ice_native.py` captures every
listed camera headlessly and compares full frames against the same source
position/time/facing/palette. Original GPU captures and all derived comparisons
are retained separately. Source planning is not production gameplay proof. The slide planners now apply the native
`UP_WALL` entry/departure rules: sideways entry creates a dry stop, while north
departure and south entry are blocked.

## Verification status

The 683-map overlay is published and passes strict Rust readback. Its SHA-256 is
`546f17a16db8e47b409e870f821a826dd5a9b9f6d8dd0c526d8cf5ad397c9f0f`.

- Source slide graphs reach all rewards and ladders; every reachable stop can
  return after all item bodies are removed.
- All nine full 640×576 production frames match source art: zero differing
  pixels across 3,317,760 pixels. Both complete camera sheets were visually
  inspected alongside whole-room compositions and the original Ice Path maps.
  The accepted receipt is `../renders/ice-caves/voyageurs/native-v02/capture-progress.json`.
  The location tester supplies its disposable party fixture; it does not earn
  regional access. The separate gameplay test uses an empty-party local fixture.
- The native atlas scrolls glyphs `$b5` and `$b1` on a shared 19-frame clock.
  The comparer uses that coherent schedule. The initial static comparison's
  513-pixel difference and all original GPU frames remain retained.
- Preservation checks retain 680 existing maps exactly, including all nine
  improved stone caves, all original Ice Path maps and Hall of Fame. R20 changes
  only two metatiles and appends a paired entrance/return; original scripts,
  actors, warp records, HOME/recovery and unowned data tables remain unchanged.
- Rust `verify_crystalpack --audio --desktop` passes every PCM hash, frame count
  and loop range for all 377 audio programs (`.local/ice-audio-v02.log`).
- The complete production controller course passes on v02, including all three
  real reward quantities/flags, forced ice input, both ladder returns,
  Save/Continue at an ice stop and same-mouth exit/reentry
  (`.local/ice-controller-final-v06.log`).
- Stdio MCP replays the actual exported exterior checkpoint, enters/exits twice,
  opens the production Start menu, plays the guide, rejects movement during
  dialogue, saves and continues. Stderr remains byte-for-byte empty
  (`.local/ice-mcp-receipt-v01.json`, `.local/ice-mcp-v01.log`).
- The production battle-menu/selected-move/PP/dialogue regression also passes
  on the final ice pack (`.local/ice-battle-v02.log`).
- Rebuilding with the final publisher reproduces v02 byte-for-byte
  (`.local/ice-final-rebuild.log`). Map groups use the first free slot in
  `84..255`, permitting publication after a northern extension already using84.

The rejected v01 is retained as test history. Its missing typed pickup entries
were caught by the gameplay test. The publisher now compiles and requires those
entries, using the same native parser as the existing authored caves.

The required all-target Clippy run stops in the pre-existing
`crystal-core/src/multiplayer/mod.rs:1332` `clippy::bad_bit_mask` failure
(`.local/ice-clippy-v01.log`). The check without dependency linting also stops on eleven pre-existing
`clippy::never_loop` errors in `mutation_protocol.rs` and `script_parsing.rs`
(`.local/ice-clippy-no-deps-v01.log`). A final check with existing lints capped
also reaches the new library/test code without ice-file diagnostics, then stops
on an unrelated in-progress winter-art binary import
(`.local/ice-clippy-current-v02.log`). These are not passing full-workspace lint
runs. Scoped Rust formatting passes; the workspace formatting check retains
unrelated differences (`.local/ice-workspace-format-final.log`).

Production course command:

```sh
RUST_MIN_STACK=67108864 \
CRYSTAL_ICE_DUNGEON_PACK="$PWD/content-packs/minnesota-voyageurs-ice-v02.crystalpack" \
CRYSTAL_ICE_DUNGEON_CHECKPOINT="$PWD/.local/ice-continue.crystalsave" \
CARGO_TARGET_DIR=target/ice-dungeon-publish CARGO_BUILD_JOBS=2 cargo test \
  --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-runtime.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  -p crystal-bevy --lib voyageurs_ice_controller -- --ignored --test-threads=1
```

The course enters from R20 through ordinary input, checks the guide's visible
pages and dialogue ownership, follows native slide stops, tests perpendicular
input during a slide, collects all three rewards once, saves/continues at an
ice stop, returns up both ladders, exits/reenters the same mouth and checks
persistent flags. Initial local position/party setup does not prove earned
regional progression.

MCP continuation, using the actual checkpoint produced by that course:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_voyageurs_ice_mcp.py \
  --client target/gym-controller/wasm-dev/geothite \
  --pack content-packs/minnesota-voyageurs-ice-v02.crystalpack \
  --load .local/ice-continue.outside.crystalsave \
  --save .local/ice-mcp-continue.crystalsave \
  --report .local/ice-mcp-receipt-v01.json
```

The same built client can open that local review continuation interactively:

```sh
target/gym-controller/wasm-dev/geothite play \
  content-packs/minnesota-voyageurs-ice-v02.crystalpack \
  --load .local/ice-mcp-continue.crystalsave
```

This is an empty-party review checkpoint with all three rewards already
collected. It is separate from an earned regional campaign. No visible game
window or fullscreen terminal was opened during automated verification.

The original reference inventory is IcePath1F 20×18, IcePathB1F 10×18,
IcePathB2FBlackthornSide 5×9, IcePathB2FMahoganySide 10×9 and IcePathB3F 10×9.
These authentic source renders are retained under `../renders/ice-caves/audit-v01`.
