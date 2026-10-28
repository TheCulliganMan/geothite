# Connected Mississippi: Winona to St. Cloud

The current accepted native outdoor milestone is
`content-packs/minnesota-state-cohesion-v01.crystalpack`: **681 maps, 98 Minnesota
exteriors, one direct-seam component and zero footprint overlaps**. File SHA256
`c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb`;
content identity `11a5f3b39ffba6252fb2246af17433ed70b6b4d722d0b3a67394a1526845b03a`.
[The immutable Root acceptance receipt](../renders/outdoor-connections/native-state-cohesion-v01/root-review-v01/acceptance.json)
records the scoped native connection milestone; [state-cohesion.md](state-cohesion.md)
describes its four added wooded connectors and chapter checkpoints.

The complete Winona–St. Cloud itinerary still passes on this exact pack:
**1,269 native inputs, all seven bridge apertures, real Surf/Waterfall and
production Save/Continue**, plus the separate upper bridge Foot-road case.
`.local/state-cohesion-controller-v01/journey-v01.log` records both cases passing
in 68.63 seconds. Root also inspected the two actual GPU river views generated
from journey Saves: [Lower Woods](../renders/outdoor-connections/native-state-cohesion-v01/river-gpu-v01/lower-woods.png)
and [Upper Woods](../renders/outdoor-connections/native-state-cohesion-v01/river-gpu-v01/upper-woods.png).
Initial bank, level 80 Mew, moves, badges and RNG are explicit local fixtures;
this establishes native traversal, not earned campaign progression or every
route's Johto-level visual quality.

The historical river-only parent is
`content-packs/minnesota-mississippi-connected-v02.crystalpack`: 677 maps,
including 94 Minnesota exteriors. Its file SHA256 is
`6b36b4b835873e4c4508a7da7727ecb759273c26e219ec026c061c0e66f24775`;
content identity `de528e31f3d06f37c41b47595806d492d8279768f71d7b508ae7ab5315b66b5e`.
Rust publication and strict reload passed in `.local/mississippi-publication-v06.log`.

The brief is a long, quiet river through trees, with a little grass and a few
optional trainers. Seventeen native maps clip four hand-authored sources:
Lower Woods 64×68, Upper Woods 64×68, Confluence 74×12 and Winona mouth 16×4
metatiles. There are five optional bank trainers: Fishers Reed, Eli and Dale,
and Bird Keepers Ash and Pine, with native legal parties at levels 29–32.
Native art, existing city activities/rooms, chapter scripts and road links remain.
No additional quest, healer or pickup quota was imposed.

## Actual route and access

The native chain is Winona → Winona mouth → Lower Woods → St. Paul's eastern
waterfront → St. Paul's south outlet → Confluence → R03's river mouth →
Minneapolis/St. Anthony Falls → R05's southern water mouth → Upper Woods →
R05's northern water mouth → St. Cloud's eastern pond. Native reciprocal
connections implement the links; no transit warp substitutes for Surf.
Lower Woods cannot skip St. Paul. A visible six-metatile R05 rock bar prevents
the old continuous-Surf shortcut from bypassing Upper Woods.

Seven existing bridges now support walking above and Surf underneath:

| Apertures | Map | Water axis |
|---|---|---|
| Mill, Civic and South bridges | MnMinneapolis | north–south |
| City-hook bridge | MnR05C00R02 | east–west |
| Confluence bridge | MnR03C00R00 | east–west |
| West and East waterfront bridges | MnStPaul | east–west |

Original glyph references, deck Floor and rail Wall semantics remain exact.
Nine appended native-art copies carry explicit bridge permissions. Surf cannot
exit sideways through the elevated abutments. Three internal chunk joins are
closed forest adjacency, rather than water ports; every actual external river
port has reciprocal native Water lanes.

Ordinary Surf requires a partner knowing Surf and the native Fog Badge
prerequisite. Climbing St. Anthony Falls requires Waterfall and the Rising Badge.
The upper Minneapolis buoy gate independently requires **both Plain and Fog
Badges**, protecting inbound travel as well as outbound travel: native waterfall
current can carry swimmers down without Waterfall. Its three visible buoys open
through existing native callbacks. Permanent shore collision closes bank bypasses.
This publisher grants no badges, HM moves, items or chapter progress.

## Current river evidence and retained parent checks

- The current 681-map acceptance receipt records 14,941 independent preservation
  checks and 16 rejected corruption variants. All seventeen river modules and
  their bridge/encounter/program data remain exact across the cohesion stage.
- Historical 677-map river-only audit: **974 checks pass;11 corruption variants reject**,
  including missing landmark binding and wrong location. All 660 original maps,
  raw programs, rooms, parties/catalogues, audio, HOME/recovery and unaffected
  art remain exact except the explicitly declared terrain/connections.
- All 17 new native landmark bindings resolve existing seven-bit entries:
  Confluence→Minneapolis124; Lower Woods/mouth→Winona125; Upper Woods→St. Cloud's
  existing River Bluffs96 fallback. No landmark-table quota was consumed.
