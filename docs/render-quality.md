# 3D rendering quality

The renderer uses the existing source palettes and shared assets. Authored
material roles now select roughness and reflectance for ceramic, plaster,
clay, mortar, timber, glass, stone, ice and water. The surface shader adds
bounded specular highlights and samples Bevy's environment lighting without
applying another full diffuse sun response over painted face shading.

The authored red Gyarados at Lake of Rage uses the pack's forced-shiny palette
with the existing shared species model. Both its dedicated sprite and the
pack's `icon_gyarados` fallback retain that appearance. The object identifier,
script, event, species and red source-palette guards remain required; ordinary
Gyarados and replacement sprites keep their own appearance. No second model or
species-palette catalog is introduced. Review Lake of Rage `(20, 27)`, orbit
`1`, zoom `0`, with `DU` at noon and 22:00 against matching native 2D.

Two small procedural cubemaps follow the visible sky's day/night palette.
The specular map has six roughness levels. Their asset handles remain stable;
the renderer refreshes their pixels at five-minute clock boundaries. Fixed
interiors and dark maps do not receive outdoor environment light. These are
sky reflections, not reflections of nearby buildings or actors.

The outdoor sky supplies 600 cd/m² scaled by those sky colors so lit Pokémon
retain readable back surfaces beside the direct sun. The painted-surface
shader removes this physical intensity from its normalized diffuse/reflection
samples. The [Pokémon lighting review](art/pokemon-lighting-fidelity.md) records
matched native comparisons, night/water controls and coverage limits.

Regional outdoor water uses four crossing wave slopes for reflection normals,
blue depth absorption, pale cyan shallows, broken shoreline wash and subtle
shallow-water caustics. A retained distance field follows the actual banks and
islands; streamed grid edges do not create foam. The depth is an optical cue
from distance to shore, not simulated underwater terrain. Coastal water has
longer, stronger waves than sheltered inland ponds. Vermilion's port uses the
same finish. Cerulean Gym uses finer, calmer ripples and shallow caustics without
coastal foam or the outdoor cloud sheen. Its original blue pool pigment, raised
walkways and water height are preserved. Forest pools, caves, park fountains
and other indoor pools retain their existing surface treatment.

