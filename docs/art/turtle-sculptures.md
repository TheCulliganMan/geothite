# Wartortle and Blastoise sculptures and skins

The canonical assets are `models/battle_species/wartortle.glb` and
`models/battle_species/blastoise.glb` in `crates/crystal-voxel-view/`.
`tools/wartortle_sculpt.py` and `tools/blastoise_sculpt.py` retain the original
geometry recipes; `tools/turtle_glb.py` authors their skins and strict readers.
The retired JSON assets are removed. Source geometry and rigs are original;
no external models, rigs or animation clips are imported.

Wartortle keeps its blue-violet rounded turtle body, ivory shell rim, fitted
breast and rear scute seams, feather ears and broad two-curl cloud tail.
Blastoise retains its heavy domed shell, muscular forelimbs, narrow gaze,
clawed feet and two real hollow cannons with recessed dark floors and annular
linings. The cannon centerlines have actual empty space, not painted circles.

Both recipes replace uniform dense face subdivision with adaptive fitted
patches. Rounded body and limb samples are concentrated within the existing
render budgets. The current source recipes contain 66 Wartortle and
71 Blastoise solids, including continuous forelimbs and palms. Fitted gaze, seams, folds and insets inherit their actual
parents' smooth corner normals after crease processing. Shell rims and cannon
bores preserve their intentional boundaries. This is local retopology, without
changing body proportions or increasing runtime limits.

| Model | Vertices | Triangles | Logical / runtime parts | Joints | Accessors | Bytes |
|---|---:|---:|---:|---:|---:|---:|
| Wartortle | 19,000 | 30,760 | 66 / 37 | 15 | 201 | 776,932 |
| Blastoise | 19,603 | 32,188 | 71 / 42 | 15 | 213 | 790,888 |

Lossless material grouping preserves each logical part's original order,
geometry and skin weights. Complete feet, toe groups and claws remain separate
by side for world walking. The unchanged limits are 20,000 vertices, 40,000
triangles, 64 runtime parts, 256 accessors and 1 MiB.

The fifteen joints are root, body, head, two shoulder/elbow/wrist chains, two
supported thighs, two species features, tail base and tail tip. Wartortle's
feature carriers flex its feather ears; Blastoise's carriers move complete
rigid cannon assemblies around their rooted mounts. Shell, breast, rim,
scute lines and cannon mounts share the rigid body owner. Head and gaze stay
coherent. Palms and claws follow the corresponding wrist. Foot, toe and foot
claw vertices remain fixed in battle. Leg support blends above the feet.

The cloud fan bends within its own plane: the first out-of-plane attack blend
folded the thin rim, caught by stored-triangle winding checks. Its fitted
spiral lines use the same weight field as the fan. The adult's short tail
retains its own two carriers. Idle lasts 3.2 seconds with 81 keys; attack and
hit each last one source-relative second with 65 keys. All endpoints are
neutral, and production battle cues own playback time.

Logical geometry pins:

- Wartortle: `880d8205d6099ec0b0256c00cc20d8c590ff2b7e0b3281df595634bf49e52d6e`
- Blastoise: `894507c9ac4c53dad84866af1e44e99f44f2bef9452dd732ee12e9e23a93ed69`

Packed geometry pins:

- Wartortle: `8f76359639251ae9161bf295201020e8e4bafafe252c9f286fd3812218eb0a58`
- Blastoise: `0e4dfa76fc3b7fd8a1a2360552fa7c2663e821fd88a442bb899b32dd6e859548`

Nine stored-skin checks independently decode the
stored hierarchy, weights, inverse binds and clips, and cover exact canonical
geometry, lossless grouping, budgets, rigid owners, every independent carrier,
fixed floor supports, edge strain, winding, occupied anatomical roots,
uncrossed limb/ear/tail/cannon surfaces, and empty cannon centerlines against
every posed solid. The strict reader rejects changed geometry and byte
mutations. The thirteen Wartortle and thirteen Blastoise stored-anatomy checks
also pass, including fitted triangle clearance,
rooted claws, detached-tail/ear negative controls and filled-cannon rejection.
Both Blender main-builder exports match canonical bytes exactly.

Ignored review evidence is under `target/pokemon-native-review/`.
Neutral front/side/rear comparisons retain both silhouettes, coherent fitted
surfaces and real cannon openings. Rust production activation/walking and
native GPU multi-view evidence are recorded below. Browser and
moving-world visual acceptance remain open. Shiny verification is skipped
per user direction; grass height is owned by another agent.


The side-view critique led to actual cloud-tail volume: front and back faces
billow away from the interior of the two curled outlines and meet at a thin
rounded rim. Selective tail normal blending smooths the curved surface while
shell and cannon boundaries retain their creases. Fitted spiral lines are
regenerated on both real surfaces. A new independent stored-ray regression
requires substantial variable thickness; the old flat fan fails that shape
requirement. The table and pins above describe this final revision.

Final stored-skin plus Wartortle anatomy checks pass (21 tests, 102.088 s);
the thirteen Blastoise stored-anatomy checks passed on its identical final
bytes. Final Blender exports match both canonical assets exactly. Four final
Rust checks pass (2.38 s): both pack-size GPU activations, rigid anatomical
owners and fixed battle support, plus 64 phases of two complete walking
chains with shell, claws and feature ownership preserved. The actual pack
heights are WARTORTLE = 0.9906 m and BLASTOISE = 1.6002 m. Original geometry
reference spans remain 1.10 m and 1.18 m; no physical-size workaround is used.
The canonical clips' front/side/rear idle, attack and hit sheets were inspected
after the final cloud-tail change. Shared registry/cue and native production
view results follow below.