- All 17 published whole-map asset composites match frozen source pixels exactly.
  These are source-art composites from the written pack, **not GPU captures**.
- Native journey/upper-road regressions passed on both retained epochs:
  `.local/mississippi-controller-v01/journey-v02.log` records the historical
  677-map run (2 PASS, 98.31 seconds); the current 681-map run is the 2 PASS,
  68.63-second log cited above. Every later movement, HM prompt, bridge crossing
  and persistence action uses `VisibleShellController`. The explicit setup
  does not establish earned regional access or a fresh TUI/MCP playthrough.

Evidence: [independent audit](../renders/outdoor-connections/mississippi-independent/written-6b36b4b8-review-01/README.md),
[landmark-only epoch/render proof](../renders/outdoor-connections/mississippi-independent/native-6b36b4b8-render-01/README.md),
[current 98-map native state](../renders/outdoor-connections/native-state-cohesion-v01/full-state-native-current.png),
[current actual-pack itinerary binding](../renders/outdoor-connections/native-state-cohesion-v01/native-plan-binding-proof.json),
and [historical 677-map connected river image](../renders/outdoor-connections/native-mississippi-v02/winona-st-cloud-native-connected.png).

V01 remains archived: its first journey failed on the new Winona mouth's missing
town-map binding. V02 changes only 17 new location attributes, their 17 mirrors,
17 exact map-to-landmark bindings and derived identity; all 677 maps' geometry,
actors, programs, parties, art/audio and report remain identical to v01.

## Rebuild and fast review

Run from the repository root with the completed ignored parent pack present.
The frozen manifest is [mississippi-v02/manifest.json](../connections/mississippi-v02/manifest.json),
SHA256 `449ce63508b11513b187e5741ac8b8dc669d9fe420b1c922907dd301d4e2b814`.
Its exact parent is `content-packs/minnesota-twin-cities-compact-v01.crystalpack`,
SHA256 `2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e`.
All packs/build products remain ignored. These commands rebuild the historical
677-map river stage; the subsequent cohesion stage is documented in
[state-cohesion.md](state-cohesion.md). One publication owner runs the Rust CLI:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin build_mississippi_connections

target/runtime-fps/wasm-dev/build_mississippi_connections \
  content-packs/minnesota-twin-cities-compact-v01.crystalpack \
  planning/minnesota/connections/mississippi-v02/manifest.json \
  content-packs/minnesota-mississippi-connected-v02.crystalpack
```

The builder API is `build_authored_river_connections(base, manifest_json,
layout_texts_by_manifest_path, bridge_inventory_json)`. It checks native module,
cell, source-file and clipped-block preimages, free group82, reciprocal seams,
closed boundaries, actors' foot collision, and complete unrelated-data equality.
Only new maps use script lowering and uncached production assembly; existing
program/NPC payloads are untouched. Publication verifies the actual input file
SHA and reloads the written output. The successful build log is
`.local/mississippi-publisher-build-v06.log`; the CLI success log is cited above.

For a source-only loop, choose a fresh output epoch and retain every render:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_mississippi_connections.py \
  --manifest planning/minnesota/connections/mississippi-v02/manifest.json \
  --output planning/minnesota/renders/outdoor-connections/mississippi-source-REVIEW

.local/minnesota-review-env/bin/python planning/minnesota/tools/bind_mississippi_input_plan.py \
  --pack content-packs/minnesota-state-cohesion-v01.crystalpack \
  --expected-sha256 c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb \
  --expected-map-count 681 \
  --source-plan planning/minnesota/renders/outdoor-connections/mississippi-long-v07/source-all-bridges-input-plan.json \
  --manifest planning/minnesota/connections/mississippi-v02/manifest.json \
  --output planning/minnesota/renders/outdoor-connections/mississippi-bound-REVIEW
```

The reviewer reads the pinned parent and explicit source; the binder checks the
actual output collision/connection/bridge itinerary. Both are read-only and do
not play the game. A binder gate-open override exists only in memory for static
geometry; the controller must use real badge callbacks. Rebind after every pack
identity change. Preserve original hand strokes, row runs, all originals/RGBA/
conversions/crops/rejected renders, native saves and earlier failed logs.

## State-wide scope

The current accepted c481 native atlas places all 98 actual exterior maps in
one physical seam component, with zero inconsistent offsets or footprint
overlaps. The four wooded connector maps resolve the detached groups and four
old overlaps while preserving the river. This is physical integration, with
native chapter guards still applying.

For historical comparison, the archived 677-map river-only atlas had 94 exteriors
in ten direct-seam panels: a main 62-map group, 63 reachable through seams plus
explicit warps, 31 outside that graph and four inherited overlaps. Those counts
describe the older parent, not the current accepted state.

Five planned native hubs remain absent: Ely, Detroit Lakes, Moorhead,
International Falls and Mille Lacs. Connector routes do not implement those
towns. Separate Fairgrounds/Ely branches still require additive integration.
An earned campaign and the broader all-route Johto visual-quality objective
remain unfinished; the accepted connection milestone does not certify them.
