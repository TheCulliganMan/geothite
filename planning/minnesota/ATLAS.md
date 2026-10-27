# Minnesota regional atlas

## Current connected progress render

[Open the complete current atlas](renders/atlas/minnesota-current-connected.png).
The full image is **6360×16380 pixels**:20whole routes,13native towns,
Boundary Waters preserve/cabin, **76paired interiors and one unconnected review
hall**. The towns are Grand Marais, Boundary Waters basecamp, Winona, Rochester,
Mankato, Minneapolis, St Paul, St Cloud, Brainerd, Bemidji, Duluth, Grand Rapids
and Leech Lake. Ryan's waterfall house and all nine caves, including the deeper
R12 seep shelter, are included. Actual doors have room tags; Winona Home2F is
paired through the real Home1F stair, and Union Depot uses its actual facade.

All exteriors share one scale. **23of40endpoint leaders are installed walking
links**;17remain dashed planned travel. Native opposite directions/offsets,
two aligned Floor lanes and the corresponding world route/town pair are checked
for every solid leader. This certifies walking geometry. Badge dialogue,
checkpoint ownership and earned progression have separate controller evidence.
Safari admission remains closed and its admission leader remains a proposal.
The diagram does not generate connecting game terrain.

All20complete route grids match native chunks after explicit story overlays.
The20declared route caps are R03six, R06six, R07two and R08six; each source
preimage/replacement is checked. Existing explicit R10 city-mouth adaptations
remain bounded. All70whole town/owned-room modules match the exact Leech Shore
story baseline, including8gyms, guards, scripts, caps, warps and actors. There
is no permissive source-drift exception. The compositor snapshots72whole source
inputs and verifies their bytes again after rendering.

Rooms use the production palette policy: INDOOR/GATE background uses indoor,
NPCs use day; forced-NITE caves stay NITE. Custom native palette files retain
their actual colors. Correcting the older outdoor-day indoor preview changes
54room scenes. The corrected Leech Fisher House and CHRIS composition matches
an existing actual native GPU frame exactly:0of327680pixels differ. No new
production capture is claimed for that inherited check.

The complete overview, all77gallery frames, all13city crops and link sheets
were independently inspected by the author and Root. The dynamically sized
interior gallery has no clipping, including the18×14 R10 room. This is a native
source-art progress composition, not a production GPU screenshot, complete
playthrough, user approval or a claim that all routes have reached Johto quality.

Current pack: `content-packs/minnesota-northstar-seep-shelter-review.crystalpack`,
SHA256 `7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a`.
Full PNG SHA256:
`d8a93d3f7640f3e85bb76f9f283d88a243a65da56f73a0349a82f7afca0e2b77`.
[Immutable image, snapshots, certificates and review](renders/atlas/leech-shore5417e5f/README.md)
retain this exact epoch. Original/rejected/conversion/GPU outputs stay ignored
and retained; prior aliases are archived before replacement.
[Gameplay and geometry runbook](playable/winding-passages.md).

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_progress_atlas.py \
  --pack content-packs/text-tui.crystalpack \
  --region-pack content-packs/minnesota-northstar-seep-shelter-review.crystalpack \
  --story-baseline .local/minnesota-city-revisions/leech-shore-03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97.crystalpack \
  --output planning/minnesota/renders/atlas/leech-shore5417e5f/reproduction/minnesota-connected.png
```

The strict helper supports two explicit story epochs: DA27/538 and03c9/541.
Wrong baseline hashes, changed town/room programs, missing reciprocal lanes and
undeclared terrain differences reject. The older510/6071 mode remains a separate
legacy path. Its actual rerender passed with6towns/7links/47paired+6review rooms
before source promotion. Matching frozen inputs are required for historical
reproduction; do not feed current layouts to old packs.

## Previous 510 and 538 epochs

The [510road/stair atlas](renders/atlas/road-bends-6071912e-native-facing/)
was6360×13572 with6towns/7links/47paired+6review rooms. Its PNG SHA256 was
`3a2a90ac67471b889de2c3d4ce745baee30c896c671f35f8913b12c682a31db3`.
The [538story integration](renders/atlas/map-quality5389aa9/README.md) added
12towns/19links/74paired+1review, preserving exact DA27 story modules. Both
remain immutable historical compositions. The current541 publication combines
the later Leech Shore chapter with the geometry improvements rather than
substituting their older gyms or missing story records.

## Historical composition commands

These commands record older pack/source epochs. Their matching frozen layouts
are required to reproduce them; they are not the current541-map recipe.
The earlier469-map regional composition was reviewed with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_progress_atlas.py \
  --region-pack content-packs/minnesota-region-review.crystalpack
```

The471-map garden/pine epoch added the basecamp and Survey Shelter, the earlier
R05/R08 edits, rockier R09/R10, R11 rivers/falls, golden R02/R16 grass, R04 garden
and R06/R07 pine edges. The498-map epoch added Brainerd, Bemidji and Duluth with
eighteen paired city rooms, using captured887afc43 canonical gyms. Later live
gym drafts were not substituted into those packs.

The505-map expanded Northwoods atlas had five towns,43 paired interiors and
six review rooms, and measured6360×13032 pixels. It included the larger R11
and preserve, with only the three basecamp links solid. That older render used:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_progress_atlas.py \
  --pack content-packs/minnesota-northwoods-expanded-review.crystalpack \
  --region-pack content-packs/minnesota-northwoods-expanded-review.crystalpack
```

The historical510 epoch additionally included Grand Rapids and its three rooms,
Ryan's house, the revised caves, the shore bends and the four installed city
links. Earlier render files remain in the atlas history. See the
[regional composition runbook](playable/region-review.md) and the
[current house/links evidence](playable/ryans-house.md) for their respective
native build and gameplay scopes.

## Earlier condensed geographic illustration

`tools/render_region_atlas.py` draws the complete nineteen-destination,
twenty-route graph from `world.json` as a condensed source-art overview. Its
route bends, returning paths, terrain districts, habitat fingers and building
lots are authored in the renderer. It does not use the geographic generator,
copy whole source maps, export a pack, or change the gameplay chunk dimensions.

The renderer reads the supplied local pack in memory, verifies its frame hash
and length, and uses its unchanged native tiles and palettes. Grand Marais uses
the actual shared `playable/grand-marais.json` layout directly. Keep generated
images under the ignored `renders/atlas/` directory.

```sh
.local/minnesota-atlas-env/bin/python planning/minnesota/tools/render_region_atlas.py \
  content-packs/core-modular.browser.crystalpack
```

Run that command from the repository root after installing Pillow and cbor2 in
a local environment. The output is `minnesota-region-native.png`
(original 8-pixel source tiles) and `minnesota-region-preview.png` (an annotated,
area-reduced thumbnail). Thumbnail filtering never changes the native pixels.

This is an art iteration, not accepted finished geography or a playable region.
The overview still needs individually fitted town streets, terrain junctions,
shorelines and route landmarks before meeting the Johto reference. Its material
canvas is separate from the large implementation layouts: do not infer map
connections, collision, warps or progression from the overview image.
