# Park trees, fixtures and live pond masonry

This kit stages original full-volume faceted fixtures after inspecting the
ignored browser Crystalpack's bank-correct source drawings, map layouts,
collision quadrants, NPC/event positions and animation loader. The source
images used for inspection stay outside the repository.

## Mature park trees

The formal broadleaf trees use their own Rust model in `foliage/park_tree.rs`.
Their complete 4×4 source drawing establishes the planting plot and rooted
ground ownership. It does not constrain the mature crown to four cells wide.
The former nominal height of about five cells left the trees barely taller
than park visitors when viewed from the promenade.

The nominal height now varies from 10.5 to 14.5 cells before stable architecture
variation, with broader overlapping crowns and higher spreading forks. Roots
and low planting stay inside the original plot; branches over adjacent paths
clear visitor height. The crowns now spread farther, with shallower lower
lobes and a modest offset leader. Their curved limbs and secondary forks
follow the same positions as the foliage. A more finely fluted trunk and
continuous warm bark grain give the exposed structure depth. Each tree
retains one cutaway owner and its original ground, collision and event claims.
Both National Park states consume the same source-authenticated tree family.

Each bough has 1,200 smaller cupped leaves in four-leaf sprays. Each six-edge
leaf now joins its curved upper face to a shallow underside at the same rim.
These thin closed volumes retain outward shading and ordinary back-face
culling from both sides. Sprays overlap through the outer growth and interior,
with broken perimeter growth instead of an enclosing shell. Shaded leaves retain blue-green pigment
beneath olive highlights rather than receiving an almost black baked color
before scene lighting. The larger mature height range is preserved.

The earlier height correction was verified on 2026-10-21 with the ignored local
pack in nine headless native captures, each inspected beside its same-state 2D
capture. Views cover all four ground-level directions at the southern promenade
and northern pool, an upward
view, an elevated view, night and Bug Contest. All nine state receipts preserve
the game state. Local evidence is in `target/outdoor-park-grown-review/`.
Those captures precede the subsequent crown, branching and bark changes.
The earlier selected foliage, park, surface-mask and batching checks had 84 distinct
passes. A separate New Bark house comparison in
`authored_exterior_buildings_reveal_only_their_new_geometry` fails; its building
path does not call park foliage and was left untouched. The canopy coverage
regression now requires a dense center and broken leaf edges rather than
demanding pinholes through the center of the bough.

At the user's initial request, the subsequent tree changes used code and
geometry checks without render previews. The user later requested direct
render inspection; that review is recorded below. The unfinished background
strips in the earlier captures precede the ground continuation change.

The crown and branch revision's foliage, park fixture/source, surface-mask and
batching selection passed all 87 checks. The canopy coverage check samples three bough poses from
five directions, including horizontal and upward views; every direction keeps
a dense center and broken outer leaf edges. It applies the production material's
back-face culling, so hidden far-side leaves cannot fill a visible gap. Trunk and
branch normals, winding, closed ring seams, rooted bounds, mature height, path clearance and single-tree
cutaway ownership also pass. The build and test logs are
`target/outdoor-park-crowns-test-build.log` and
`target/outdoor-park-crowns-verified-tests.log`. No render captures were run for
these crown, branch, leaf or bark changes. The native location-test client also
compiles successfully (`target/outdoor-park-crowns-native-build.log`); it was
not launched for preview.

## Leaf undersides and closed stems

The park revision preserves the upper leaf positions, spray layout, full leaf
count, crown architecture and height range. Each leaf adds a back sheet and
six underside triangles. Both sheets meet at the same rim positions but keep
separate normals, retaining thin blade lighting instead of rounding the edge
into a puffy scale. Exposed leaves remain visible from below and behind with
ordinary back-face culling. No enclosing crown surface is added.

The trunk and curved boughs also close their floor and tip rings with real bark
faces. Cap vertices share the existing ring positions and have separate normals,
preserving the side lighting and fluted silhouette. The trunk remains rooted
at the original floor; bough tips close inside their existing foliage.

