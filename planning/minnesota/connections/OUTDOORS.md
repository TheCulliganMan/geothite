# Continuous outdoors and the Mississippi

The requested result is real outdoor connections, a continuous Surf trip from
Winona through St. Paul and Minneapolis to St. Cloud, and a state render using
the actual exterior maps at one scale. A diagram link does not implement a seam.

## Actual native river publication

The resulting local native pack is
`content-packs/minnesota-mississippi-connected-v02.crystalpack`, SHA256
`6b36b4b835873e4c4508a7da7727ecb759273c26e219ec026c061c0e66f24775`.
The v02 successor corrects native location/binding metadata for the 17 new
routes; v01 and all of its images/plans remain archived. It contains 677 maps,
including 94 Minnesota exteriors. The actual-map atlas is
`renders/outdoor-connections/native-mississippi-v02/full-state-native-current.png`;
`winona-st-cloud-native-connected.png` shows the complete 26-map river corridor
at a larger common scale. Both were personally inspected. All 94 full-size
native originals, intermediate conversions, and prior images are retained.

The actual collision/connection recheck binds `all-bridges-input-plan.json` in
that folder to this exact native output. Its 1,269 production-input steps start
on Winona Floor(25,11), launch Surf right to Water(26,11), visit the western
St. Paul basin and every one of seven bridge apertures, climb Minneapolis Falls
from(35,10) to(35,5), and finish at St. Cloud Water(40,20). It checks native
bridge axes, every seam, and standing actor exclusions. Root subsequently ran both named production-controller cases successfully:
`.local/mississippi-controller-v01/journey-v02.log` proves the continuous native
Surf journey/bridge Continue and retained Foot road/upper save. The current
corridor image references `native-journey-evidence.json`: 1,269 route inputs and
all seven bridge apertures pass under the declared local party/badge fixture.
No earned regional progression is implied. The binder itself remains a static
recheck: it opens only an in-memory gate copy and never changes the pack.

The main direct-seam atlas group now has 62 exterior maps; explicit seams and
warps reach 63 of94 from Winona, including Itasca's gatehouse. The other31
exteriors remain detached in this graph. There are four inherited footprint
overlaps, zero new overlaps, and no invented terrain filling the dark gaps.
The independent written audit and 17-map source-pixel equality evidence are in
`renders/outdoor-connections/mississippi-independent/`.

## Frozen river source

The current source is [mississippi-v02/manifest.json](mississippi-v02/manifest.json),
SHA256 `449ce63508b11513b187e5741ac8b8dc669d9fe420b1c922907dd301d4e2b814`.
It is based on the completed local pack
`content-packs/minnesota-twin-cities-compact-v01.crystalpack`, SHA256
`2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e`.
That parent contains the compact 40×32 Minneapolis and 40×34 St. Paul.
Separate Fair, Ely, and regional Pokémon branches are not assumed integrated.

Seventeen proposed maps in reserved group 82 clip four hand-authored layouts:
a 74×12 confluence, 64×68 lower wooded river, 16×4 Winona mouth, and 64×68
upper wooded river. They use native source art, quiet winding channels, trees,
a few grass banks, and five optional trainers. Existing city rooms, activities,
actors, road links, audio, and progression remain in the parent. Bounded native
preimages and exact block/module hashes are part of the manifest.

Final layouts contain supported explicit row runs. The original reviewed
hand-stroke files remain in `mississippi-v02/hand-authored-intent/`.
`tools/bake_explicit_layout_rows.py` encodes those exact pixels; it chooses no
geography and modifies no source-art pixels. Layout hashes cover exact JSON
bytes; clipped block hashes cover the raw u8 block bytes, not a JSON array.

The route runs Winona → Winona mouth → long Lower Woods → St. Paul's eastern
waterfront → the south outlet →
confluence → R03's existing Minnesota River mouth → Minneapolis → long Upper
Woods → St. Cloud's eastern pond. Inland Rochester/Mankato roads retain their
identity. A visible six-cell rock bar across R05 closes the old continuous-Surf
shortcut. Lower Woods has no direct confluence bypass around St. Paul.

Three undirected internal chunk boundaries contain only closed forest shore:
Lower Woods C00R01 south/C00R02 north, Upper Woods C00R01 east/C01R01 west,
and Upper Woods C00R01 south/C00R02 north. Those are exact contiguous source
windows needed for native outdoor adjacency, rather than water ports. Every
new↔old external water port has reciprocal native Water lanes.

## Source verification and limits

