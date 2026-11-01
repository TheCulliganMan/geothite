# Rochester care green

This whole-town revision was selected for local map review as the final stage of
`content-packs/minnesota-reviewed-maps-v04.crystalpack`, SHA256
`47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48`.
It preserves the complete combined regional parent and all 742 other maps.
Nine current controller cases, an earned opening/first badge and two production
stdio sessions pass. Broader map quality and the full campaign remain open.
It is preserved exactly inside the current sixteen-stage v05 selection; the
[Grand Rapids successor](../grand-rapids-depot-quay-v01/README.md) adds the next
town revision without changing Rochester.

- [Complete installed town](../../renders/rochester-care-green-installed-v1351/after-whole-original.png)
- [Complete before/after](../../renders/rochester-care-green-installed-v1351/before-after-original.png)
- [Exact layout and anchors](source-v01.json)

## Design and current source

The old central lawn separates the clinic, patient house and Gym without giving
that walk enough purpose. Bring the two southern buildings onto one ordinary
street, retain the northern clinic/fossil/Mart frontage and give the gardener a
small brick-and-flower court. The clerk/patient lane bends around the house;
the south street stays two walking tiles wide and connects both real mouths.
The guide stands in a side pocket above it. No added Cut, ice, water or forced
town encounters are used to make this revision seem more complicated.

The exterior remains 20×16 metatiles so the real regional footprint, both
connection offsets and four mouth cells remain intact. Its unused southern lawn
becomes the wooded town edge. The inhabited area and useful trips become tighter;
this is a recomposition of actual terrain, not a cropped or resized image.

Parent: `content-packs/minnesota-reviewed-maps-v03.crystalpack`, SHA256
`f79f8a3226ef2b4d97d25b546871359797f006265beaa5dcded344024ec5e89d`.
Complete native Rochester module, canonical JSON SHA256:
`6ece567f1311540d9e15fa6410f27c09e979e145930be4e190d1b4350dbb4f2e`.
Current `source-v01.json` SHA256:
`3a58e14c9bd788e6b350c275966437b01e55199ff81d1564162bbb927e4a1296`.
It contains both complete row sets, 117 changed metatiles, exact native building
rectangles and all moved anchor preimages. All source pixels/palettes are reused.

| Useful walk | Before | After |
| --- | ---: | ---: |
| Clinic → Gym | 31 | 21 |
| Clinic → patient house | 24 | 18 |
| Fossil lab → Gym | 21 | 13 |
| Patient house → Gym | 23 | 19 |

These are shortest native FLOOR paths with actual NPC bodies. They are static
measurements; the actual door, service and story journeys also pass separately.

## Exact entrances, actors and signs

Coordinates below are 16px walking tiles. Warp identities stay fixed, so every
interior retains its original return index and all current room content.

| Destination | Door | Passive exterior return | Outdoor warp indices |
| --- | --- | --- | --- |
| Clinic | (15,7) | (15,8) | 1 / 2, unchanged |
| Mart | (29,9) | (29,10) | 3 / 4, unchanged |
| Gym | (26,17) | (26,18) | 5 / 6 |
| Clerk house | (7,7) | (7,8) | 7 / 8, unchanged |
| Patient house | (7,17) | (7,18) | 9 / 10 |
| Fossil lab | (23,9) | (23,10) | 11 / 12, unchanged |

The patient house's complete native 2×2 assembly moves from metatile (3,10) to
(3,7). The Gym's complete 3×2 assembly moves from (14,10) to (12,7). Roofs,
wall edges and door quadrants remain complete original assemblies.

`MN_RC_CITY_GUIDE` moves (28,16)→(23,17); the gardener remains (19,11).
Both prairie wardens remain (2,18)/(2,19), retaining their existing first-badge
gate, free return behavior and scripts. East connects to `MnR01C00R00` with
offset −3; west connects to `MnR02C00R00` with offset 2. Their exterior mouth
cells remain (0,18)/(0,19) and (39,18)/(39,19).

The close native tile check finds **one** original misaligned sign: the clinic's
event (19,8) occupies FLOOR beside its visible post. Move it to the post's WALL
quadrant (18,8). The other original sign quadrants are correct. Gym/city signs
move to (30,16)/(34,16) alongside the new frontage. Mart (32,10) and fossil lab
(26,10) remain fixed. Every final sign occupies an actual solid post and has a
reachable adjacent reading tile.

The first source view mistakenly used sign variant 65 as a planter. That view,
its exact source snapshot and all references remain in the rejected v1344
output. The inspected v1345 replaces those extra signs with native brick 06 and
flowers 04. Do not restore the decorative sign clutter.

## Fast source loop and installed review

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_rochester_care_green.py \
  --output planning/minnesota/renders/rochester-care-green-FRESH