The focused ten-check selection passes. It verifies paired, oppositely directed
edges and positive enclosed volume for leaves, trunks and boughs, outward
winding, unit normals, and front-face intersections from eight directions over
four leaf orientations and two sizes. The existing crown check still requires
dense interiors and broken leaf edges from five directions. Its ray arithmetic
uses f64 against the actual f32 vertices because rays exactly on shared fan
edges can otherwise reject both adjoining triangles through rounding.

A fully loaded seven-bough tree also passes the production paged group
roundtrip. This exercises the real leaf and bark buffers, independently of the
map's deferred tree recipes. Its opaque mesh exceeds 65,535 vertices and retains
wide indices. Byte-for-byte recapture preserves all buffers, transforms and
reveal policies; two differently placed instances still share opaque/sorted
geometry and lazy collision data. The restored collision surface accepts a
ray through the trunk and rejects a ray outside the crown. Evidence is in
`target/outdoor-park-solid-foliage-focus-tests.log`. No renders were used.

The broader foliage, instancing, park/source, ground continuation, compact
buffer, batching, worker and streaming-bound selection passes 149 checks, with
one external-pack matrix check ignored. This run includes the loaded park
geometry transfer above; the earlier thirteen-map pack run is recorded below.
The source compilation and selected-test logs are
`target/outdoor-park-solid-foliage-test-build.log` and
`target/outdoor-park-solid-foliage-verified-tests.log`.
The native location-test client also compiles successfully; the log is
`target/outdoor-park-solid-foliage-native-build.log`.

## Direct angle review and border trees

After the user requested renders, fresh production native captures were
inspected at the southern promenade `(20,40)` from an elevated diagonal,
eye level and an upward view, plus eye level beside the northern pool
`(20,24)`. Each capture has a same-state 2D pair and a successful state receipt.
The initial images are in `target/outdoor-park-solid-review/`.

The low views exposed a bright flat tree-art band beyond the modeled trees,
mechanically repeated bare forks, and inflated leaf-edge shading. The supplied
pack confirms that native border block `$00` and interior tree block `$06`
contain the same sixteen tree tiles. The selector previously accepted only
`$06`. Both complete drawings now use the same real spreading tree family.
The guard still requires exact tile identities, phases, one coherent source
block and a complete four-by-four plot; clipped, stale and mixed drawings
cannot acquire ownership. Ground, authoritative collisions and map layout
stay on their existing paths. No occlusion wall or enclosing canopy is added.

The leaf rim now separates upper and lower shading normals while keeping its
closed geometry. The primary limbs start at more varied, higher forks, taper
more strongly and rise into their own foliage on a gentler curve. This reduces
the exposed horizontal branch racks without changing the mature height range,
crown spread, leaf count or palette.

The revision passed twelve focused checks and the broader 149-check selection
(one external-pack matrix check ignored), then compiled in the native client.
Logs are `target/outdoor-park-angle-fixes-focus-tests.log`,
`target/outdoor-park-angle-fixes-verified-tests.log` and
`target/outdoor-park-angle-fixes-native-build.log`.

Ten fresh 1280×960 production captures and their same-state 2D pairs were then
visually inspected in `target/outdoor-park-angle-review/`: the southern
promenade from all four eye-level compass directions, an upward southern view,
southern and northern elevated diagonals, northern pool eye level, Bug Contest,
and night. All ten receipts report `state_preserved=true`. The green border
art band is gone, leaf highlights no longer round into scales, and the bare
fork racks are reduced. Reverse close views still exposed the trunk's flat
end beneath its upper crown. The leader now continues into that crown with
progressive taper, keeping the existing overall crown height and footprint.
Lateral limbs also finish as fine shoots instead of quarter-width stubs, with
their closed bark caps retained.

