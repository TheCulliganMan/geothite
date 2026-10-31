# Coastal passing-width integration boundary

Verified final candidate:
`content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack`

SHA256: `e951fddafee3a0c09a2356d4dd683890ac498e23daeb266d08c5f12d79ec1167`

Exact parent: `content-packs/minnesota-fauna-progression-title-v01.crystalpack`
SHA256: `8ea32c5704ae8e763941295e14bd035ebbeac9e0e6d84018d599aa1dd5b0c504`.
The newer fauna/title content, all602 audio programs, all225 active Minnesota
species references, and the736 other complete maps are preserved. Seven maps
change: both towns, R11C00R00..03, and R11C01R00.

At this boundary the local planning review selector has ALREADY advanced to
`north_shore_passing_width_revision`; see `world.json` and the retained
`world-preimage-v1035.json` in the verification directory. This metadata update
occurred before the later integration handoff message. No live app, renderer
window, web deployment, process restart or running-client pack was changed.
The final pack is a local reviewed candidate, not a deployed release.

The primary checkout is `feat/connected-johto-3d`. Minnesota itself must remain
native2D-only, including menus, preferences, shortcuts and previews, as required
by the user's guidance and AGENTS.md. The general branch name does not authorize
enabling Minnesota3D/2.5D. Preserve the new precipitation policy's explicit warm
Boundary Waters/Safari exclusions when combining renderer changes.

Owned coastal source:

- `crates/crystal-assets/src/north_shore_passing_width.rs`
- `crates/crystal-assets/src/north_shore_passing_width_tests.rs`
- `crates/crystal-assets/src/bin/rebuild_north_shore_passing_width.rs`
- The module/export registration beside the older footbridge module in lib.rs.
- `crates/crystal-bevy/src/bevy_shell/tests/north_shore_passing_width.rs`
- The corresponding test-module registration in bevy_shell/tests.rs.
- `planning/minnesota/tools/review_north_shore_footbridges.py`.
- This planning folder, both canonical town dossiers, and the new world/source
  bindings. Preserve unrelated work in shared registration/world files.

The README records the exact overlay CLI. A combined parent may be passed only
with its real SHA and identical complete ten-map terrain/checkpoint preimages;
the publisher preserves that parent's programs/resources. New checkpoint rows
must cover every widened lane. Do not restore the rejected narrow84 bridge as
the current source or drop newer story/fauna changes by selecting an old pack.

Final proof root: `planning/minnesota/renders/north-shore-width-fauna-v1029/`.
`acceptance-v1035.json` records three passing controller cases, all28 deck lanes,
16 outward denials, the1885-call no-Surf stdio trip,12 additional inward cases,
actual Continue, ten inspected reached GPU views, all602 audio checks, strict
parent preservation and an identical rebuild. Existing380grass quadrants and
required4step/14step grass costs remain. Current source bindings were checked
again in `current-source-bindings-v1036.json`. Workspace formatting has unrelated
existing differences; both affected-crate all-target Clippy and owned formatting
pass. Every render, save, failure and prior candidate is retained and ignored.

Current shared Cargo target is `target/verdigrim-scene`; no map-owned build or
renderer process remains running at this handoff. Disk is below the AGENTS15%
free-space threshold, so do not start another substantial build until space is
recovered. Preserve the shared live cache and retained evidence/packs/saves.

Pending broader map work remains explicit: rebase the already-reviewed newer
Ely pine-camp candidate onto the final combined parent; finish rain/UI/gameplay
coverage and the coherent inland winter belt including Bemidji and Grand Rapids.
Boundary Waters, its Safari preserve and entrance, North Shore, and Grand Marais
stay green/unfrozen. The statewide map-quality goal is still active.
