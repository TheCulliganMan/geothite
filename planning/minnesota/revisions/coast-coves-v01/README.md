# Warm Superior cape refinement

This is a bounded native 2D terrain revision of the approved compact R10 pack.
Three small pine headlands interrupt the long straight coast, with a sheltered
cove above the middle cape. Each new overlook is reachable by a short grass
saddle from the existing road and displays actual Lake Superior water.
The sea remains the dominant continuous body; no creek or waterfall is enlarged.

The sparse `delta.json` is an authored terrain patch, not a game catalog or
exported program dump. Its frozen 179 native-cell preimages prevent replay onto
an incompatible coastline. The before/candidate layouts retain the existing
source plan and explicit native point strokes; clipped terminal padding stays
blocking. No new art, scripts, actors, Pokemon, rooms, warps or flags are added.

Native player anchors (global gameplay tiles):

- Split Rock finger: start (40,66), via (58,66), view (58,68).
- Sheltered middle cape: start (34,136), view (60,136).
- Southern pine cape: start (48,178), view (64,178); crosses a real chunk seam.

The first review spur crossed the existing sign block at metatile (18,67).
The revised middle spur is one metatile south and preserves the sign and its
approach. Existing gates, pickups, cabin doors and passive returns stay exact.

## Publication

```sh
CARGO_TARGET_DIR=target/coast-shape cargo build --locked --profile wasm-dev \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  -p crystal-assets --bin pack_north_shore_coves
target/coast-shape/wasm-dev/pack_north_shore_coves \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  planning/minnesota/revisions/coast-coves-v01/delta.json \
  content-packs/minnesota-coastal-coves-v01.crystalpack
.local/minnesota-review-env/bin/python \
  planning/minnesota/revisions/coast-coves-v01/check_source.py
```

The publisher validates compact warm geography before and after, rejects frozen
cell/preimage/event-anchor drift, retains all exterior boundary cells and all
connections, and proves exact equality to the parent after normalizing only its
changed block arrays, encoded mirrors and derived identity. Compatible disjoint
parent revisions can be retained during final integration.

The generic `revise_authored_route` was attempted first, including an identity
before-to-before operation on the approved parent. It rejects the stale
`SOUTHERN_SHORE_HABITAT` dossier rectangle (32,126)-(38,132), which is not grass
in the actual installed parent. Its shared validation was not weakened; this
bounded publisher avoids transplanting the stale dossier/events.

## Review and verification

Whole-map native source pixels, weakest approach and all three player views are
retained under `planning/minnesota/renders/coast-coves-v01/`. Production images
are explicitly named `production-*`; source crops alone are not GPU acceptance.
`author.py` only writes planning source and read-only source-art renders. It
never writes a pack; future render runs use unique directories.

`check_source.py` proves actual native walking access, required grass cost on
all three spurs, water within each player camera, zero ICE, unchanged route
perimeter and one Superior water component through the real north/south seams.

The new ignored native regression is
`north_shore_coves_controller_walks_three_grass_capes_and_continues_at_water_views`.
It uses explicit legal Floor/Repel fixtures, then production controller inputs,
Save/Continue at each viewpoint and walking returns while preserving party,
bag and story flags. Fixture setup is not earned campaign progression.


Verified publication: `content-packs/minnesota-coastal-coves-v01.crystalpack`,
SHA256 `08b99db5cd4ffe7599368de8d1c2f5e400a8890876e5f630696443690f613b67`.
All three cape native production 2D views were captured and visually inspected.
Four legacy warm-controller cases pass (grass/Continue, stairs/ledge, Surf buoy,
Waterfall gates/return), and the compact north-gate badge/signal case passes.
The legacy all52-former-ICE fixture fails its old terrain-course assertion on
both this output and the untouched approved R10 parent; it is baseline fixture
drift, not a newly introduced gameplay failure. Current compact warm geography
and zero ICE are separately checked over every installed R11 block/callback.

Required all-targets Clippy is blocked by the pre-existing denied
`clippy::bad_bit_mask` in `crystal-core/src/multiplayer/mod.rs:1332`.
Workspace `cargo fmt --all -- --check` also reports extensive unrelated existing
formatting changes. Changed coast files are individually rustfmt-clean.

The focused assets regression passes against the actual approved external pack:
positive publication/source equality, stale-terrain rejection, an existing
trainer moved onto a compatible Floor delta cell rejected by the event-anchor
guard, and outer-perimeter delta tampering rejected by the frozen contract.
The initial test-only relative path was corrected to a canonical path required
by the strict pack loader. No production validation was relaxed.

The scoped cape traversal regression declares the R11 trainers previously
defeated in its initial fixture. Reed (`MN_R11_T05`, ordinary
`BATTLETYPE_TRAINER`, `EVENT_MN_R11_TRAINER_05`) stands at (43,65) facing south
with sight one, intersecting the first cape course at (43,66). Trainer scripts,
teams and sight remain native and unchanged; this fixture represents a return
visit and does not prove earned trainer victories. Starting flags are recorded
after setup and preserved through all walks and Save/Continue checks.