The final native build succeeds and thirteen focused geometry, source,
instancing and loaded-tree transport checks pass. The same ten angles and
their 2D pairs were captured again and all twenty images were inspected in
`target/outdoor-park-shoots-review/`. All ten state receipts pass, and the
capture logs report no shader/device failures. The border remains fully
modeled, undersides remain visible, and limb tips are finer. Exposed joins and
the repeated formal trunks still read stiffly in the closest views; this is
progress rather than a claim that the park has met the reference. The logs are
`target/outdoor-park-shoots-native-build.log`,
`target/outdoor-park-shoots-test-build.log` and
`target/outdoor-park-shoots-focus-tests.log`.

The broader park is not visually accepted. This tree review still showed flat
pink entrance artwork, addressed in the entrance work below. Flowerbeds remain
flat and regimented, and the pool lacks the saved reference's masonry depth and
water treatment. These are visible remaining defects rather than passing-test
claims.

## Native park gatehouses

The low entrance review exposed the original roof and brick drawing lying on
the ground. The source has the same complete twelve-by-twelve drawing at both
gates: nine native blocks `$28`–`$2a`, `$2c`–`$2e`, and `$30`–`$32`. Its adjacent
four-by-four approach determines the doorway. The southern gate uses north
threshold `$0a`; the eastern gate uses west threshold `$0b` beside its final four
source rows. The first north-only implementation still left the eastern gate
flat, caught by the paired eye-level capture at `(31,18)`.

`mesh/park_gate.rs` now builds a closed masonry gatehouse, pitched ceramic roof,
real recessed doorway, bevelled jambs and staggered shallow bricks. The west
variant rotates the whole structure and places its opening at the actual side
approach. The roof and wall sample their current native pigments. No windows,
opposite doorway or extra platform are invented. Each model owns only the
144 building cells, retains the original support and registers one building
with its doorway datum and whole-object reveal policy. All source tiles and
phases, plus the complete sixteen-cell approach, must match before ownership
can transfer.

Both eight-cell thresholds reuse the existing continuous cloth finish. Their
original floor positions, normals, triangles and support remain intact. Live
palette changes on the gate's roof/wall or either mat now invalidate retained
baked paint; ordinary flat paving can still refresh its atlas in place.

The entrance, threshold, retained-paint and rigid-building selection passes
eighteen checks. The external-pack drawing check also passes for both entrances
in both park states. The full 8,640-cell region worker check passes separately
for National Park and Bug Contest, including exact prepared-output recapture,
bounded paged transfer, unchanged source/support and independently owned
building metadata. Those runs preceded a clearing of the local
`target/` directory; their temporary logs and earlier diagnostic captures were
removed with that cache. The source changes and saved art reference remain.

The fresh native client build succeeds; its log is
`target/outdoor-park-gates-final-native-build.log`. Ten new 1280×960 headless
production captures and their same-state 2D pairs were inspected in
`target/outdoor-park-gates-review/`. They cover both gate approaches at eye
level, the eastern approach looking down, three elevated gate diagonals, the
eastern gate at night and during Bug Contest, and elevated/upward southern
promenade views. All ten receipts report `state_preserved=true`; capture logs
report no shader/device failures. The original flat roof/wall artwork is gone
at both entrances, and the offset eastern doorway meets its original mat.

The gate forms now have real depth, but their close materials remain plain.
Low views still expose stiff repeated trunks/forks, dense rectangular grass
walls and flat flower heads. The broader park remains below the saved visual
reference; the gate repair is not visual acceptance of the surrounding park.

## Shared runtime park trees

Runtime park trees now use the existing bounded forest library. Three branch
architectures across four maturity classes provide twelve full-detail recipes;
each placement retains continuous height, width and yaw variation. The original
10.5–14.5-cell height envelope, broad crowns, 1,200 leaves per bough, fluted bark
and rooted planting remain. Each instance keeps its own reveal state while
sharing canonical geometry and collision data. Worker messages retain every
placement recipe through the same runtime stream used by ordinary groves.

The park uses a four-cell prototype plot, preserving its own proportions rather
than inheriting the ordinary two-cell tree fit. Root outlines keep the stems
centered in their formal planting plots. The wider conservative bounds account
for real spreading crowns at arbitrary yaw; they add no geometry. Source guards,
ground claims and authoritative footing retain their original ownership.

