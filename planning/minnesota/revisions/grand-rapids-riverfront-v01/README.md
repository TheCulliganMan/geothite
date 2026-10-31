# Grand Rapids riverfront

This complete 24×24-metatile candidate replaces the broad empty town clearing
with a pine-bounded service circuit and a winding riverbank loop. The three
original doors, their complete native facades, all existing rooms and both
guarded regional roads retain their exact positions. The southern walk reaches
a stone fishing pier. KNUTE offers an optional battle beside a passing lane;
the end of the pier supports the town's original regional fishing table and
ordinary badge-gated Surf access to the contained backwater.

The verified local pack is the ignored
`content-packs/minnesota-grand-rapids-riverfront-v01.crystalpack`. Selection is
controlled by `world.json`. This is one completed town improvement within the
ongoing regional map review; it does not establish earned campaign arrival or
completion of the whole state.

[Whole-town before and after](../../renders/grand-rapids-riverfront-v659/whole-town-v675/whole-town-before-after.png),
[twelve native player views](../../renders/grand-rapids-riverfront-v659/native-v673/production-decisions.png),
[four actual reached-save views](../../renders/grand-rapids-riverfront-v659/acted-v676/actual-reached-views.png).

## Authoring and fast review

`after.json` is the entire explicit town, with native port art recipes and
twelve player-scale viewpoints. `before.json` and `contract.json` bind the
exact original town, event anchors and programs. `trainers.json` authors KNUTE's
dialogue and original Minnesota team; the existing regional scaling applies.
`plan.json` records the two-core-tile streets, real doors, actor approaches,
guard lanes and pier quadrants. Coordinates marked `core` are 16px walking
tiles; source rows use 32px metatiles.

The source renderer reuses the supplied pack's graphics and palettes, retains
every whole-town revision and has a measured warm render time of 0.094 seconds.
Use a fresh output directory, or retain the existing watcher and its immutable
history:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-gunflint-spring-shelter-v02.crystalpack \
  --source planning/minnesota/revisions/grand-rapids-riverfront-v01/after.json \
  --output planning/minnesota/renders/grand-rapids-next-source-review \
  --palette day --watch
```

The renderer's filename is historical; its explicit native-room rows also
support this town. Source previews do not prove services, battles or movement.
All current renders, including the original three-tile port art inspection and
rejected checks, remain under
`planning/minnesota/renders/grand-rapids-riverfront-v659/`.

## Implemented geometry and gameplay contracts

- The west road meets `MnR09C01R00`; the east road meets `MnR10C00R00` with its
  original seven-metatile offset. All eight coordinate events and four officers
  preserve the depot handover gate and free inbound access to care.
- The clinic entrance is `(9,11)`, depot `(27,13)` and mart `(39,15)`. Their
  passive return positions remain one walking tile south. All six warp records
  are unchanged. The clinic retains healing, PC and blackout recovery; the
  mart retains its real priced inventory; the depot retains its acknowledgment,
  refusal/retry and prerequisite checks.
- All six original town actors and three reading boards stay exact. The
  ordinary streets are two walking tiles wide, with small service forecourts
  and the original native shore railing. There are no new Cut gates, ledges,
  stairs, encounter grass or misleading exterior doors in this town.
- KNUTE is at `(25,36)`, approached from `(26,36)`. Declining must leave party,
  money and story flags unchanged. Accepting starts a real FISHER battle with
  SCUTLET29 and TORROUT31 before scaling. Victory must consume native move PP,
  award the native prize, set only his own completion event and survive
  Continue. It cannot substitute for the depot foreman, instrument or handover.
- The pier has a two-tile stem and a visible paved cap. Its right side remains
  walkable past KNUTE. `(26,38)` is FLOOR and `(26,39)` WATER. Without the move
  or Fog Badge, the player must remain on the cap. Qualified input must mount,
  paddle, survive Continue, dismount and use the registered Old Rod there.
- The backwater remains contained by its original shore and outer pine buffer;
  no unconnected Surf edge or new water seam is introduced. The three imported
  port blocks preserve original pixels and collision. Their native water glyph
  reuses the outdoor animation destination. The renderer explicitly inherits
  `johto_modern` water and flower animation for this private tileset.

## Native build and acceptance

The implementation is in focused Rust leaves
`grand_rapids_riverfront.rs` and `grand_rapids_riverfront_art.rs`, with the
`rebuild_grand_rapids_riverfront` Rust CLI. One owner serializes Cargo and pack
publication. Read the active `world.json` parent again before a new build or
selection; do not use this historical command to overwrite newer content.

The completed candidate publication used:

```sh
target/verdigrim-scene/debug/rebuild_grand_rapids_riverfront \
  content-packs/minnesota-gunflint-spring-shelter-v02.crystalpack \
  6a4af10d7d025cb9041878a63855c6d01cf4e5180c2672cf10a07b0ceda5fecb \
  planning/minnesota/revisions/grand-rapids-riverfront-v01/before.json \
  planning/minnesota/revisions/grand-rapids-riverfront-v01/after.json \
  planning/minnesota/revisions/grand-rapids-riverfront-v01/contract.json \
  planning/minnesota/revisions/grand-rapids-riverfront-v01/trainers.json \
  content-packs/minnesota-grand-rapids-riverfront-v01.crystalpack