```

The executed v1345 review takes 0.092 seconds after pack loading. It checks the
complete preimage, whole native assemblies, six doors, all passive approaches,
actors, readable solid sign quadrants and all four shorter trips. It retains
the full source snapshot, native terrain/actor originals and before/after.
Every invocation requires a fresh ignored output. Both v1344 and v1345 were
viewed; source checks alone do not certify native gameplay.

Add `--installed content-packs/minnesota-reviewed-maps-v04.crystalpack` to compare
the actual published typed map with the source projection and render its native
town. The executed installed review takes 2.916 seconds after loading the parent,
including loading/checking the child. Its whole before/after and five production
player cameras were inspected. The source JSON's frozen status records its
prepublication authoring epoch; `world.json` records the live selection.

## Rust publication and exact preservation

`revise_rochester_care_green` applies the explicit rows, four exterior warp
positions, guide and three signs to the exact parent above. It checks complete
building assemblies, the full map preimage, public paths, native signs/doors,
the existing clear Fly/recovery spawn and the unchanged perimeter. Its inverse
native map replacement must recover the complete parent. The major-city
recomposer's allowlist remains unchanged.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_rochester_care_green
target/verdigrim-scene/debug/revise_rochester_care_green \
  content-packs/minnesota-reviewed-maps-v03.crystalpack \
  f79f8a3226ef2b4d97d25b546871359797f006265beaa5dcded344024ec5e89d \
  planning/minnesota/towns/rochester-care-green-v01/source-v01.json \
  content-packs/minnesota-rochester-care-green-FRESH.crystalpack
```

The actual standalone output is `minnesota-rochester-care-green-v01.crystalpack`.
The [fifteen-stage rebuild](../../revisions/reviewed-maps-v01/README.md) produces
the selected v04 bytes identically from the coastal parent without intermediate
packs. A second standalone run also passes the later explicit spawn guard.
`.local/rochester-reviewed-build-1793507072509902000/receipt.json` records both
builds, fourteen-stage compatibility, all 602 PCM/hash/frame/loop and desktop
checks, and all-target assets Clippy. Eleven publisher refusal cases and nine
corrupted-index cases pass in the corresponding `rochester-publisher-guards-*`
and `rochester-source-index-guards-*` directories under `.local/`.

The independent whole-root audit in `renders/rochester-care-green-preservation-v1348/`
matches the explicit map, native event wire and required NPC/block mirrors.
Every other complete map, program, room, flag, encounter, roster, art and audio
remains exact. `renders/rochester-reviewed-sources-v1355/` binds all 28 current
native layouts, six route groups, twenty sources and the eight moved anchors.

## Played acceptance

[The selection receipt](../../renders/selected-rochester-care-green-v1356/receipt.json)
lists only the nine successful cases and hashes their exact evidence:

- Six doors, all twelve active exit carpets, five solid readable signs, actual
  gardener collision, Start and Continue: the first successful case in
  `renders/rochester-care-green-acceptance-1793507550252897000/`.
- Both lanes of both roads, zero-badge inward passage, native Care refusal and
  qualified badge exit/return with Continue (explicit local fixtures):
  `.local/rochester-care-green-road-final-1793507664160712000/`.
- Real HP/status/PP healing, clinic recovery binding, PC input ownership, exact
  priced Potion purchase and Continue; all three actually awarded rods/casts;
  three early-starter/fossil research cases:
  `renders/rochester-care-green-services-1793507620845900000/`.
- Fresh New Game delivery and complete home return, plus route trainers,
  training/capture, rival and earned first Gym badge/TM with Continue: the two
  successful earned cases in
  `renders/rochester-care-green-earned-1793507010544642000/`.

The earned course uses ordinary New Game and input, with no injected partners,
levels, progress, results or RNG. Local road/service/rod/research setups remain
explicit fixtures. Their scope is not a complete earned campaign.

The new tests initially counted passive arrivals as carpets, assumed a 64-tile
prairie edge instead of its actual 60-tile width, expected a generic PC label
instead of the authored wrapper, and assumed one frame sufficed after the
inward gate animation. Corrected tests retain the native scripts and terrain.
Every failed run, source render and original checkpoint remains available.

`tools/check_rochester_care_green_mcp.py` continues the actual earned Care Badge
save through moved doors, clinic return, guide/signs, courtyard and both first-
route lanes, using production input across two fresh processes. It passes
285 calls with byte-empty stderr in `renders/rochester-care-green-mcp-v1354/`.
It never grants or rewrites position, party, badges, resources or saved terrain.

Five actual player cameras were inspected in
`renders/rochester-care-green-camera-1793507160993917000/` and
`renders/rochester-care-green-earned-cameras-1793507665306650000/`.
The refreshed [complete outdoor atlas](../../renders/rochester-reviewed-state-v1353/full-state-native-current.png)
retains all 108 maps and actual offsets: no overlapping footprints, inconsistent
placement cycles, missing planned hubs or unreachable outdoor graph nodes.
Graph reachability is static, separate from qualified controller travel.

Current affected-crate builds, scoped rustfmt and all-target Clippy pass.
The final door-test build and replay also pass in
`.local/rochester-care-green-door-lint-1793508343470142000/`, with no new
warnings in the owned map/publisher/test files. After selection, the source
loop still takes 0.118 seconds after loading, and the default v04 source audit
passes in `renders/selected-rochester-source-loop-v1357/` and
`renders/selected-rochester-default-index-v1357/`.
Workspace formatting retains unrelated differences beginning in the offset
allocator. Existing player saves and their terrain overrides are not migrated;
use a separate review save. No browser release or user visual approval is claimed.

The opening, first-route delivery and early research already work on the parent;
see [the current opening review](../../revisions/reviewed-maps-v01/OPENING-REVIEW.md).
The current child now repeats its earned delivery/first-badge and research
courses through the preserved clinic and actual installed door/road positions.
