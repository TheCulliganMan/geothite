# Minnesota outdoor terrain integration

This cohort fixes Duluth's continuous upper plateaus and context-specific gray
stairs, removes high-cliff hop strips, closes broken retaining contours, moves
overhanging boulders inward, and repairs exterior stair landings across the
region. R10's western ridge, cache terrace and reachable grass cove are included.
The separately accepted Grand Rapids, Grand Marais and Rochester façade repairs
are replayed onto the same native pack and preserved exactly.

Selected as `outdoor_terrain_integrated_revision` in `world.json`. Fresh final
verification passes568 production MCP calls across nine cases, all377 audio
checks, and15 inspected native captures with zero differing pixels. The exact
selection/preservation record is
`renders/outdoor-stair-landings-v01/integration-local-proof-v155.json`.
Every previous world revision is retained.

`sources.json` records the15 ordered Rust builder stages and the two complete
current layout overrides. It is the authoring index for this cohort. Existing
canonical Duluth/R10 layouts remain guarded build preimages; starting another
edit from those files alone would discard the reviewed terrain. The other
maps' edits are explicit bounded plans over the native baseline, rather than
new exported game catalogs. Resolve the active revision in `world.json` before
building, rendering, or selecting another child.

The complete native candidate is
`content-packs/minnesota-current-outdoor-composed-towns-v01.crystalpack`, SHA256
`fcac136e08876c25d5c8b6250fc7c0fa1aedbfc6977e165504505b2b4d02016f`.
Current selection and verification status belong to `world.json`; the existence
of this file alone does not establish selection or statewide completion.

## Reproduce or extend

The native inputs start from Duluth cliff-stairs v03, SHA256
`6878059cebf9826b319b840bbcb2a8f243a73556c68d00cc0d135ad4a597265f`.
Follow the ordered source index. The builders require each actual parent hash,
native map preimages and fresh output filenames. Build with the pinned toolchain
using `CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1
RUST_MIN_STACK=67108864`; serialize heavy builds with other owners.

- Duluth: `revise_duluth_terrace_stairs PACK SHA256 BEFORE AFTER CONTRACT OUTPUT`.
- Landings: `revise_outdoor_stair_landings PACK SHA256 OUTPUT`; its row recipes
  are implemented in Rust and documented in the indexed JSON.
- Bounded cliff, ramp and north-flight plans: corresponding indexed builder,
  `PACK SHA256 PLAN OUTPUT`.
- Three R10 phases: corresponding indexed builder,
  `PACK SHA256 BEFORE AFTER DOSSIER ART_OUTPUT OUTPUT`.
- Town signs and façades: corresponding indexed builder,
  `PACK SHA256 BEFORE AFTER OUTPUT`.

Each completed stage is retained under ignored `content-packs/`. Rust APIs
enforce native art and map guards; the final production CLI additionally rejects
the superseded inward-turn recipe before writing. Use the current selected
parent for future edits and compose independent repairs; never reset selection
to a historical cohort parent.

Fast whole-town preview with an explicit current layout:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_duluth_cliffs.py \
  --pack content-packs/minnesota-current-outdoor-composed-towns-v01.crystalpack \
  --layout planning/minnesota/revisions/duluth-garden-terrace-v02/after.layout.json \
  --output planning/minnesota/renders/duluth-rear-left-plateau-v01/next-review
```

Once this cohort is selected, omitting `--layout` resolves the source index for
this exact pack. Explicit historical source review remains available. Source
previews are distinct from production screenshots and do not alter packs.

## Evidence and limits

`renders/outdoor-stair-landings-v01/composition-root-proof-v151.json` compares
the written pack independently against the common baseline, corrected geometry
and accepted town pack. Every data/binary root is the exact disjoint composition;
all three full accepted town modules remain exact. Relative to accepted Rochester,
270 metatiles change across23 maps and707 complete maps remain identical. Only
the separately documented Mina facing and private stair-art attributes accompany
those terrain edits. Audio remains exact.

The v66 parent-cohort production controller, roster, animation and both
all-targets Clippy runs pass. Its68 native views were all inspected. The four
later north-facing flights have separate40-call production input and four
inspected native captures. R10's final floor-ended corner has514 successful
production calls and11 inspected native views; v142 assets Clippy passes.

Fresh composed-pack verification is retained in the v152 audio, GPU, stair and
R10 stdio records. These are distinct from inherited parent receipts. Review all
new images before changing selection. Keep every original, crop, rejected pass,
save and log; generated evidence and packs remain ignored and uncommitted.

Initial party, position, badges and Repel are explicit local fixtures. Movement,
denials, reached-save Continue and Start afterward use the production controller.
This cohort does not establish earned campaign progress, reward collection,
complete statewide traversal, cave quality, or acceptance of every map's
composition. Existing story/source-pointer issues remain separately recorded.