```

The output must be fresh and ignored. The publisher strictly reads back the
result and refuses stale preimages, damaged facades, disconnected approaches
and invalid regional teams. The read-only
`planning/minnesota/tools/audit_grand_rapids_riverfront.py` independently checks
all other maps, original programs, rooms, global content and runtime art/audio,
the three native port recipes at every time palette, current roster closure
and the complete dry graph. Only the eight intentional inbound guard pockets
may be separated from the town by actor bodies.

Run the default-profile `gr_riverfront_controller` cases with
`CRYSTAL_WINONA_TEST_PACK` pointing to the completed candidate and a fresh
`CRYSTAL_GR_RIVERFRONT_REVIEW_DIR`. Run all six existing
`grand_rapids_controller` cases with `CRYSTAL_GRAND_RAPIDS_TEST_PACK` set to
that same candidate and `CRYSTAL_GRAND_RAPIDS_SAVE_DIR` set to a fresh output.
All four new cases and six existing cases pass. They cover the new activity,
all-time native animation, existing depot, native services and both real seams.
All initial party, badge, item and position fixtures are declared; subsequent
play uses production controller input without state edits. The optional battle
consumes move PP and awards its native prize. Its result survives Continue;
declining does not grant progress or consume resources. Both missing-move and
missing-badge fixtures remain on the pier, while qualified play mounts, paddles,
continues in Surf mode, dismounts and produces a real regional fishing battle.

The first twelve captured player views used an older renderer without the new
animation binding and remain retained as that explicit version. All twelve
fresh views from the rebuilt renderer match the source at every pixel; they
and the four reached-save views were personally inspected. The latter preserve
the actual saved night palette, position, facing, party and battle state.

Independent stdio-MCP courses pass 240 RPC calls: the complete town loop and
optional refusal, then a real registered Old Rod bite and native RUN/return.
Native save inspection confirms SCUTLET and the correct bank position. Both
processes retain byte-empty stderr. All 377 audio programs retain their PCM
hashes, frame counts and loop ranges; the exact-parent audit preserves all
731 other maps and all original runtime art, services and encounters.

Affected-crate all-target Clippy passes, including `location-tester` for the
renderer, with existing warnings and none in these new leaves. All changed
Rust leaves pass formatting. Workspace-wide `cargo fmt --all -- --check`
still reports pre-existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`; those unrelated files were preserved.

`current.json` binds this complete exterior and its actual original doors,
actors and events. `sources.json` composes it over the latest selected parent
source index. Shared interior reservations need no edits because every room,
door, arrival, return position and regional port remains exact.
