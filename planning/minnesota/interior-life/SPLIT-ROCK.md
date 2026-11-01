# Split Rock shore clinic

Publication lineage correction: the pack receipts below are retained history.
Current source refuses their obsolete coastal geometry; preserve the rooms and
[rebase onto the compact coast](COMPACT-REBASE.md) before further publication.

Current scope correction: Red Wing's train station is excluded by user direction.
The [current five-stop publication](RAIL-STOP-CORRECTION.md) supersedes earlier
station reservations below; historical pack/test receipts remain unchanged.

The current interior child is
`content-packs/minnesota-interiors-split-rock-v01.crystalpack`: 717 maps,
475 species, 270 moves, 377 audio programs and 216 non-gym interiors.
File SHA-256: `978ab6e3f674acf72967ca2f6dba799d590b47f34ef855ea4cfb8cfa7c9c5397`.
Runtime content hash: `3430117b94738bed99ecc020e326d9277d240b7ebd96fe7802212c6c00b21a39`.
It builds on the protected [fishing-home child](FISHING-HOUSES.md).
Current visual work is native 2D only.

The [authoritative clinic source](split-rock-clinic.json) retains the native
Cherrygrove 5×4 floorplan, replaces its unused link stair with ordinary floor,
and supplies three people:

- The nurse provides the production `PokecenterNurseScript`, free HP/status/PP
  care and one dynamically registered clinic recovery point.
- A shore grandfather mends mittens while his granddaughter cleans the
  lighthouse lens; their slow walk home connects the clinic to Split Rock.
- A deckhand recalls turning back in fog and the nurse's reassurance that
  coming home matters. The conversation grants no reward or progression flag.

The PC binds the actual native `PC` collision at `(9,1)`, approached from
`(9,2)`. The separate earlier clinic build's `(8,0)` read target is wall
collision; its original artifact remains historical evidence. This child uses
the real machine and verifies the production PC menu and movement ownership.

## Physical integration

The original `MnR11C00R02` module is identical in the protected interior parent
and compact geography parent. Fifteen bounded native cells reproduce the
complete existing south-bank facade and forecourt. The new door `(33,33)` and
passive return `(33,34)` append warp indices 3/4 after the original lighthouse
pair. Indoor carpets `(3,7)/(4,7)` return to index 4; passive arrival 3 is
`(3,5)`. A reachable placard explains that care is free.

All four original southern lanes `(34..37,47)` connect to the porch in native
collision checks. The controller walks from `(34,47)` through the actual door.
Every outer-edge collision, original connection, river, cliff, actor, trainer,
item and story program remains exact. No west edge is opened.

The separately published Ely snow/back-trail revision depends on the compact
coastal world layout. This clinic composition does not claim that whole
geography revision. Its wider integration and the canonical Cedar/gate rebase
remain open; the new clinic and existing interior stories must survive them.

## Executed Rust build and publication

Run from this checkout with its pinned toolchain:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_split_rock_clinic \
  --config 'profile.wasm-dev.opt-level=1' \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0'
target/interior-life/wasm-dev/pack_split_rock_clinic \
  content-packs/minnesota-interiors-fishing-life-v02.crystalpack \
  planning/minnesota/interior-life/split-rock-clinic.json \
  content-packs/minnesota-interiors-split-rock-v01.crystalpack
```

The publisher requires a fresh output, writes the pack, rereads it strictly
and compares the written result to the validated in-memory pack. Use fresh
output names on a repeat and a new save for the resulting identity. Packs and
all build/render products remain ignored.

## Verification

The external-pack asset regression passes (82.54 seconds). It establishes
identity-equivalent reconstruction from the source, exact original maps
including every gym, exact existing shore programs and battle data, typed/raw
native reconstruction, original assets/audio, old recovery/home records,
landmarks, metadata and unrelated game catalogs. Only the clinic's new recovery
record is added. Ten unsafe source mutations and duplicate installation are
rejected, including altered floorplans, native PC positions, nurse counters,
residents/services, carpet targets and facade preimages/extent.

Both production-controller cases pass on the written child (30.45 seconds):
public road approach, real entry, both local conversations and text input
ownership, both carpets, repeated entry, free-care placard, Start and recovered
movement, public Save/Continue, free HP/status/PP restoration, native PC opening
and ownership, actual walking poison damage, blackout to this clinic, native
money halving and Continue after recovery. Bag, money and logical flags remain
unchanged in the conversation/return case. Care preserves badges and costs zero.

Position, a legal regional party, injuries and a later explicit poisoning/money
fixture are setup. The poison phase retains the recovery selected through the
real nurse and never rewrites position or a warp; native inputs cause the faint,
transition and arrival. Save/Continue uses the public controller save/load API.
These focused cases do not claim earned regional campaign progression.

The first publication rejected the misplaced PC target. The first controller
run also exposed two test-fixture errors: four different directional presses
turned in place without walking, and raw flag-map equality treated newly
stored false values as progression changes. The corrected fixture proves real
movement and compares the production flag getters' false/missing semantics.
Both original failed logs remain retained.

All six complete 640×576 headless native 2D frames match the independent source
compositor exactly: 2,211,840 pixels compared, zero exclusions or differences.
The complete room, shore chunk and all six native views were personally
inspected. All 377 audio PCM hashes, frame counts and loop ranges match. Capture
left the pack unchanged and removed its private executable.

Current source/native originals, raw GPU descriptors, crops, conversions,
differences and authoring-program snapshots are retained under
`planning/minnesota/renders/interior-life/split-rock/`. The earlier PC source is
retained in `revisions/split-rock-clinic-v01-pc-preimage.json`. Verification logs
are `target/interior-life/split-rock-{publication-v02,preservation-v01,controller-v02,source-comparison-v01,audio-v01}.log`.

Changed Rust leaves pass rustfmt. Whole-workspace formatting still fails first
in the existing `offset-allocator` formatting differences. Normal affected-crate
all-target Clippy stops at the existing `crystal-core` `bad_bit_mask` denial;
the all-target check for both affected crates completes successfully with only
that and the previously known `never_loop` denial allowed (3m35s). It reports no
diagnostics in the new clinic leaves. The separate receipt is
`split-rock-clippy-v02.log`.

Every current non-gym reservation is now present: 215 registered rooms plus the
additional Rainy River warming hut make 216. The two planned Staples/Red Wing
stations preserve the minimum 218-room scope. Their publication, full Glacier
Line behavior, canonical geography composition and regional native 2D quality/
service audit remain open. This clinic does not complete the all-interiors goal.