The former direct expansion duplicated every park tree's opaque and sorted
render meshes and exceeded the existing worker buffer cap on the complete map.
The full 8,640-cell National Park now passes the production paged roundtrip at
281,535,718 raw bytes with a 38,195-byte header, without increasing protocol
limits. Every header and data page matches the independently captured CPU
output before and after restoration. The geometry checks cover all twelve
recipes and additional placements, verify full triangle counts and mature
height and broad crowns, require visitor clearance over adjacent paths, and
keep one reveal owner per tree. Runtime culling bounds include the new family.
The shared-runtime selection passed 140 checks. The thirteen-map external-pack
roundtrip also passes, including both complete 8,640-cell park states through
the production page transport. Bug Contest has the same 281,535,718-byte raw
payload, a 38,205-byte header and 1,805 pages. Native compilation succeeds.
Evidence is in `target/outdoor-park-shared-verified-tests.log`,
`target/outdoor-park-shared-external-worker-test.log` and
`target/outdoor-park-shared-native-build.log`. No render preview was used.

## Ground continuation beyond the park

The existing flat presentation apron now accepts authenticated park paving in
the same finished material domain as lawn and paths. Previously its material
filter excluded paving, leaving a repeated native tile image beyond finished
plazas. Both terrain build paths now use the existing ground detail coordinates
on this plane. The four runtime strips retain exactly their positions, normals
and triangles, at the same height below the real floor. No new scenery,
terrain height, source ownership or gameplay support is added.

The compact apron also retains its actual donor cell. Artwork alone cannot
identify a donor map when connected maps share the same tiles. Ground color
and detail coordinates now use that cell's source map and origin. The expanded
build uses its sampled donor for the same decision. This avoids borrowing an
arbitrary frame edge's style or the last finished surface's coordinate origin.

The three focused geometry/source checks cover both park states, exact apron
and finite-surface preservation, unknown-art retention, the outside-plane
guard, and identical-art donors across a Route29/New Bark handoff in both
build paths. These are code checks; no render preview was used.

An earlier broad source, foliage, park, background, batching and worker selection had
245 passes, four failures and three ignored checks. The failures are
`johto_signs_follow_live_paint_without_moving_posts_or_footing`,
`kanto_directed_bank_replaces_owned_detail_and_grains_caps_without_support_changes`,
`runtime_path_uses_authored_geometry_with_live_source_roof_paint`, and
`tree_variants_preserve_horizontal_plot_and_ground_anchor`. The sign test calls
its prop builder directly; the other fixtures are smaller than the apron
threshold and have no outside plane at the apron height. These failures remain
outside the current continuation change. Full-suite acceptance is not claimed.
The native location-test client compiles successfully. Logs are
`target/outdoor-park-background-test-build.log`,
`target/outdoor-park-background-verified-tests.log` and
`target/outdoor-park-background-native-build.log`.

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
The pedestal fountain has two low masonry steps, a shaped stem and a shallow
carved bowl with a rolled limestone lip. Its separate water surface sits below
the rim. A small bronze nozzle feeds a central plume and eight fine recirculating
arcs that land inside the bowl. Rear, inner and underside geometry is explicit.
Whole 4×4 metatiles guard each drawing, including the unowned backing. Only the
4 or 6 drawing cells receive ownership. Their native same-block plaza pattern
is composed underneath; it remains at zero height and earns no extra count.

## Joined live pond fountain

In NationalPark and NationalParkBugContest, `3f` immediately adjoins `33`.
Together the 8×4 guard contains 4 static center cells (`80/81/90/91`),
8 live spray cells (`5f`) and 20 water cells (`14`). Each complete fixture owns
its 4 center and 8 spray cells. The 20 ordinary water cells remain unclaimed.
The source checker verifies `[8, 6, 12]` bin/pedestal/pond ownership in each park
state, with no completed pond fixture invented in SafariZoneBeta.