`renders/outdoor-connections/mississippi-long-v07/` retains the final connected
source image, all 17 whole-map images, source geometry proof, and preliminary
controller input plan. The author inspected the complete long reaches, three
player-scale camera crops from each reach, and the complete connected source
image. Earlier epochs and original full-resolution images remain available.

The static candidate passes 191 existing actor/event standing-collision checks,
all five trainer landing checks, all three St. Paul basin checks, and a 1,162-step
adjacent Water/Waterfall route. Removing the bridge layer, the upper river, or
St. Paul rejects that trip. This is conditional geometry evidence: the fixture
opens the existing badge gate and supplies declared bridge and Waterfall
capabilities. It is not a native controller or production renderer result.
Both other St. Paul basins are reachable through the two declared city bridge
underpasses; the shortest northbound trip exits the eastern basin directly.
The preliminary input plan leaves `pack_sha256` null until bound to the actual
published output and rechecked by the production controller. Its shortest trip
visits every lower-river window and five upper-river windows; the sixth is an
optional backwater, rather than a required detour.
`source-all-bridges-input-plan.json` adds a legal west-basin St. Paul visit: its
1,269 inputs exercise all seven bridge apertures while retaining the same
Winona Surf launch, Waterfall ascent, long reaches, and St. Cloud endpoint.
The shorter plan contains 1,157 inputs after its initial Surf launch.

The current [bridge inventory](mississippi-v01/bridge-inventory.json) is pinned
to the same compact parent. It lists three Minneapolis north/south water
apertures, R05 and R03 east/west apertures, and two compact St. Paul east/west
apertures. Original walking decks and rails remain upper-layer collision;
bounded appended bridge blocks supply lower-layer Surf collision. Scripted
warps cannot substitute for this: native warps end Surf and reject water
arrivals. The implementation requires native controller continuity, layer-aware
rendering, and save/Continue checks before being described as playable.

## State-wide coverage

`renders/outdoor-connections/v03/full-state-native-current.png` contains all
77 exteriors actually installed in the parent at eight pixels per metatile.
The ten panels are direct exterior seam groups. Explicit native warps link
Itasca through its gatehouse, so these panels do not assert ten disconnected
gameplay regions. Plain seam/warp graph reachability from Winona reaches 46 of
77 exteriors; story-specific transport needs production-controller evidence.
Ely, Detroit Lakes, Moorhead, International Falls, and Mille Lacs are absent
from this parent. Other branches or source plans do not count as publication.

The final footprint proof checks all 77 native exteriors, the 45 with a common
seam-derived origin, and all 17 proposed river maps. There are zero new overlap
pairs or connection-cycle conflicts. The remaining 32 native exteriors have no
unique physical placement from the main direct-seam graph; their detached
panels remain explicit. Four pre-existing overlap pairs remain marked:

- R03C00R01 / R02C00R00.
- R08C00R00 / R14C01R01.
- R12C00R01 / R13C00R00.
- R13C00R00 / R12C00R02.

The old audit's 174 water-edge lanes lacking reciprocal Water are collision
inventory, not 174 proven accessible leaks. Dark atlas areas represent missing
coverage; no interstitial terrain is invented. The actual94-exterior successor atlas above preserves those same detached
regions and inherited overlaps.

## Read-only rerender commands

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_mississippi_connections.py \
  --manifest planning/minnesota/connections/mississippi-v02/manifest.json \
  --output planning/minnesota/renders/outdoor-connections/mississippi-long-v07

.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-twin-cities-compact-v01.crystalpack \
  --output planning/minnesota/renders/outdoor-connections/v03
```

The actual native plan can be rebound without executing gameplay. The binder
accepts only explicit map counts677(default) or681;681 additionally requires the
verified677-map parent plus exactly the four named cohesion maps. All17 frozen
river terrains and every route step are checked in either mode:


```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/bind_mississippi_input_plan.py \
  --pack content-packs/minnesota-mississippi-connected-v02.crystalpack \
  --expected-sha256 6b36b4b835873e4c4508a7da7727ecb759273c26e219ec026c061c0e66f24775 \
  --expected-map-count 677 \
  --source-plan planning/minnesota/renders/outdoor-connections/mississippi-long-v07/source-all-bridges-input-plan.json \
  --manifest planning/minnesota/connections/mississippi-v02/manifest.json \
  --output planning/minnesota/renders/outdoor-connections/native-mississippi-v02
```

Choose a new output epoch for each accepted publication. Both tools preserve
originals before conversions and never export or mutate a content pack. Static
proofs do not replace `VisibleShellController` inputs across every seam,
underpass, gate, and Waterfall, or native save/Continue and production captures.
