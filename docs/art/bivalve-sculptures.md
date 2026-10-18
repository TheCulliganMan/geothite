# Shellder and Cloyster sculptures and rigs

`tools/bivalve_sculpt.py` authors original hollow mollusk shells, and
`tools/bivalve_glb.py` exports their canonical skins. The runtime, main builder
and model checker use the same GLBs; the legacy static JSON files are removed.

Shellder's lavender upper valve now tilts over the black mollusk body instead
of framing it as a vertical ring. A separate lower valve tilts upward beneath
the face. Dark interior linings, four rooted valve points, a seated rear hinge,
fitted pale oval eyes and pupils, three supported shell grooves and a broad
pink tongue with a fitted fold complete the model.

Cloyster has two separately closed hollow outer armor halves, slightly opened
around their rear hinge. Each half carries its own four purple lances. A
separate blue-gray mantle carries the forward horn and a recessed charcoal
lining. A darker purple central orb carries the sharp pale gaze, pupils,
curved four-panel grin and tooth divisions. The tall rear crown remains rooted
between the armor halves. The cups retain rear walls, interior walls, joined
opening rims and closed hinge cuts.

The official [Shellder](https://www.pokemon.com/us/pokedex/shellder) and
[Cloyster](https://www.pokemon.com/us/pokedex/cloyster) artwork informed the
shapes and palettes. No reference pixels or external geometry are shipped.
These are original authored models with new authored skins and clips.

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Shellder | 19 | 15,585 | 22,292 | 6 | 88 | 544,000 | 1.05 | 0.3048 m |
| Cloyster | 24 | 17,021 | 23,892 | 5 | 100 | 657,744 | 1.05 | 1.4986 m |

The verified local pack supplies height digits 100 and 411, respectively;
both are classified BIVALVE. Neutral geometry digests:

- Shellder: `9603bdedd0b70757c615251f8ea5ea6b258a33c53c3ce6ab434f2f441edd95e9`.
- Cloyster: `714eb4a74d131c5a8b303d1fde4af07a30dd0c739fa0741176cee1efbb336d74`.

A fixed floor root and yaw-only body carry each coherent face. Shellder's
upper hood hinges independently, the lower valve braces with a separate yaw
carrier, and the tongue blends through a base and fold joint. Its crease
shares the same position-dependent influences. Cloyster's left and right
armor valves open independently around rear hinges; spikes follow their own
valve, while the mantle, lining and forward horn share another carrier. All
three rotation-only idle, attack and hit clips retain neutral endpoints and
source-cue timing. The authored clips preserve their shell floor baseline.
Production battle and world presentation retain the existing mollusk hover;
world movement preserves the authored valves and face ownership.

Fifteen Python checks cover canonical bounded containers, hierarchy and inverse
binds, normalized weights and anatomical owners, occupied shell/lining/hinge/
spike/tongue roots through motion with a detached negative control, floor
contact, independent articulation, endpoints, winding, strain, source cues,
closed outward solids, unit normals, exact geometry parity, physical size,
anatomical counts, recessed cavities, fitted face clearance and supported
surface details. Existing 20,000-vertex, 40,000-triangle, 64-primitive,
256-accessor and 1 MiB limits remain unchanged. Shiny checks are skipped at the
user's request.

Four focused Rust checks pass for both verified-size GPU paths, production
routing and anatomical ownership, shell contact, animated culling bounds,
independent valves and tongue, and 64 phases of world swimming with the
authored face and valve ownership retained. Front, side and rear poses of all
three clips were inspected for both models. The rear hinge was seated deeper
after the first side review. Final Blender generation is byte-identical.

Rebuild and check:

```sh
python3 tools/bivalve_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p 'test_bivalve*.py'
```

The rebuilt production client passes normal Route 29 terrain admission at
original pack size for both models. The inspected Shellder/Pidgey 3D/2D pair
preserves core frame 2592, checksum 1129406471, LCD animation frame 3197 and
virtual time 55.903143251. The Cloyster/Pidgey pair preserves frame 3367,
checksum 3177080196, LCD frame 4078 and time 72.56970316499999. Both show the
full silhouettes, correct species HUDs, coherent faces and the existing
mollusk hover placement. Shellder's hood overhang, seated hinge and tongue are
visible; Cloyster's dark interior, grin, complete armor and rooted horns remain
readable. Neither capture reports shader errors.

Final ignored evidence uses `target/pokemon-native-review/shellder-normal`
and `target/pokemon-native-review/cloyster-normal`. Both captures select the
existing supported -110-degree angle while retaining normal size, source
support, scenery omission and whole-body/frustum checks. The shared
all-species grounded/volumetric and source-cue pose timing regressions pass.
Grass height is untouched. Browser visual acceptance remains open.
