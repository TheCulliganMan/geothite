# Tangela sculpture

The installed original recipe is `tools/tangela_sculpt.py`. It replaces the
previous dark upright body, repeated bent front rods and bead pupils with
a rounded blue tangle, fitted dark eye opening, white oval eyes with tiny
high pupils, and broad grounded red shoes. This is static original geometry.
No external model, texture, skeleton or clip was imported. The
[source review](vine-parent-source-review.md) records the official artwork,
individual third-party metadata and actual inaccessible download endpoints.

## Geometry and integration

The model has 49 parts and 106,668 triangles, floor 0 and height 1.00.
Canonical SHA-256:
`d2112c7af307009754020e3500bb545174b0a86d62401f56a0e84194e573fa1e`.

There are 41 physically rooted vines: layered front spirals, four ropes
framing the eye opening, nine differently tilted wrapping paths, four rear
curls and four curls angled toward the sides. Each wrapping centerline is
projected onto the core ellipsoid, including its compressed top, before its
short buried-root transition and outward surface offset. Rope widths follow
local bend radii, with independent nonadjacent triangle-crossing checks.
The side curls rotate both positions and normals through 0.90 radians.
The long wrapping ropes use 18-sided sections, while the other ropes retain
24-sided sections, within the unchanged 110,000-triangle limit.

`tools/build-battle-species.py` dispatches directly to this pure recipe before
Blender batching. Its old generic Tangela body is replaced by an explicit
guard. The exact Rust Tangela species route remains unchanged.

## Verification

All nine candidate checks passed in 19.168 seconds. All nine checks against
the installed canonical bytes passed in 26.181 seconds. The checks cover:

- Closed outward surfaces and normals aligned with actual faces.
- Recipe parity, floor, height, palette, unique parts and triangle budget.
- All 41 vine roots, both shoe roots and both eye roots against actual body
  volume, with detached negative examples.
- Fitted face and pupil vertices, triangle centers and edge clearances
  against their actual stored parent surfaces.
- Nonadjacent triangle crossings within every individual vine.
- Every stored pupil vertex against actual occluding vine triangles.

The final front, oblique and rear software renders each exited 0 at
800×824 and were individually inspected in full. The round silhouette,
clear pupils, broad shoes and layered curls are substantial improvements.
Smooth core remains visible between strands in the oblique view; these
renders do not establish that every angle is fully polished.

Earlier candidates exposed real defects: self-crossed tight curls, abrupt
buried-root joins, rear loops hidden beneath parallel bands, a handle-like
crest, and tilted ropes sinking into the core. The current recipe uses
spaced spirals, bounded widths, a smaller crest, projected wrapping paths
and earlier root emergence. Those failures were corrected in geometry;
the root, crossing, pupil and budget guards were retained.

The native union build exited 0 in 4 minutes 49 seconds. Its binary contained
the full final canonical SHA and the preview-mute switch before launch.
Frozen binary identity was `(218972741, 215590880, 1791824407445108302)`
(inode, bytes, modification time in nanoseconds). With explicit
`CRYSTAL_PREVIEW_MUTE=1` and `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
the real external `content-packs/realtime-clock.browser.crystalpack` produced
a 2000×1400 native GPU battle capture and exited 0. The full image was
individually inspected: Tangela's vines, clear gaze and red shoes rendered
with the opponent, arena and HUD. The binary identity and canonical source
hash remained stable across capture. This proves that native image-target
render, rather than ordinary live-window, browser, animation or gameplay
coverage.

The final-hash Rust unit binary contains the canonical Tangela SHA and was
frozen at `(219161165, 245836800, 1791824972427262607)`. Its test listing
identified the exact species regression, which passed as one actual test
in 5.99 seconds. That regression asserts 251 supported species and checks
each authoritative route, loaded mesh, indices, finite geometry, floor,
bounds and unit normals. Binary identity and Tangela source hash were
stable across the run. This is integration coverage, not approval of all
251 species' art. The broader Cargo union invocation exited 0 after
14 minutes 41 seconds of compilation. It ran one actual species regression,
passing in 18.42 seconds; the other crates had zero matching tests and are
not counted as additional species coverage.

Scratch output is ignored and retained while the Tangela/Kangaskhan family
work continues.