The center remains `(4, 2.15)` source cells and the basin's maximum radii remain
`(0.88, 0.65)` cells. Its submerged foot and curved belly support a thin rolled
lip at `WATER_HEIGHT + 6 = 4` source pixels. The closed stone section includes
the inner wall and floor; its separate water plane stays at 3 pixels. The
surrounding pond remains at −2 pixels, with the original stepped outline and
all footing unchanged. Limestone grain uses fixture-local coordinates.

The central jet rises above the bowl; eight outward arcs break into individual
droplets and land at the surrounding pond's actual water level. All streams
have round geometry visible from low and reverse angles. Their highlights flow
along each stream. The water finish uses restrained blue-green depth, crossing
wavelets and concentric ripples anchored to the complete source fixture.

The production animation loader is unchanged. Native water `14` retains its
four-frame, 22-tick animation and supplies the retained water motion clock.
Only an owned complete fountain authorizes the local pool finish; clipped,
customized and beta fragments retain their native surfaces. No animated source
pixels, animation frames or scripts enter an art document.

## The beta-map topology exception

The old checklist's 48 fountain cells meant one complete 16-cell `3f` metatile
on each of three maps. It did not identify three complete two-sided fountains.
SafariZoneBeta places its `3f` at block `(5,11)` immediately beside grass block
`13`, not `33`. Its two `33` fragments are at `(4,13)` and `(5,13)` farther south.
There is no complete joined fountain at any of those positions. All beta source
fragments remain native, including their real water and spray. No missing half
is invented over grass. The old source-loose `3f` shell path is disabled in the
production authored mesher; it remains only in the samples-only diagnostic path.

This package therefore replaces 44 small-prop cells plus 24 basin/spray cells.
The remaining beta topology is an explicit fidelity exception requiring
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
backing, repeat append, outward face winding, recessed water levels, jet landing
height, and unchanged pool geometry and ripple coordinates while scrolling.

Current fountains are assembled in Rust by `mesh/park_fountain_basin.rs` and
`mesh/park_fountain_spray.rs`. Their shape and water are not read from the older
stored fountain mesh documents. The bin still uses its stored model.

Generate the stored models' editable source with `tools/build-park-scenery.py` in Blender.
The source manifest/chunks use the existing gzip-Blender contract, while mesh
documents use the existing lossless compact model storage. Workshop front/rear
renders review original volume; they deliberately leave the pond center empty
because live source art belongs to the runtime. These previews do not substitute
for final native camera, collision/warp and NPC interaction review after integration.

## Fountain review, 2026-10-21

Built the production `new_bark_3d` example with `location-tester` and inspected
ten headless 1280×960 captures against their frozen, matching 2D views. Evidence
is local and ignored in `target/outdoor-fountain-final-review/`: the northern
pool at a diagonal, low, side, reverse, close, night and Bug Contest view, plus
the southern fixture at a diagonal and two close eye-level angles. Every paired
state report has `state_preserved=true`. The low views exposed the former
straight-sided tub and barely visible spray. The first replacement water pass
had excessive regular rings; those were shortened, warped and broken into
quieter arcs before the final review. Close-up droplets were rounded as well.

The original grass implementation was restored and left unchanged during this
fountain pass. Original pool shape, floor geometry, gameplay ownership and the
park's two grass types are preserved.

Validation: 51 focused park tests passed, with one unrelated external gate test
ignored. Both full 8,640-cell park states passed the external-pack terrain worker
roundtrip, including identical prepared geometry and metadata. The standalone
source checker passed NationalPark, NationalParkBugContest and SafariZoneBeta.

Changed fountain modules are formatted and introduce no Clippy warnings.
`cargo fmt --all -- --check` still reports pre-existing formatting throughout
the workspace. Required full Clippy fails on the existing `bad_bit_mask` in
`crystal-core/src/multiplayer/mod.rs` and approximate constants in
`scenery_finish.rs`, `foliage/pine.rs` and `mesh/shore_ecology.rs`. An additional
local `--no-deps` inspection completed with the existing `approx_constant` and
test-only `erasing_op` denials allowed; it is not a clean full-workspace gate.
Build, test and lint logs use the ignored `target/outdoor-fountain-` prefix.
