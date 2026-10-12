# Angular thunder bird with black underfeathers

`tools/zapdos_sculpt.py` replaces the shared legendary-bird primitive model
with an original Zapdos sculpture. The battle-species builder dispatches this
pure recipe and rejects the earlier Zapdos route. Moltres and the separate
shared overworld props retain their existing sources.

Official [normal-form Zapdos artwork](https://sg.portal-pokemon.com/pokedex/0145/)
was inspected visually in a temporary browser tab, which was closed after the
review. No reference image was downloaded into the repository. Its sharp
silhouette guided the original wing, crown, face, breast and tail geometry.
The [external candidate review](legendary-bird-rig-research.md) has not yielded
a validated usable rig. No third-party geometry was imported. This sculpture
is static.

Two broad gold wing fans have angular outer points and deep zigzag edges.
Each fan has three rooted black underfeathers, which make the second layer
read clearly from behind. Gold shoulder coverts fit the actual stored wing
triangles. A lean breast tapers into the abdomen, with rooted jagged gold
side feathers. Grey feather spats connect the torso to narrow brown-grey shins;
each foot has three forward digits, a rear digit and ivory talon tips.

The compact gold head has three long lightning crown points and two jagged
cheek rays. Narrow ivory eyes, small dark pupils and black sockets fit the
actual cranium. The long ochre bill tapers downward through independently
closed upper and lower lance shapes, separated by a narrow charcoal join.
The tail is a sweeping five-point gold fan over a black underfan. Both fans
join the actual body volume.

The first software view exposed an overly round torso. Narrowing its lower
profiles and adding rooted feather points improved the silhouette. The root
regression then caught that the narrower body no longer supported the original
shin caps. Actual grey thigh feather solids now join both body and shin;
the final test checks the complete body/thigh/shin/foot chain. The attachment
check was not relaxed.

Final Zapdos has 54 parts and 32,360 triangles. Its presentation height remains
1.28 and its lowest point is zero. Its canonical SHA-256 is
`36be6eb0fa6cf91e0dc84274593804194fd0a4df2b788ea08d1d09b81b34f6e9`.

The final candidate passed eleven stored-mesh tests in 5.159 seconds. The
installed canonical source passed all eleven again in 5.315 seconds. Tests
cover connected closed outward solids, finite unit normals and actual face
winding, recipe parity, height, grounding, feature counts and triangle budget.
Independent nonadjacent triangle checks cover body, neck, crown, wings, black
underfeathers, cheek and breast rays, thighs and tail. Independent stored-ray
volume checks cover anatomical roots, both wing assemblies, both tail layers
and complete digit/talon chains. Fitted eye, shoulder and throat vertices,
front triangle centers and edge midpoints are checked against actual parent
triangles. Deliberately detached wing and tail negative controls fail.

Final front, oblique and rear software images completed with exit zero and
were individually inspected after the final geometry generation. Earlier
images were superseded. Canonical layout validation passed, storage validation
passed for the current shared tree of 572 raw JSON model documents, all three
edited Python files parsed successfully, and the relevant Git whitespace check
passed.

The fresh native build completed with exit zero in 6 minutes 59 seconds,
including its shared build-directory wait. The production GPU capture
completed with exit zero in 4.11 seconds for the complete verification command.
Its 2,000 by 1,400 image has 46,971 distinct colors and was inspected in full:
Zapdos's gold zigzag wings, black underfeathers, long pointed bill, crown,
feathered breast, grounded digits and trailing fan are visible alongside the
opponent, arena and production battle HUD.

Before launch, the executable contained the complete canonical source SHA-256
shown above and the compiled mute switch. Its inode, size and modification
timestamp and the canonical model hash stayed unchanged across the capture.
`CRYSTAL_PREVIEW_MUTE=1` was set before process creation, and the native backend
sets both music and effect sinks to zero before appending samples and keeps
music-volume refreshes at zero. No system volume or other application was
changed. The capture explicitly targeted the owned GPU image.

The fresh Rust all-251 regression completed with exit zero: its matching test
passed in 1.08 seconds, with 1,253 other tests filtered from that crate. The
overall test build took 11 minutes 05 seconds including the shared-cache wait.
The renderer reported writing the GPU image in 0.90 seconds. This image verifies
the replacement in the production GPU image path; it does not establish gameplay,
resolve the separate live-window capture problem or prove every model's
appearance finished. The full 251-species appearance goal remains open.
