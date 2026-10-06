# Park fixtures and live pond masonry

This kit stages original full-volume faceted fixtures after inspecting the
ignored browser Crystalpack's bank-correct source drawings, map layouts,
collision quadrants, NPC/event positions and animation loader. The source
images used for inspection stay outside the repository.

## The two small-prop identities

The 44-cell selector is nine complete objects, not 44 round props:

| Map | Litter bins, block 0f | Pedestal fountains, block 2f | Owned small-prop cells |
| --- | ---: | ---: | ---: |
| NationalPark | 2 × 4 | 1 × 6 | 14 |
| NationalParkBugContest | 2 × 4 | 1 × 6 | 14 |
| SafariZoneBeta | 1 × 4 | 2 × 6 | 16 |
| Total | 5 bins / 20 cells | 4 fountains / 24 cells | 44 |

Block `0f` has an open-top cylindrical litter-bin drawing at its upper-left
2×2 quadrant. Block `2f` contains a broad, tiered stone pedestal fountain in
columns 1–3 of its upper two rows. The existing map generator independently
names the `2f` drawing its canonical Park fountain source. This static fountain
is distinct from the animated spray in the large northern pond. No text or
interaction establishes that its water is potable; the asset is called a
pedestal fountain rather than assigning a drinking action.

The bin has a real open mouth, thick rolled rim, inner wall and recessed bottom.
The pedestal fountain has a tiered masonry base, separate stone bowl, glazed
well and central spout. Rear and underside geometry is authored explicitly.
Whole 4×4 metatiles guard each drawing, including the unowned backing. Only the
4 or 6 drawing cells receive ownership. Their native same-block plaza pattern
is composed underneath; it remains at zero height and earns no extra count.

## Joined live pond fountain

In NationalPark and NationalParkBugContest, `3f` immediately adjoins `33`.
Together the 8×4 guard contains 4 static center cells (`80/81/90/91`),
8 live spray cells (`5f`) and 20 water cells (`14`). Each map therefore adds
4 modeled basin cells, for 8 total; the 56 water/spray cells across both joined
plots remain live source surfaces. The basin is an actual closed hollow rim,
with submerged foot, outer wall, inward face, pale coping and underside.

The old grouped cap remains on the exact same live scene-atlas coordinates,
center `(4, 2.15)` source cells, radii `(0.74, 0.50)` cells and height
`WATER_HEIGHT + 5 = 3` source pixels. The masonry's broader outer radii are
`(0.88, 0.65)` cells and its top is `WATER_HEIGHT + 6 = 4` pixels. It remains
inside the complete 2×2 static source center. Backing water remains at −2 pixels.
The new hollow rim never fills the live center with a generated water picture.

The production animation loader is unchanged: spray tile `5f` cycles
`0,1,2,3,2,3,4,0` every 11 ticks; water `14` keeps its four frames every 22 ticks.
No animated source pixels, animation frames or scripts enter an art document.
The original source cap is retained as a meaningful native surface. Decorative
water in the separate static pedestal fixture is an original modeled material.

## The beta-map topology exception

The old checklist's 48 fountain cells meant one complete 16-cell `3f` metatile
on each of three maps. It did not identify three complete two-sided fountains.
SafariZoneBeta places its `3f` at block `(5,11)` immediately beside grass block
`13`, not `33`. Its two `33` fragments are at `(4,13)` and `(5,13)` farther south.
There is no complete joined fountain at any of those positions. All beta source
fragments remain native, including their real water and spray. No missing half
is invented over grass. The old source-loose `3f` shell path is disabled in the
production authored mesher; it remains only in the samples-only diagnostic path.

This package therefore replaces 44 small-prop cells plus 8 basin cells, not
44+48. The remaining beta topology is an explicit fidelity exception requiring
a source-map authoring decision, not an unfinished generic 3D asset.

## Guards, semantics and verification

Matching requires the three named maps, exact metatile art, 4-cell world phase,
correct tileset, complete backing and a complete rectangle. Rephased, cropped,
changed or reserved guard cells reject the whole object. Raw custom-profile
source ownership wins even when the custom object is incomplete or lacks its
required ground sample. Append revalidates source and all guard ownership before
writing geometry. No collision, actor, event, script, warp or source map is edited.
The three inspected maps contain no script block changes. Bug Contest NPC state
continues to use its separate existing map and scripts.

Run `tools/check-park-scenery-source.py --pack <external-pack> --rustc <rustc>`
for native counts, per-guard-cell mutations, map/atlas identity, phase, source
backing, custom ownership, clipping and scrolling checks without Cargo. Run
`tools/check-park-scenery-models.py` for compact storage, manifold geometry,
normals, bounds, front/back/underside faces and open-mouth/live-aperture tests.
The focused Rust renderer tests cover ownership, custom overrides, stale
backing, repeat append and the exact live cap/water heights.

Generate editable source with `tools/build-park-scenery.py` in Blender.
The source manifest/chunks use the existing gzip-Blender contract, while mesh
documents use the existing lossless compact model storage. Workshop front/rear
renders review original volume; they deliberately leave the pond center empty
because live source art belongs to the runtime. These previews do not substitute
for final native camera, collision/warp and NPC interaction review after integration.
