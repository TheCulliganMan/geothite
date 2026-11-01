# Caterpie and Weedle sculptures and rigs

The generic detached-looking larva segments are replaced by continuous,
annulated bodies with raised heads and curled tails. Caterpie has a continuous
cream underside, five hollow yellow circles on each green side, six raised
cream leg knobs, broad yellow eye rims, black lenses and fitted ivory glints,
a two-part green snout, a red Y antenna with rounded ends and a soft pale tail
point. Weedle has a tan body, fourteen pink prolegs including six planted pads,
a broad rosy nose with a fitted reflection, small glossy black eyes and curved
ivory head and tail stings. Neck caps are buried inside the heads. The visual
references are the official [Caterpie](https://www.pokemon.com/us/pokedex/caterpie)
and [Weedle](https://www.pokemon.com/us/pokedex/weedle) artwork.
Reference pixels are not shipped.

`tools/larva_sculpt.py` authors both species. `tools/larva_glb.py` exports
canonical skins within all existing renderer and schema limits:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Caterpie | 33 | 13,539 | 25,360 | 12 | 152 | 507,296 | 0.95 | 0.3048 m |
| Weedle | 24 | 10,537 | 19,772 | 12 | 112 | 415,840 | 0.95 | 0.3048 m |

The verified local pack supplies height digits 100 for both species, classified
WORM and HAIRY BUG respectively. Original pack size is retained. Neutral digests:

- Caterpie: `527fcc82fab88435eb538b1b93614b03f896566ecefafc9d0f6e05e5cb419931`.
- Weedle: `ef69247a874d0f1cec8f314d3885cd93ddaa5356fd45ce850dda1f75a0a162ea`.

Eight blended body carriers create small crawling waves. A coherent head
carries the complete gaze and snout. Caterpie's fork branches articulate
separately; its rounded antenna ends share those carriers. Weedle has separate
head- and tail-sting carriers. Caterpie's lowest belly points and Weedle's
planted pink pads stay on the neutral floor root. Raised prolegs and side rings
use the body skin field. The two closed green/cream body halves share 114 exact
stored boundary points and matching quantized skin influences through every
clip. Rings sit fully on the green surface and retain open centers.
Idle, attack and hit keep neutral endpoints and source-cue timing. No grass,
terrain, scale or camera-admission rules change.

Ten Python checks pass across both assets: canonical bytes and anatomy/limits,
closed outward connected solids, actual normals, fitted reflections, exact
anatomy owners and body carriers, exact animated green/cream seams, rings
seated on the stored green surface, occupied feature roots through motion with
a detached negative control, floor/endpoints/winding/strain/independent motion,
and source-cue metadata. Shiny verification is skipped at the user's request.

Rebuild and check with:

```sh
python3 tools/larva_sculpt.py --out crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p test_larva_glb.py
```

The world crawl adapter reuses the eight authored body joints. It preserves
the continuous skin and both sides of the cream/green seam. Existing world IK
creates six independent Caterpie proleg chains and fourteen Weedle chains;
the head follows its authored front-body parent once. A smaller arch fits the
tiny supports' reach, preventing planted pads from being pulled above the
floor. The world regression samples 64 travel phases, checks support lengths
and level feet, enforces ground contact and a closed material seam, and returns
the complete skin to its exact neutral rest pose. Canonical battle bindings
and source-cue clips remain separate from these world-only leg chains.

Four focused Rust checks pass: runtime routing/anatomical ownership/floor/
culling/material seams and independent tips, both verified-size GPU paths,
and the world crawling regression. The three supporting pairs use staggered
phases so they cannot all enter swing together; this closes Weedle's brief
airborne interval. Front, side and rear poses of all three clips were inspected
for both species, and Blender generation is byte-identical.

All three shared grounded/volumetric, exact source-cue pose timing and readable
map-speed cadence regressions pass. The rebuilt native Route 29 Caterpie
encounter against Pidgey passes normal terrain admission at original pack size.
Its inspected 3D/2D pair preserves core frame 5514, checksum 2831601804,
LCD animation frame 6261 and virtual time 116.21247896. The small larva remains
readable with its red fork, yellow-rimmed gaze, cream body, side markings and
curled tail; the belly meets the ground and the HUD says CATERPIE. No shader
errors were reported. Final ignored evidence uses
`target/pokemon-native-review/caterpie-normal`.

The native Route 29 Weedle encounter against Sentret also passes normal terrain
admission at original pack size. Its inspected pair preserves core frame 5081,
checksum 1179181144, LCD animation frame 5848 and virtual time 113.864353625.
The tan segmented body, rosy nose and prolegs, black gaze and both ivory stings
remain readable with the correct WEEDLE HUD and grounded pink pads. No shader
errors were reported. Final ignored evidence uses
`target/pokemon-native-review/weedle-normal`. Both captures select an existing
supported angle with `CRYSTAL_BATTLE_REVIEW_YAW=-110`, retaining normal scale,
support, omission and whole-body/frustum checks. Browser visual acceptance
remains open for both species.