Both shared checks pass against the 200-entry authored rig registry
(193.15 s): complete species schema/anatomy/physical spans, and production
idle plus exact current-attack-cue timing. Fresh native battle/world examples
build successfully. Both exact canonical assets' Cargo-generated compressed
payloads are present in the native executable. The production GPU capture
sequence uses normal pack size, ordinary camera controls and unmodified
terrain support, omission, whole-body and frustum admission rules.


Native paired Route29 battle views preserve state, with the correct WARTORTLE
and BLASTOISE HUD names, modeled opponents and no fallback. Normal terrain
admission passes at original pack size; support, omission, whole-body and
frustum rules remain intact. Wartortle/Sentret: frame 4187, checksum
3466727637, LCD 4928, time 92.526552793. Blastoise/Sentret: frame 3193,
checksum 1895116822, LCD 3901, time 70.134078791. The full bodies, shell
profiles, claws, feather ears/cloud tail and hollow cannon rims remain
readable against the retained scene.

Stationary Route36 (20,8), noon first-person/elevated pairs also preserve
state. Wartortle front: frame 2340/checksum 799225882; side: frame 1861/
checksum 513033191. Blastoise close face/cannons: frame 2687/checksum
457042847; elevated side: frame 1761/checksum 1417231184. Wartortle's feet
appear supported, its ear/eye surfaces remain coherent, and the tail has a
fuller profile. Blastoise's larger body, shell and cannon assemblies remain
readable; the close view crops the feet, while the elevated/battle views
retain the complete body. Native image evidence uses the `wartortle-turtle-`
and `blastoise-turtle-` prefixes under the ignored native-review directory.

This initial review exposed visible arm/palm junctions at the wrists, despite
occupied roots and correct skin ownership. The continuous wrist correction
and its final native review are recorded below.
Browser and moving-world visual acceptance also remain open. Shiny checks
remain skipped; grass height and renderer AA/filtering remain untouched.


### Continuous wrist correction

The close native views exposed the arm/palm junctions. Each forelimb and palm
is now one connected closed surface, replacing the separate intersecting
palm solids. The source has two fewer solids per model, reflected in the
current table above. Curved palm sections carry every original claw, and the
existing wrist carriers still control their distal regions. Forelimb normals
blend across the organic bends; shell, rim and cannon crease rules remain
unchanged. The updated wrist blend is saturated across the claw-bearing palm.

All other geometry and skin weights, the complete hierarchy, inverse binds
and clip keys match the preceding models exactly. The new independent stored
geometry check detects large planar closing fans and requires them to lie in
the shoulder plane. It rejects the previous wrist disks even with old palm
labels removed; final closed topology and occupied claw roots are also checked.
The initial test used the centroid of a partial cap, which was unreliable
after float quantization; plane incidence uses the actual stored joint and
triangles instead. All 35 Python anatomy/skin checks pass (57.445 s), and both
final Blender exports match canonical bytes exactly. Source/profile/posed
review sheets show continuous forelimbs with the claws still seated. Native
verification of this revision is recorded below; earlier receipts refer to
the preceding wrist geometry.


The four production activation/support/world-walking checks pass again
(2.09 s) on the continuous wrists. World walking preserves complete forelimb
skin weights as well as shell, claws and species-feature weights. The actual
pack heights remain 0.9906 m and 1.6002 m. Native production rebuild and
paired inspection follow below. Shiny verification remains skipped.


The complete 200-entry species schema check passes with the new geometry
counts (105.37 s). Both current canonical compressed payloads are verified
in the rebuilt native executable. Native production examples build with the
normal optimized asset profile; model size and all terrain admission rules
remain unchanged.


The next close view exposed a remaining wrist groove from linearly
interpolated radius controls, although each limb was already one closed
surface. The forelimb helper now uses the same smooth interpolation for width
and depth as for the centerline. A further close-view critique eased the
terminal centerline bends so the curved surface does not pinch at the palm. This local change retains the original claw
positions, rig, clips, body proportions and admission rules. The current table
and pins describe this smoothed revision. Both Blender exports match the
canonical bytes before the final centerline easing. That interpolation
revision passed all 35 geometry/skin checks (94.271 s) and four production
activation, support and walking checks (2.94 s). The final centerline revision
passes all 35 Python checks (107.557 s), all four focused Rust checks
(3.50 s), and exact Blender export parity. Native review follows below.


The final native first-person and elevated pairs at Route36 (20,8), noon,
preserve state against the matching 2D captures. Both revised forelimbs now
read as smooth continuous surfaces without the previous wrist ring; original
claws remain seated. Wartortle retains its feather ears and full cloud tail,
and Blastoise retains its heavy shell and open cannon rims. The close adult
view crops the feet; its elevated view retains the full supported silhouette.
These views accept the local wrist correction, with browser, moving-world
and final-revision battle visual acceptance still open. Earlier battle
receipts describe the preceding wrist geometry.

Final normal-only receipts (`*-wrist-*` under the ignored review directory):

| Model / view | Frame | State checksum | Virtual seconds |
|---|---:|---:|---:|
| Wartortle first-person | 2715 | 1671176586 | 49.350091 |
| Wartortle elevated | 1990 | 4160061106 | 34.033158833 |
| Blastoise first-person | 2774 | 3456946714 | 48.818713001 |
| Blastoise elevated | 1582 | 4183813991 | 27.220163125 |

Both current canonical compressed assets are present in the rebuilt native
executable. Physical sizes, terrain support, omission and whole-body/frustum
admission remain unchanged. Shiny checks remain skipped by user direction.