Outdoor wave layers use independently seeded, continuous random fields to
warp their phase and vary their envelopes. This breaks up regular rows of
ripples without tile boundaries or frame-to-frame random flicker. Analytic
gradients include both variations, and filtering follows each wave's locally
warped frequency. Randomness is anchored in map coordinates, so camera motion
and streamed grid changes do not move the pattern. The approach follows
[GPU Gems' constrained wave variation](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-1-effective-water-simulation-physical-models)
and the use of different ripple scales in
[Unity's water samples](https://docs.unity3d.com/Packages/com.unity.shadergraph@17.4/manual/Shader-Graph-Sample-Production-Ready-Water.html).

Regional outdoor water also uses a damped Rust heightfield inspired by
[Evan Wallace's WebGL Water](https://madebyevan.com/webgl-water/). Surf restores
the old displaced volume and depresses the new foot position; the resulting
waves propagate, reflect at banks and decay. Small source-clocked disturbances
seed ambient ripples. A stationary swimmer adds no further disturbance. The
field retains heights and velocities in world coordinates across grid changes,
and dry or unavailable cells block propagation. Work is bounded to 512×512
samples at four samples per source cell; larger grids keep procedural waves.

The existing shoreline image retains its exact RG distances and stores current
and previous optical heights in BA. The shader interpolates those heights,
derives reflection normals and approximates shallow light focusing from the
same surface curvature, using
[Wallace's refracted-area principle](https://medium.com/@evanwallace/rendering-realtime-caustics-in-webgl-2a99a29a0b2c).
Snell refraction with water IOR 1.333 adjusts the absorption path with viewing
angle. Focused light follows direct illumination and shadows; fine disturbances
fade at distant zoom. Basin depth remains an optical estimate and reflections
use the existing sky/environment. Source water animation owns the simulation
clock, including pauses. Indoor and cave water keep their existing treatment.

Wave motion advances only with authenticated source water animation and settles
when that animation pauses. Reflections respond to the camera, wave normal and
scene lighting, with procedural cloud sheen plus Bevy environment radiance.
Fine waves fade below pixel size at distant zoom. This adds no render pass,
reflection camera, texture asset or water mesh displacement, preserving the
authored banks, Surf support and gameplay geometry.

Painted terrain detail, authored scenery textures and human character paintings
have complete mip chains, linear-light downsampling and premultiplied-alpha
filtering. Anisotropic sampling uses 16x on high, 8x on balanced and 4x on
low/legacy. Masked scenery preserves approximate alpha-test coverage at each mip.
Character paintings retain their original mip-zero pixels and shared upload
across body rigs; smaller filtered levels prevent distant facial and clothing
details from flickering while the camera or character moves.
Source tile/sprite atlases keep their original pixels and sampling. Terrain
detail uses explicit gradients computed before material branches or discards.

Shadow coverage follows camera distance, zoom and viewport scale in stable depth
bins, with Gaussian filtering and a smaller normal bias. Browser rendering uses
one cascade; native low/balanced/high use one/two/three respectively. This policy
applies across maps instead of selecting a few towns by name. Existing nighttime
and software-renderer shadow rules still apply.

## Presets

Select a native preset at launch with `CRYSTAL_RENDER_QUALITY`:

| Preset | Anti-aliasing | Contact shading | Dynamic range |
| --- | --- | --- | --- |
| `high` | SMAA Ultra | Native SSAO on supported hardware, plus authored contacts | HDR, neutral tonemapping, subtle highlight bloom |
| `balanced` | SMAA High | Authored contacts | Existing display-linear output |
| `low` | Existing FXAA policy | Authored contacts | Existing display-linear output |
| `legacy` | Existing FXAA policy | Existing opt-in SSAO | Previous material, texture and shadow settings for comparison |

Native hardware defaults to high. Recognized software adapters fall back to
low. Browsers default to balanced; Bevy 0.20 SSAO is unsupported on both WebGL
and WebGPU. `CRYSTAL_MODELED_OCCLUSION=off` disables native SSAO independently;
`on` requests it on supported native hardware in other presets.

Post-processing applies only to the world camera. While that HDR world is
active, the marked UI camera shares its floating-point main-buffer format so
Bevy composites both cameras into the same buffer. Its own tone curve and
effects remain unchanged. Switching to 2D restores the original UI buffer
format; battle actor capture targets keep their own camera settings.
Bloom extracts highlights above display white, including luminous
night windows, instead of blurring daylight paint. SMAA uses Bevy's bundled
lookup textures with the Rust zstd decoder. TAA is not enabled: animated grass,
skinning and camera apertures need a separate motion-vector and ghosting review.

For a native same-binary comparison, `CRYSTAL_ANTI_ALIASING=smaa-high` restores
the former edge filter and `smaa-ultra` selects the new high-preset filter;
`off` disables SMAA on balanced/high. Low/legacy and software fallback keep
their existing FXAA policy. `CRYSTAL_TEXTURE_ANISOTROPY=4` restores the former
authored-texture sampling (valid overrides: 1, 2, 4, 8, 16), and
`CRYSTAL_CHARACTER_MIPMAPS=off` restores the former human texture upload.
These controls apply at launch. Compare both camera modes while stationary
and walking; a softened image alone does not establish stable motion.

The October 17 native comparison uses one optimized executable and frozen
artwork at 1280x1024: Goldenrod `(15, 22)` in first person at noon and 22:00,
New Bark `(8, 8)` in orbit, Route29 `(50, 9)` with `LLRR` in first person,
and New Bark `(13, 6)` with `DU` in orbit. Matching 2D/3D receipts preserve
authoritative state; movement traces confirm translation in both walking cases.
The five texture-filtering tests, nine camera-quality tests and supplied-rig
painting-sharing regression pass. The latter checks shared GPU upload and exact
mip-zero painting pixels across different rigs. Edge improvements are modest;
subpixel foliage and existing surface overlaps remain visible. Concurrent
graphics activity prevents an isolated performance conclusion from these runs.
The optimized native examples build and the Wasm `location-tester` check pass.
The prepared `just play-3d` native executable includes these changes; its
post-install `DU` route in New Bark and matched 2D capture also pass. Browser
source is checked, but this comparison does not publish a new browser bundle.

Balanced and high apply a miniature-style tilt-shift effect to the 3D orbit
camera. A completely sharp band covers at least 28% of the output height around
the trainer and expands at close zoom to protect the hat and shoes. Above and
below the band, blur increases with a power-of-2.5 falloff normalized separately
to each screen edge. It starts with zero slope and curvature and reaches full
strength only at the edge, preserving detail near the center and avoiding a
uniformly blurred outer strip. Raised roofs and ground at the same screen
height receive the same softness, avoiding the uneven focus of the earlier
physical lens effect.

Native Johto, Modern and Kanto town/route views widen that sharp band to at
least 40% and use 55% of the standard edge blur. This keeps nearby roofs,
windows and people readable while retaining a gentle miniature falloff.
The map and native tileset must both identify an exterior; gate rooms, caves,
unknown drawings and forest views retain their existing focus treatment.
Review Mahogany Town `(10, 10)` and Goldenrod City `(15, 22)` against matching
2D by day and night, including a short walk at each position.

Shared Johto and Modern building and sign underlays prefer a level native
ground sample from their own tileset before considering the same paint on a
remote bank. This lets the ordinary earth/lawn finish cover the whole backing
instead of leaving elevated rectangles of enlarged source pixels beneath
models. Source claims, building geometry, authoritative footing and tree/root
sample selection remain unchanged. Other tileset families retain their prior
sample policy. Review Mahogany `(8, 9)` with `UD` and Blackthorn `(18, 22)` with
`RUUDDL`, orbit `1`, zoom `0`, by day and night against matching native 2D.
Mahogany `(8, 15)` checks the gym and its sign from the clear south approach.

Two separable Gaussian passes run on the world color buffer before tonemapping.
The maximum blur diameter is 1.8% of the output height (about 13 pixels at 720p),
bounded to 3–36 pixels. The Gaussian tail fades to zero at three sigma so tap
changes cannot create visible steps in the gradient. Nearly sharp pixels bypass
the kernel entirely. Focus follows the rendered camera, interpolated player
position, terrain height and
jumps. The uniform layout and color-only sampling work on native, WebGPU and
WebGL; no depth texture or extra bokeh target is needed.

First-person view, 2D, low/legacy presets and software adapters retain their
sharp output. Set
`CRYSTAL_TILT_SHIFT=off` at native launch to disable it independently for a
comparison. Battle actor capture cameras and the UI camera do not receive the
effect. The render integration follows Bevy's
[custom post-processing path](https://bevy.org/examples/shaders/custom-post-processing/)
and uses the world camera's actual HDR/LDR target format.

The SSAO normal prepass uses the visible surface shader's source cutouts,
coverage fades and cave apertures. It writes geometry normals for unlit painted
materials, which Bevy's standard unlit prepass intentionally omits. Shadow and
depth-only shader selection retain their existing behavior.

Modeled overworld Pokémon use the same shadow-readability policy as painted
people. Their bodies retain PBR contour lighting and cast ground shadows, while
cast-shadow reception is disabled so small followers such as Totodile do not
become silhouettes. Their canonical colors, animation and emission are unchanged.
Elm's lab also continues its existing clinical wall panels behind the complete
rear bookcase run; the freestanding bookcases do not acquire walls.

Shared Kanto brick buildings fold repeated source window courses into height.
Tall variants keep a compact upper volume over a lower rear wing, retaining the
complete occupied source footprint and native front facade. Roof decks, coping,
window recesses and masonry remain part of the same shared producer. The gray
roof field follows the native diagonal slate pattern, with closed beveled
pieces clipped to the inner coping. The deck and coping retain their original
heights and footprint; color and material registration stay local to the
building. Review Celadon City `(26, 18)` at noon and 22:00, including `UUDD`
along the paved passage, against matching 2D.

Brick firing variation uses irregular local course/brick identities instead
of a repeating five-brick diagonal. The bounded brightness range preserves
the live native hue, mortar, trim and window paint; scrolling cannot move the
variation. Geometry, openings and all support heights remain unchanged.
Review the mansion frontage at Celadon City `(16, 12)` and the department
store at `(4, 12)`, orbit `1`, zoom `0`, noon and 22:00, with `LR` against 2D.

The shared Burned Tower roof retains its original eaves, ridge and source
plot. Smaller closed slate fragments have staggered courses, uneven seating
and fractured rims around two burn-through openings, with charred rafters
visible above the existing dark interior.
The openings have no continuous roof backing; surviving fragments and
rafters each remain closed solids. Intact Ecruteak roofs use their existing
ceramic producer. Review Ecruteak City `(5, 6)`, orbit `1`, zoom `0`, noon
and 22:00 against matching 2D; keep the entrance and surrounding ledges fixed.

Burned Tower's facade uses the same complete live wall-art palette sampler
as Ecruteak's other timber buildings. Pale plaster, surviving cedar trim and
charred members follow the native clock colors. Closed chamfered members,
upright base boards and a recessed entrance surround add physical joinery
inside the existing footprint. The roof retains its separate charred finish;
the facade pass cannot repaint its slate, rafters or adjacent scenery.
The ruined drawing's fifth source row is still gray roof art; facade palette
sampling begins on its sixth row so those fragments cannot contaminate plaster.
Soot and the interior under the open roof use the darkest native ink with a
small timber contribution, keeping the bright ochre trim out of the burn holes.

Tall Kanto retail drawings use that same producer when the complete native
MART sign proves the right-hand base block. The source roof, masonry, windows
and story count remain authoritative; a small solid signboard retains the
retail identity. Complete adjacent native paving continues under the building.
Ordinary eight-row Centers and Marts keep their existing separate producers.
Review Celadon City `(4, 12)`, orbit `1`, zoom `0`, with `LR` at noon and
22:00 against matching 2D, then approach the source entrance from the south.

The modern department-store model has solid MART lettering in place of the
five blank plaque blocks. Its complete twelve-by-sixteen source drawing binds
live teal masonry, pale trim, window and roof pigment to the existing material
groups. Source identity and window ink are checked before paint is applied;
missing or altered art keeps the existing fallback. The adapter is shared by
drawing, with no map-name test. Stone footings, hardware, fitted openings,
height and roof geometry remain unchanged. Ordinary Centers and Marts do not
use this department-store adapter. Review Goldenrod `(24, 30)`, orbit `1`,
zoom `0`, at noon and 22:00 with `LR`, then `UUU` into the store, against
matching 2D. The named editable lettering remains in the existing exterior
exporter and canonical `modern_department.mesh.json` asset. Fourteen staggered
courses of the existing closed civic-brick primitive now sit over separate
recessed mortar on all four store walls. The same window cutter opens both
layers, so brick faces cannot fill recessed glazing. This wall detail fits the
existing twelve-thousand-triangle budget; roof, windows, lettering, hardware,
bounds and door anchor remain identical. Review the same frontage in daylight
and at night, including the repeated upper-window openings.

The same brick-building kit repeats one low covered roof vent across its
native slate deck. Raised covers, recessed dark wells and physical louvres
use the existing live roof/trim pigment. Narrow and wide roofs, stacked
stories and rear wings share the producer; there is no Power Plant-specific
façade or cloned asset. Covers project slightly above the unchanged coping;
the roof deck, windows, doors, plot and authoritative support retain their
original geometry. Review Route10North `(3, 11)` with `LR` and Fuchsia
`(18, 8)` with `DDUU`, orbit `1`, zoom `0`, at noon and 22:00. Verify the
Power Plant entrance with `UUU` from the same Route10North position.

Shared Kanto civic foundations continue complete adjacent native street art
under the model. The ordinary `$01` street drawing is verified across all
sixteen source cells before it can replace the lawn fallback, alongside the
existing courtyard/plaza donors. This removes the green foundation strip at
Viridian's Trainer House without changing the building, its support, or the
surrounding garden plots. Incomplete or altered street drawings retain the
existing fallback; residential, Center and Mart foundation policy is unchanged.
Review Viridian City `(22, 17)`, orbit `1`, zoom `1`, at noon and 22:00,
including `LR`, against matching 2D; `(22, 21)` gives the central street view.

The shared doorless Kanto brick building becomes a side gate only when both
complete `$17`/`$16` entrance drawings immediately flank its native facade.
That source-proven variant cuts the foundation, masonry backing and side bricks
around one open corridor, with stone jambs and lintels on both sides. Its two
native entrance lanes remain clear. Front-door offices and annexes without
those complete approaches retain their existing geometry; no town-specific
model is added. The same side mats share the port threshold's source-colored
woven finish, retaining every original floor triangle and support height.
Gate floors continue the verified plain street sample from those same side
approaches, preserving the surrounding road's material inside the passage.
The finish follows existing terrain contours by retaining each vertex height
and normal; it does not require or create a horizontal replacement floor.
Review Route 16 `(8, 6)`, orbit `-3`, zoom `1`, with `R` at noon, and
`(8, 7)` with the same camera at 22:00. The opposite approach is `(15, 6)`,
orbit `3`, with `L`. Normal `RR` from `(8, 6)` enters `Route16Gate (0, 4)`;
`LL` from `(15, 7)` enters the other side at `(9, 5)`.

Outdoor Johto, Modern and Kanto cutouts borrow ground only from their own
native atlas, preferring a level sample over a raised bank cap. Connected maps
can reuse the same tile number for different pixels; a foreign donor otherwise
leaves bright square patches beneath the modeled flowers. The initial build
and flower refresh use the same donor selection. Flower refreshes also use the
existing outdoor terrace field, keeping roots on the lawn after movement;
forest and interior bloom geometry retains its previous path. These changes
alter neither the terrain height field nor the canonical flower asset.
Review Goldenrod `(29, 8)` with `DU` and `(33, 3)` with `LR`, orbit `1`, zoom
`0`, noon and 22:00 against paired 2D. The latter catches blooms sinking below
the raised lawn after a refreshed frame; check both movement samples as well
as the final view. The external donor audit checks availability in 67 native
outdoor maps; connected captures supply the context omitted by full-map grids.

Shared outdoor Kanto flower clusters retain their existing mesh, fitted size,
leaves and roots, while the petal materials follow the live native red/pink
flower palette. Material ownership retains the model's facet shading; the
painted petals share the original sway in both visible and shadow passes.
Their source color is not tinted a second time at night. Missing or unknown
paint retains the authored cluster, and other flower families keep their
existing appearance. Review Pewter City `(8, 4)`, orbit `1`, zoom `1`, at noon
and 22:00, including a short `LR` walk, against matching 2D.
Viridian City `(22, 17)` with the same camera and a noon `LR` walk checks
the same shared producer beside the Trainer House.

Shared traditional-house roofs retain their closed tile geometry, with
readable side joints and quiet firing variation. The traditional gym uses ten
staggered rows of closed ceramic prisms per slope, fifteen tiles across its
wider roof, matching the smaller town kit's tile scale. Beveled shoulders and
thick lips replace the large unstaggered panels. Its structural deck, ridge,
roof bounds, facade, lettering design and doorway remain unchanged. The boxed
GYM nameboard projects past the fascia so its complete inscription remains
visible from the production elevated camera, within the original source plot.
The renderer
preserves these authored courses without applying the older roof subdivision.
Violet's gym now samples its native roof paint through the same live palette
path as other Johto gym placements, including the night palette, instead of
retaining fixed purple paint. Other roof families keep their prior shading.
Review Mahogany `(8, 15)` and Violet `(18, 20)`, orbit `1`, zoom `0`, at noon
and 22:00 against matching 2D, including `UD` walks. From Violet's same start,
`UUU` checks the gym entrance. The roof silhouette and scene layout stay intact.
Sprout Tower uses that same closed ceramic refinement on its shared roof kit.
Its roof colors follow the live native roof interior instead of the fixed
purple checkerboard, including source night colors. Only the roof receives
source ceramic shading; plaster, joinery, lamps, forecourt and standing support
retain their existing treatment. Review Violet `(23, 7)` at noon and 22:00
with `LR`, and `(23, 5)` for a closer frontal view, against matching 2D.

Kanto cottages continue a complete adjacent native brick path beneath their
occupied plot through the existing shared paving producer. This removes lawn
strips at paved entrances and keeps cutaways paved as well. Missing, partial,
raised or altered paving retains the existing ground fallback; cottage walls,
roof, windows, entry and standing height stay unchanged. Review Route 25
`(47, 8)`, orbit `1`, zoom `0`, at noon and 22:00 against matching 2D, plus
an approach to Bill's house from the south.

Shared cottage, Center and Mart chimney/vent caps use a separate weathered
stone material. They must not inherit the source roof's black outline pigment;
the structural roof deck still does. Cap geometry, building bounds, doors and
service signs remain unchanged. Review Blackthorn `(18, 22)` and Cherrygrove
`(24, 10)` at noon and 22:00 against the matching 2D state.

The shared Kanto civic/gym roof uses the museum's closed barrel-tile primitive.
Its three ceramic shoulders catch light across each course, including tapered
hip courses, rather than reading as flat blue plates. The original deck,
ridge, roof footprint, native palette and entrance remain intact. Review
Vermilion `(14, 17)` and `(12, 22)` at noon and 22:00 against 2D; the museum
continues to use exactly the same tile geometry through the shared helper.
Shared Kanto cottage roofs use that same closed ceramic profile, including
narrowing hip tiles. Three long courses replace the flat five-course panel grid
within the existing mesh budget. Roof decks, ridge caps, building footprints,
source colors and independent window courses retain their existing behavior.
Review Cerulean `(11, 17)` at noon and 22:00 and the taller Route 5 entrance
`(10, 14)` against matching 2D.

Authenticated Kanto city-brick paths carry quiet mineral grain in source-map
coordinates. The pale pavers, recessed joints, bevels and standing datum keep
their existing geometry and live palette. Grain fades with pixel footprint
and remains fixed when a streamed crop moves; source lighting avoids a second
night tint. Brick bridges, raised piers and decorative borders retain their
previous finish. Review Celadon `(26, 18)` at noon and 22:00 with `UUDD`, plus
Fuchsia's paved streets, against the same 2D state.

Verified cells of Kanto's native `79/7b` checker plazas share warm limestone
flags with staggered joints, worn light edges and fine map-anchored grain.
They reuse the existing paving producer, replacing the uniform gray slab grid.
Both native checker phases receive the same finish; the `79` prop quadrant,
unknown phases, ordinary brick roads and the original walking plane remain
intact. The finish follows source identity across maps. Review Fuchsia
`(18, 8)`, orbit `1`, zoom `0`, with `DDUU`, plus Lavender `(14, 7)` with `LR`,
at noon and 22:00 against matching 2D. Fuchsia's northern frontage is closed
in the native drawing and must not acquire an invented doorway.

Blue-roof Kanto cottages also recognize the complete `$38/$39` roof over the
`$10/$12` bright-window facade. This drawing uses the existing shared cottage
producer, including both native window courses and the right-hand entrance;
it must not fall back to a flat sprite. The `$10/$66` underground facade keeps
its distinct dark panes. Review Route 5 `(10, 14)` at noon and 22:00 against 2D,
including a short `LR` walk below the house. Terrain ledges remain unchanged.

Outdoor rock and boulder props use the live sprite's body pigment, excluding
transparent background and dark outline pixels. Their existing shared mesh
retains its shape and uses neutral stone grain with source-palette lighting.
All native palettes share one grain material; movement reuses the cached mesh,
and palette changes refresh its paint. Cave and gym stone finishes retain
their own source rules. Review Cianwood City `(10, 18)` at noon and 22:00
against matching 2D, including a short `UD` walk on land.

Closed town terrace lips use the existing shared mineral-grain registration
on both their sealing face and closed rim stones. This restores textured rock
and the normal mineral lighting response while retaining every vertex,
support height and outline. Review Blackthorn City `(18, 22)` by day and night;
`RUUDDL` enters the actual gap at column 19, crosses north to row 20, then
returns. Straight north from column 18 hits the authored ledge instead.

The outdoor canyon body reveal has hidden ground backing inside verified
native rock cells beside dry source ground. It uses the existing neighboring
floor finish, so opening the rock around the trainer exposes ground rather
than the pale horizon. Water cannot supply a backing floor, and these hidden
faces do not supply walking support or change the mineral silhouette. Inspect
the moving frames along Blackthorn's `RUUDDL` route by day and night; the
small reveal at the spur's foot must retain the surrounding ground pigment.

Shared Kanto cottages also retain both native window courses when the source
has two floors. The same producer preserves each drawing's door position,
window count and lit or dark panes. Raised frames surround recessed glass;
upper walls and eaves provide room for the second floor, and hip tile counts
taper toward the ridge to avoid compressed seams on both compact and tall
cottages. Lit panes use the shared clock-driven window treatment. Compact
cottages retain their existing shape.
Cottage panes and door lights use the shared normalized source-glass finish,
including dark panes. A subdued sky/ground reflection and recessed gradient
provide depth without changing their native pigment, geometry or source lamp
identity. Each pane remains independent: only authored lit windows and the
door light emit at night. Review Pallet `(6, 7)`, orbit `1`, by day and at
22:00 with `LR`, plus Cerulean's wider cottage from `(11, 17)` against 2D.

Shared Kanto paving uses the live source palette once. Slab tops, grout,
bevels, circular trim, clipped bricks, bridge faces and building underlays all
carry the source-paint material identity, avoiding an additional moon-blue tint
on already night-colored stone. Their geometry, joints and standing datum are
unchanged. Route 8's western Lavender approach at 22:00 provides a useful
comparison with the muted paving in the matching 2D view.
The complete `$5e` south-end path drawing uses the same paving and round caps
as its `$5d` north end; neither end should retain a striped flat sprite. The
native paths remain walkable, so these borders do not become upright fences.
Review Route 5 `(10, 14)` at noon and 22:00, including the two side-path ends.

Connected scenery includes two authored map connections from the active map.
This completes landmarks split across a neighbor's boundary: approaching Lavender
from Route 12 now includes the tower roof on Route10South and selects the same
brick tower as inside town. Direct neighbors retain priority over farther maps;
reciprocal links do not duplicate a placement. Each source keeps its own palette
and time of day, and edits to any contributing map invalidate retained scenery.

Native canyon rock opens a feathered, body-sized view when it stands in front
of the trainer. The surrounding ridge stays solid, and surfaces at or below
the trainer's resolved walking support remain opaque. This uses the existing
player reveal calculation without changing rock geometry, materials or footing.
At Lavender's western entrance, Route 8 `(38, 8)`, the foreground ridge now
leaves the trainer's feet visible. Walk `LLLLRRRR` to check that the opening
follows the trainer and disappears after clearing the ridge.

Native town and route orbit cameras also check higher foreground walking
surfaces and increase pitch when a bank would hide the trainer. The requested
yaw, orbit distance, and target remain fixed; flat ground and rear terrain do
not raise the view. First person, caves, and forest presentation keep their
existing policy. Camera support is refreshed in the same update that publishes
completed terrain, avoiding a stale initial view until the first movement.
Review Route 24 `(9, 15)` with `CRYSTAL_RENDER_TEST_CAMERA='0 2 30'` and walk
`UUUUDDDD`: the bridge should remain visible with opaque banks. The opt-in
`CRYSTAL_CAMERA_CLEARANCE_TRACE=1` reports the first resolved clearance.

Wind-rippled Kanto seas break their long swells into finite wave groups, with
reflection slopes derived from both the wave and its envelope. Sheltered water
and indoor pools retain their existing optics. Kanto's complete sea-rock
drawings contribute the actual shared model's seated base to the shoreline
mask, rather than four square dry atlas cells. Foam follows each rock and leaves
water in its corners. Model selection and texture registration use the same
world coordinates across streamed crops; altered drawings and land rocks retain
their source behavior. Review Cinnabar from the waterfront land at `(10, 13)`,
orbit `2`, zoom `0`, against 2D, including a short `LLRR` walk at night.

Modern Johto's quiet paths finish both the sparse speckles and the blank
native cells between them as one continuous ground material. Only the proven
blank phases of blocks `07/0b/0c/0d/0e/3e` qualify; all-blank block `00`, fence
and wall phases, other tilesets and unknown art retain their previous handling.
The existing rural earth and city paving producers keep their styling, source
footprint and footing. Review Route34's Day Care frontage at `(10, 18)`, orbit
`1`, zoom `0`, noon and 22:00, with `LR`; also review Route35's Johto gate apron at
`(9, 32)` as an unchanged control, with the same camera and a short `LR` walk.

The shared north-facing gate authenticates its Johto `08/09` roof, complete
`16/1e` facade and complete Johto `0c/0d` northern approach as one drawing.
The facade may be entirely Johto, as at Alph, or entirely modern across the
Route35/Goldenrod connection. Every source cell must agree; the northern
approach distinguishes this gate from the ordinary house using the same
building blocks. Only a full mixed-family signature bypasses the ordinary
single-tileset guard. Each half borrows its own family's flat ground,
preserving earth/stone finishing under the threshold. Both versions reuse
`world-exterior/east_gate`, including its rigid roof and two-lane north
aperture. No map-specific asset or gameplay changes are introduced. Ordinary
houses without the approach and incomplete drawings retain their producers.
Review Alph `(7, 7)`, orbit `1`, zoom `0`, with `UD` at noon and 22:00,
and press `UU` to verify its Route36 gate entrance. Inspect Alph’s open
passage at the same position with `CRYSTAL_RENDER_TEST_CAMERA='0 0 -5 1'`
and an `800x640` viewport. Also review Route35
`(9, 32)`, orbit `8`, zoom `0`, with `LR` at noon and 22:00, plus Goldenrod
`(19, 3)`, orbit `1`, zoom `0`, with `LR`. Check both northern entrance lanes
by pressing `DD` separately from Route35 `(9, 32)` and `(10, 32)`.
Inspect the open corridor from the same Route35 position with production
first-person camera settings `CRYSTAL_RENDER_TEST_CAMERA='0 4 -10 1'`;
the overhead camera looks down onto its floor and obscures the far opening.

The shared modern Gym keeps the native two paired front window banks and
raised dark `GYM` lettering on a pale fascia. Fine closed running-bond masonry,
slim corner piers and one closed mitered coping replace the generic badge
lumps, oversized columns and overlapping roof corners. The original plot,
height and centered threshold remain unchanged. All six complete native
`johto_modern` blocks and exact opaque pane ink must agree before applying live
wall, glass and lettering paint. Pale brick faces and warm recessed joints
retain the source drawing’s light wall balance; the existing shared modern
roof producer keeps its field and border colors. Brick cells alone supply
masonry swatches, excluding the windows in that same source course. Altered
or incomplete drawings retain native presentation. Other Gym families, Centers and Marts keep their existing
assets and material producers. Review Goldenrod `(24, 10)`, orbit `1`, zoom
`0`, with `LR` at noon and 22:00 against matching 2D; walk `UUUU` from that
position to verify the Gym entrance.

Goldenrod's shared Radio Tower binds its charcoal cladding, gray metal trim,
roof and pale glazing to the live native broadcast drawing. All six complete
source blocks and exact opaque wall/window ink must agree before applying
paint. All four upper floors have four paired window banks, recessed horizontal
louvres and fine vertical piers. Dark mineral cladding uses stationary local
grain; metal trim keeps the source gray. Missing or altered source coverage
retains the native drawing.

The left entrance and raised RADIO letters share one architectural bay. Its
authored threshold supplies the exported anchor, so fitting to the native door
keeps the building evenly proportioned. Closed mitered roof coping frames a
front-corner open steel mast with triangular horizontal rings, alternating
braces meeting the next triangular ring and paired dipoles. The four upper
window rows interpret native rows `1`, `3`, `5` and `7` (the highest native
row is partial). The taller wall body and shorter mast share the previous total
silhouette height. The mast steel and red warning lamp retain separate
material/light roles. Footprint, total height and native entrance location stay
fixed; one canonical `world_exteriors/radio_tower` asset serves the drawing.
Review the upper silhouette and entry from Goldenrod's street at `(11, 18)`,
noon and 22:00, first person with `CRYSTAL_RENDER_TEST_CAMERA='0 1 -25 1'`,
an `800x960` viewport and `LR`, against 2D at the same state. Walk `UUUU` from `(5, 18)` to verify the
unchanged entrance into `RadioTower1F`.

Battle Tower uses one shared tournament building with coral side piers,
lower wing roofs and a taller convex cyan curtain wall. Closed faceted glass
panes, narrow curved floor ribbons and recessed side windows give the central
volume depth. Mitered roof copings close the stepped silhouette; an angled
entrance canopy and two low round tournament seals replace the shop awning and
roof crest. The paired sliding entrance, podium, plot bounds, overall height
and door anchor retain their previous coordinates. The canonical
`world_exteriors/battle_tower` asset is generated through
`tools/build-world-exteriors.py --only=battle_tower`.

The southern Tower gate recognizes its complete eight-cell-wide drawing across
three metatile columns, including the two-cell source offset. It reuses the
shared east/west gate rotated into a north/south passage, with both native warp
channels left open. The surrounding tree columns and outer rails are outside
the building claim. Tower rail bands reuse the existing source-painted fence
asset; the complete north threshold uses the shared cloth finish. Plain blank
approach cells receive the same level forecourt paving only at their proven
source phases. Review the gate from `BattleTowerOutside (9, 26)` and walk south
from `(9, 20)` through its authored entrance against the matching 2D state.

The Tower mountain's exact rock drawings use the shared outdoor canyon material
and mountain relief producer, including its mixed rock/tree boundary blocks.
Only proven rock cells form the connected volume; the tree columns, gatehouse,
warp approaches, lawn and paving retain their own ownership. Mineral beds,
fractured faces and world-anchored grain replace the flat golden source sheet.
The source cliff datum is the same universal mountain height used elsewhere;
additional visible relief remains inside that permanent rock footprint.

Raised native bridge bricks retain explicit ownership of their fitted pier
heights, including beveled faces clipped exactly onto a land boundary. The
terrace pass must not classify those faces as adjoining land and lift them
again. Their live source paint uses the existing shader role; this ownership
marker changes neither pigment nor geometry. Review `Route27 (56, 6)`, orbit
`1`, zoom `0`, at noon and 22:00 against matching 2D, with `DRRRLLLU` across the
shore/bridge contact. Keep land contours, shoreline shapes and water levels
unchanged.

Battle Tower's plain `battle_tower_outside` ground tile `06` uses the shared
level flagstone finish with quiet gray mineral pigment. Native ochre brick tile
`2f`, lawns and water retain their own producers. Flagstones change neither the
walking datum nor terrain ownership. Review `BattleTowerOutside (9, 16)`, orbit
`1`, zoom `0`, at noon and 22:00 against matching 2D, including a short `LR` walk.
A north-facing first-person view at the same position shows the full tower:
`CRYSTAL_RENDER_TEST_CAMERA='0 0 -15 1'`, CLI orbit `0`, zoom `0`.

The existing shared harbor-concrete producer retains its live native paint
and recessed two-cell slabs. Its narrow expansion joints are now wide enough
to read at the production exterior camera. The concrete finish adds quiet,
world-anchored per-slab weathering alongside its existing fine grain. Walking
support, slab tops, water, quay barriers, stairs and ship geometry retain their
existing heights and producers. Review Olivine Port `(10, 8)` at noon and
22:00, orbit `1`, zoom `0`, with `DU`, against matching 2D; check the same
shared treatment at Vermilion Port.

The existing shared ferry asset has recessed passenger and bridge glazing
with closed window frames. The same window-pocket helper used by the building
kits cuts the cabin skin before glazing is installed. Low roof vents, folded
roof seams, exposed teak-deck joints and an open tapered funnel give the ship
surface detail without changing its bounds, waterline, deck footprint or dock
fit. The source remains `tools/build-special-environments.py`; only the
canonical `harbor_ferry.mesh.json` is replaced. Review Olivine Port `(7, 21)`
and Vermilion Port `(7, 15)`, noon and 22:00, orbit `8`, zoom `0`, with `UD`
against matching 2D. The actual boarding seam remains on its original warp.

Complete four-tile harbor posts share one source classifier between their
existing modeled producer and shoreline optics. Their water-backed plots no
longer seed rectangular dry islands or straight bank-foam strips offshore.
The quays still contribute their actual shoreline; incomplete drawings,
ship artwork and non-harbor pools retain their source baseline. This changes
only the retained optical distance image, with no geometry, collision or
support-height changes. Review Olivine Port `(10, 8)` and Vermilion Port
`(8, 8)`, noon and 22:00, orbit `1`, zoom `0`, with `DU` against matching 2D.

Water height is independent of land terraces throughout the shared renderer.
This is an app-wide rule, not a Cinnabar exception: no height transition may
cross standing water unless an authored waterfall connects the pool levels.
An island's ledge raises its land only; standing water, Surf support, and
water-rooted plants cannot inherit that rise or slope. Land-contour walls stop
at water rather than cutting through a connected pool. Existing authored
waterfalls retain their own transitions between source-proven pool levels.
Where a shoreline rock or floating course covers the level water underlay,
the raised land beside it receives the same lower support closure as an open
water edge. That closure uses the existing support-face finish below the land;
the upper rim and barrier meshes retain their shape. Two neighboring water
owners never receive a wall between them.
Review Route 18 `(5, 6)`, orbit `1`, zoom `0`, at noon and 22:00 with `LR`;
`LLLL` from the same start enters `Route17Route18Gate (9, 4)`. Cinnabar
`(10, 13)`, orbit `2`, zoom `0`, with `LLRR` checks the level offshore rows.

Shared harbor moorings bind their stone body, crown and metal fittings to the
live native barrier drawing. The model retains its existing closed geometry,
placement, height and water backing; blue-gray daytime paint and night paint
come from the current atlas rather than fixed beige materials. A matte mineral
finish softens the source's vivid blue into weathered blue-gray concrete;
ordinary source paint keeps its existing palette. Reversible source-paint
coordinates avoid a second time-of-day tint. Review Olivine Port
`(10, 8)` at noon and 22:00 against matching 2D, including a short dock walk.
The complete `$35` harbor stair drawing uses the existing shared stair-flight
model with its own native material samples. A closed backing seals the recessed
dock opening; the surrounding deck and water keep their original heights.
The ladder warp and authoritative footing remain unchanged. Inspect both
harbors' stairwell treads and rails at noon and 22:00 against 2D.
Johto port approaches also use the shared fence asset for the outer rail
columns in native blocks `$0c/$0d`. The exact upper/lower rail tiles and phases
select it; the central threshold and both authored warp channels remain clear.
Their live source palette, ordinary floor and walking support are retained.
The complete port threshold shares the entry-cloth weave, using the mean
linear pigment of its live source pixels. Its original floor triangles and
height remain exact; incomplete drawings, custom ownership or missing paint
retain the source rendering. Woven detail fades with pixel size, and native
night pigment avoids a second moon tint.
Where that complete threshold sits north of the gate, the same shared gate
asset turns toward it. Its authored clear-jamb span fits both native lanes;
the plot, source roof palette and floor support remain unchanged. Drawings
without this approach retain their original side-facing passage.
The roof deck, shingles and gable truss keep one rigid fit while the lower
jambs accommodate the native opening, preserving their closed overlap.
Both harbors share the port-family coastal water finish: live source colors
provide absorption and reflection, with short crossing ripples replacing the
flat blue surface. Indoor port-family pools retain their separate pool optics.
Water geometry, source animation and authoritative support remain unchanged.
Exposed Johto and Modern-family coasts use that same shared wind finish,
including Cianwood, Olivine, New Bark and Cherrygrove. Their live day/night
palettes drive the color and reflection. Inland Johto pools share the pigment
and crossing ripple pass with weaker normals, shore wash and sky reflection,
replacing their flat indexed-blue surface while retaining a calmer finish.
Forest and cave water retain their existing presentation. This
changes optical shading only, without displacing water or reshaping ledges.
Review Cianwood `(11, 20)`, orbit `-4`, with `UD`, and Olivine `(30, 29)`,
orbit `1`, with `LR`, at zoom `0`, noon and 22:00 against matching native 2D.
Inland reviews use Violet `(27, 14)` with `LR` and Lake of Rage `(20, 27)`
with `DU`, orbit `1`, zoom `0`, at noon and 22:00. Both walks stay on dry land.
The shared harbor concrete floor samples the native blank background instead
of enlarging its speckled pixels. Smooth slab faces replace triangular facet
shading; fine aggregate and gentle weathering use stable map coordinates.
Two source cells form one slab, with narrow joints recessed below the unchanged
walking datum. Source tiles `$05` and `$31` share this finish in both harbors;
water, stair drawings, indoor pools and other floor families remain separate.
Verified floating buoy drawings also belong to water: they cannot acquire
land terrace height or generate stone backing walls through the sea. This
uses the shared buoy drawing classifier, including its flat source variants.
Their exposed paint samples the live atlas with the shared source-paint
lighting policy, preserving the native night palette without a second blue
tint. The water underlay retains its ordinary water material.
The shared float has a rounded shoulder, a visible narrow gray waterline seam
and a shallow recessed cap, all painted from those same live source swatches.
Its footprint and upper/lower height limits remain unchanged; the replacement
does not add a second float asset or change water geometry or Surf support.
Review Cianwood `(11, 20)`, orbit `-4`, zoom `0`, with `UD` at noon and 22:00.
Review Olivine City's lighthouse waterfront at `(30, 29)` at noon and 22:00,
against matching 2D, including a short `LR` walk on land.
The shared lighthouse has a circular 32-sided shaft, plinth, balcony floor and
roof cap. Export and production fitting use a uniform horizontal scale, with
the cylinder centred on the native doorway inside the source plot. The entry
projects beyond the round body without distorting its cross-section. An
asymmetric door no longer stretches one side of the tower; the former shallow
three-cell depth no longer squeezes it into an oval. Total height, source
claims, walking support and the native entrance position remain unchanged.
Only the curved shaft exterior uses smooth radial normals; window reveals,
caps and architectural edges keep hard normals. The lowest front window bay
stays clear of the doorway lintel. The lantern
handrail is a continuous open ring of closed segments; the stone balcony stays
visible between its posts. A closed ribbed Fresnel optic, eight framed clear
bays, narrow reflection strips and hollow weather rings replace the opaque
drum that hid the beacon. Clear panes are represented by open space in the
existing opaque terrain material; the lens retains its warm night-light role.
Inspect Olivine `(22, 22)` with first-person camera
`CRYSTAL_RENDER_TEST_CAMERA='3 5.3 -45 1'` at noon and 22:00 against paired 2D.
For the complete silhouette use camera `'0 5 -25 1'` and an `800x960` viewport.
Check the ordinary approach at `(29, 29)`, orbit `1`, zoom `0`, with `LR`.
The shared smashable `actor_props/rock` is one closed fractured stone rather
than four rounded stacked lumps. Broad cleavage planes, an asymmetric shoulder
and chipped crown stay within the existing floor-rooted envelope
`0.443640 × 0.430000 × 0.357527`; 98 triangles replace 80. Outdoor paint keeps
the live sprite hue with a restrained neutral mineral mix and the existing
neutral bedrock grain. The larger Strength boulder and its Gym palette retain
their own shared asset and material behavior. Runtime actor identity, collision,
placement and movement remain authoritative. Review Cianwood `(10, 18)`, orbit
`1`, zoom `0`, with `LR` at noon and 22:00 against matching native 2D.

Coastal reflections and bright crests use short crossing wave groups; coastal
water has no pool-floor caustic overlay. These optical effects change normals
and color only, never water geometry or height.
Kanto inland ponds use the same finite crossing wavelets at their calmer
inland strength. Soft, warped contour light replaces the continuous silver
ribbons on narrow pools. Indoor pools retain their original caustics and low
ripples; exposed coasts retain their existing wave treatment. Inspect Route 6
from the dry north bank `(10, 9)` at noon and 22:00, with a short `LR` walk,
and compare Cerulean Gym `(4, 14)` and Olivine Port `(10, 8)` as controls.

Raised pier decks do not seed the shoreline-distance field: water remains
continuous beneath them, so floating decks no longer acquire a square beach
of foam and shallow-water tint. The same native drawing classifier owns both
deck geometry and this optical treatment. A native front-face drawing also
keeps water beneath it when its matching deck is directly north; an isolated
face drawing does not qualify. Route 12's supported soil islands
remain dry shoreline sources. Review the isolated Route 20 pier at `(13, 9)`
against 2D, including a short `LR` walk along its deck.

The shared Kanto brick-building panes use individual normalized glass
coordinates for recessed shading and a subdued daylight sky reflection.
Their native pigment and lamp identity remain separate: dark source panes
receive glass shading but never emit light, and the reflection fades out at
night. Masonry openings, source ownership, building geometry and walking
support are unchanged. Review Vermilion City `(14, 17)` at noon and 22:00
against its matching 2D view; the alternating night lights must match source.
Source glass includes a subdued lower-hemisphere reflection as well as sky,
so vertical panes retain depth under the raised game camera. The recessed
gradient and broad reflection remain subordinate to the source pigment.
Also review Lavender Town's repeated tower panes from `(14, 7)` at noon and
22:00, including a short `LR` walk; its alternating native lamps stay intact.

The shared Kanto museum hall uses the same pane finish and retains each native
lit/dark window identity. Source-lit panes use the existing warm night emission;
dark panes never emit, even when the daytime pigments are similar. Museum roof
and masonry coordinates fit the bounded town-paint domain before placement,
preventing deep front hips or wide halls from losing their source palette and
receiving a second night tint. Geometry, source ownership and support stay
fixed. Review Pewter City `(15, 12)` at noon and 22:00 against 2D, including
a short `LR` walk in front of the flower border.

Kanto's narrow capped posts retain their single shared sculpt and use the
current source cap/shaft palette instead of fixed gray model paint. The eight
component material names are exported explicitly for the runtime adapter.
Source-colored finish prevents a second nighttime tint; missing, transparent,
incomplete or unsupported multicolor samples retain the whole source fallback.
The native footprint, spacing, source claims and walking support are unchanged. Review Fuchsia City
`(24, 22)` at noon and 22:00 against 2D, with a short `UD` walk on the path.
The source-painted shafts use vertical timber grain and the pale caps use
the existing rough mineral finish. Coordinates come from the immutable sculpt,
so every post retains stationary grain across scrolling and day/night palette
updates. The closed sculpt uses 24 radial sides and 576 triangles within the same
exact envelope. Authored corner normals smooth the rounded dome while the
shaft, collar and foot retain crisp edges; open spacing stays unchanged. Review Fuchsia City `(18, 6)` and Vermilion City `(26, 12)`, orbit
`1`, zoom `0`, noon and 22:00 against 2D, including a short `LR` walk.

Non-forest outdoor sign inscriptions retain the live atlas's time-of-day
palette while receiving neutral source-paint shadow/contact modulation. A
reversible UV tag is decoded before atlas sampling and the normal prepass;
the cropped lettering is neither repainted nor copied into a new texture.
The frame and floor backing keep their existing materials. Forest signs keep
their prior treatment. Review Celadon City `(26, 22)` at noon and 22:00, with
a short `LR` walk, against matching 2D; night panels must retain the source's
muted purple instead of acquiring a second saturated blue cast.
Successfully modeled non-forest framed signs also maintain a small tapered
grass margin. Only nearby blade height and corresponding wind height change;
roots, footprint, pigment, source encounter ownership and walking support stay
intact. Corrected blade normals follow the taper. Unknown/custom/unappended
signs cannot authorize a margin, and forest signs keep their existing grass.
Review Route 6 `(17, 9)` by day and at 22:00 with `LR`, plus a closer stationary
view from `(17, 6)`, against 2D: the board should remain visible through the
adjoining grass without moving or enlarging it.

The shared `$0a` Johto rock platform uses the same outdoor eligibility for
mineral paint as for mountain relief. This includes named sites such as
`RuinsOfAlphOutside`, `LakeOfRage` and `SilverCaveOutside`; suffix-only paint
checks had left raw pixel-art strips through their rendered stone. The fix
reuses the existing canyon producer and adds no new source classification,
height definition, asset or terrain footprint. Newly finished caps receive the
same existing bounded relief as neighboring stone. Walking support, water,
portal openings, forest maps and cave interiors retain their producers.
Review Alph `(14, 10)`, orbit `1`, zoom `0`, with `DU` at noon and 22:00,
and the north-gate approach `(7, 7)` with `UD`. Press `UUUU` from `(14, 10)`
to verify the unchanged Kabuto chamber entrance. External footprint checks
cover all three named outdoor sites, including every open cell.

Cinnabar's mountain finish retains cool volcanic pigment through the shared
outdoor rock shader. Its material tag preserves the same map-space coordinates
while applying neutral mineral grain instead of ochre sediment paint. Existing
relief, ledge geometry, walking support and the Center remain unchanged; other
maps keep their existing stone material. Review `CinnabarIsland` at `(10, 13)`
at noon and 22:00, and walk `LLRR` along the waterfront to check registration.

## Verification

The native location tester renders the production cameras and can capture a
2D reference and 3D view with the same authoritative state:

```sh
cargo build --locked -p crystal-bevy --example render_at_location --features location-tester
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_QUALITY=high \
CRYSTAL_RENDER_TEST_CAMERA='1 0' CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
target/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map ElmsLab --x 4 --y 3 --hour 12 --view both \
  --screenshot /tmp/geothite-lab-review.png
```

The outputs include a comparison image and a paired-state receipt. Keep capture
outputs outside tracked source.

For Kanto water, capture `VermilionCity` at `(20, 25)`, `Route19` at `(10, 12)`
and `CeladonCity` at `(12, 20)` at noon and 22:00. Use the same tester command
above, changing `--map`, `--x`, `--y`, `--hour` and the screenshot path. To
inspect actual motion, use `--view 2.5d` with
`CRYSTAL_RENDER_TEST_SECOND_VIEW=same`; this captures two production frames
without the paired-state animation hold. Water should move while banks and
walking support remain fixed. Inspect Cerulean Gym's indoor pool at `(4, 14)`
with `CRYSTAL_RENDER_TEST_CAMERA='1 0 65'` and the same paired-view capture.
The water regressions check source eligibility,
palette exposure, phase continuity, shoreline registration and the
coastal/inland style split:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view --lib water
```

For outdoor building occlusion, Lavender Town at `(14, 7)` places the trainer
behind the low brick building south of the tower. Its roof, walls and windows
remain faintly visible while the trainer stays clear. The normal movement route
`ULLLLLLRRRRRRD` goes around the nearby NPC and sign, clears that building at
`(8, 6)`, then returns behind it. Check that the building regains full opacity
when clear and fades again on return, at both noon and 22:00. With the native
`new_bark_3d` location tester, use `--walk ULLLLLLRRRRRRD` together with
`CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic` and
`CRYSTAL_RENDER_TEST_WALK_STEPS=1`. Inspect the moving captures and authoritative
positions in the movement CSV; repeated blocked inputs do not verify restoration.

Outdoor buildings that already own depth coverage now use smooth faint
transparency, retaining their roofs, walls and windows without stippling the
street. Native alpha cutouts still reject their transparent texels. Their
feathered body aperture uses the current authoritative support capsule. Capsule
updates continue after opacity settles, including walking and camera changes;
coverage-only forest gates, foliage and room-wall reveal policies remain
unchanged. This changes no mesh geometry, original paint, shadows, support
height or collision. Review Saffron `(20, 24)`, orbit `1`, zoom `0`, at noon
and 22:00; the city must remain flat. Also review Goldenrod City
at `(15, 22)`, noon and 22:00, with a short `LR` walk behind the foreground
building. The trainer's hat, face, clothing and shoes must remain readable.

Also inspect the tower from Route 12 at `(8, 1)` with
`CRYSTAL_RENDER_TEST_CAMERA='0 0 0 1'`. Walk `UUUUDDDD` to enter Lavender and
return. The tower must retain its brick facade and full height across the map
change; a short plaster tower indicates that its connected roof was omitted.

The browser/WebGPU compilation check is:

```sh
cargo check --locked --target wasm32-unknown-unknown -p crystal-bevy \
  --features voxel-view,browser-webgpu
```

The Kanto workyard's complete four-block enclosure shares the existing irregular
boulder sculpt across all eight round stone drawings. Closed beveled rails join
the corner stones; the four central stones use the same asset. The rounded
stone silhouette replaces the former square-footed harbor bollards without
changing the canonical boulder or harbor geometry. Exact 8-by-8 source
proof and unclaimed cells are required before replacement. Rail and stone paint
comes from the live source palette, while the interior retains its existing
ground and walking support. Review Vermilion City `(26, 12)`, orbit `1`, zoom
`0`, at noon and 22:00 against matching 2D with a short `LR` walk.

The shared two-storey house keeps its upper storey, gable, tiled roof and chimney
on a single plot-centered fit above the authored floor division. Only the lower
facade follows an off-center native door; its doorway band uses the existing
bounded fit. This prevents the broad dark roof deck from crossing the small
ceramic courses. The asset metadata and generator carry the same division;
other building assets retain their fitting policy. Review Route40 `(9, 6)` and
New Bark Town `(13, 7)` against 2D, including the roof's complete slopes.

The outdoor drawings on either side of Route40's Battle Tower gate share a
presentation connection. Complete gate block signatures and their authored
warps into the same gate room establish the alignment. The real Battle Tower
map supplies the northern scenery when Route40 is active, and Route40 supplies
the southern scenery from the Tower grounds. This is part of the existing
bounded render halo and block-identity invalidation; authoritative map
connections, collision, movement and warp routing are unchanged. The active
map retains overlap precedence. Review Route40 `(9, 6)` at noon and 22:00,
orbit `1`, zoom `0`, against the paired 2D state.

Shared route notice boards retain their existing geometry, native paint and
raised ink. Model-local grain runs along the timber frame and vertically up
the posts; the recessed enamel uses the existing subtle ceramic finish.
Brass fixings and notice strokes remain clean source paint. The coordinates
come from the immutable shared sculpt, preserving grain across scrolling and
day/night palette updates. Review Cianwood City `(14, 32)`, orbit `1`, zoom
`0`, noon and 22:00, with matching 2D and a short `LR` walk.

The authored-scenery audit reports `unmodeled_facade_cells` with source-cell
column/row and metatile/tile identity, so a remaining facade can be located
before altering a model. Isolated-map coverage omits neighboring map context:
Mahogany Town's 32 facade cells at columns 16–23, rows 0–3 belong to the north
gate completed across the Route 43 boundary in production. Review that gate
from Mahogany `(9, 3)` against matching 2D before interpreting those cells as
a missing building. Coverage counts alone do not establish visual quality.

Kanto round boundary posts use a flatter cut crown over their closed planted
body. The seated basal ring and original plot bounds remain fixed. Cached
boundary derivatives retain continuous normals; ordinary boulders keep their
facets. Explicit exported material labels bind native pale cap and darker shaft
pigment to complete canonical faces, so reshaped derivatives cannot acquire
dark triangle wedges. Pigment uses restrained mineral chroma and model-local
weathering. Offshore source-painted posts retain the standing-water marker and
a separate lighting marker to avoid a second night tint. Review Route 24
`(9, 17)` at noon/22:00 with `UD`, and Route 25 `(47, 8)` at noon with `DU`,
orbit `1`, zoom `0`, against matching 2D. Terrain heights and water levels stay
with their existing producers.

The complete Kanto `0x7a` rock-mound drawing uses a shared three-boulder pile
with native mineral pigment and the existing rough stone finish. Recognition
requires all 16 source cells, their subtile phases, opaque native paint and
level support; partial drawings and different road blocks retain their normal
rendering. The underlying support and collision remain authoritative, so
scripted road clearing removes the pile through the ordinary block update.
Review Route 19 `(9, 5)` at noon and 22:00, orbit `1`, zoom `0`, against the
matching 2D state. The headless audit includes each block edit's original block
identity to distinguish the base road from its scripted closure states.

The source-bound doorless Kanto hall repeats native bays with plot width.
Compact drawings have three panes; the wider drawing has five, including its
independent dark pane. Complete metatile, subtile and tile-art signatures own
one shared producer. The source window positions, pigment and lamp identity
supply the closed glazing directly, and dark glass stays out of the emitter
list. Overlapping enamel pans, folded seams and a louvered vent finish the flat
roof. Its height retains the previous model's 5.332-source-unit maximum.
Source support, the separate door-bearing cottage, Center and Mart producers,
and the existing generic fallback models remain unchanged. Review Cerulean
`(16, 16)` at noon and 22:00, orbit `1`, zoom `0`, against matching 2D.
The audit reports native window-cell locations to make frontage mismatches
visible before choosing a model's pane count.

Native park ponds now use the shared source-palette water optics for complete animated park water in National Park, its Bug Contest variant, and Safari Zone Beta. Blue/white source pigment and source animation drive the surface; a sheltered style reduces wave strength and breaking wash around fountain footings. Static fountain-bowl water, geometry, banks, collision and water heights remain unchanged. National Park was visually checked against matched 2D at noon and 22:00 using headless image-target captures only; both pairs preserved state. The 16 water-source and 13 shoreline/clock/contact regressions passed. This verifies the reviewed pond material, not whole-world visual completion.

Goldenrod Game Corner review: the shared modern arcade now has a closed marquee frame, recessed violet enamel, raised solid GAME lettering and six jewel lamps in sockets. The original footprint [-2.975,-1.95]..[2.975,1.95], height 3.026 and south door anchor [0,0,1.95] are unchanged; the canonical model has 3968 triangles. The editable source is art/johto/source/world-exteriors-modern_arcade.blend, regenerated with tools/build-world-exteriors.py --only=modern_arcade --skip-preview. Kanto arcade and service buildings retain their existing producers. Noon/22:00 Goldenrod (15,24), orbit1/zoom0 were personally reviewed against matched native 2D via headless image targets, both state_preserved=true; 22 exterior-model checks pass.

The Game Corner review also exposed a connected-atlas crop failure: a ground tile number present only in another atlas could pass the global availability guard, then fail the source-bound renderer lookup and disable the complete 3D scene. Facade/cutout admission now checks the matching native atlas as well. Missing evidence retains the existing flat source fallback only for that clipped shape. The reproducing synthetic regression, clipped-profile baseline and four native-ground regressions pass; one external-pack regression remains ignored. The exact formerly failing Goldenrod pose now renders successfully in both final captures. The headless asset audit also records inclusive source-cell bounds per authored family to locate landmarks without guessed camera positions.

Outdoor approach mats now retain a visible diagonal weave at gameplay distance, with a narrow bound edge and stitched seam. A source-bound orientation tag lets the same material span the 16x32 Kanto side mat and the 32x16 Johto entrance without restarting at source-cell seams. Live source pigment, original floor triangles/normals, contours, support, collision and placement remain unchanged; other room fabrics keep their previous treatment. Route16 noon and Route35 (9,32) noon/22:00 were personally reviewed against matched native 2D using headless image targets, all state_preserved=true. Two mat regressions and eleven shared brick-office/gate regressions pass. Celadon central/dept-store reviews justified no building changes; a ten-route western Kanto audit had no mesher errors. These checks verify the reviewed approaches, not whole-world visual completion.

Route12 raised planting uses muted organic pigment and the retained granular ground finish instead of saturated lime panels. The exact native flowerbed drawing (Kanto0x74) supplies day/night exposure, including blue night grass; the sampler no longer relies on green being the largest channel. Its original top quad, flower base, structure and support stay exact. The mixed landing (0x79) retains separate paving: walkable0x30/0x39 cells sample the verified0x30 background in their own metatile and use the shared native paving grain, avoiding both a daytime-green night fallback and a tile-sized checkerboard. Other mixed-landing cells retain their previous pigment. Material coordinates include source-grid origin, so viewport movement cannot slide the surface grain.

The final Route12 noon/22:00 matched 3D/2D image pairs were personally inspected using HEADLESS=1 and CAPTURE_TARGET=image; both state_preserved=true at zoom0/rotation0.5/pitch45. Four elevated-pier regressions pass, including the new palette/geometry/scroll/landing-continuity regression and the external-pack support, water-level and approach checks across eleven maps. Water, deck/landing heights, collision, and tree assets were not changed. This verifies the reviewed material pass; the broader outside-quality goal remains active.

The shared tall Tin Tower model now follows the native drawing's eight-tier rhythm, with hipped ceramic courses, lifted eaves, hip/ridge caps, bracket blocks and square cedar braces. Solid cedar doors and framed wooden shutters replace the office-style glazing. The existing native roof and Ecruteak facade samplers now paint the tall source type wherever that drawing is used; ceramic and cedar grain use immutable model coordinates. The separate low temple drawing and Burned Tower producer retain their existing geometry. The canonical source remains one model, with editable art/johto/source/world-exteriors-tin_tower.blend regenerated through tools/build-world-exteriors.py --only=tin_tower --skip-preview. Its12556 authored triangles remain below the existing13000 budget, with exact previous bounds[-2.95,0,-2.25]..[2.95,10.1,2.25] and door anchor[0,0,2.25].

Ecruteak(36,10), noon/22:00 was personally inspected against matched native2D, including a normal-camera close view and complete tier/finial views. All captures used HEADLESS=1 and CAPTURE_TARGET=image; each final pair preserved state. The23 exterior-model checks (including eight structural roof tiers, fit, source paint and translation-stable grain) and6 city-light controls pass. No terrain, water, vegetation or support heights changed. Whole-world visual completion remains unproven.

Tall-landmark QA may explicitly set CRYSTAL_RENDER_TEST_FOV_DEGREES=110 (accepted range30..120) for a one-shot capture. This opt-in affects only active3D capture projections after world synchronization; gameplay defaults and the paired2D camera stay unchanged. The full Tin Tower day/night captures used this override at zoom0/orbit0,640x800; the close view used ordinary FOV and orbit1.
