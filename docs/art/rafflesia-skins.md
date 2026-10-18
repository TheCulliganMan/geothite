# Gloom and Vileplume botanical sculptures and skins

The old generic primitive assemblies are replaced by original botanical
sculptures. Gloom has a round indigo bulb, short rooted limbs, fitted closed
sleepy lids, broad orange lip, narrow dark opening and two connected cream
drool strands. A smooth four-lobed brown-orange bud carries fitted pale
mottling above four independent drooping orange sepals.

Vileplume has fitted red bead eyes, a tiny dark mouth, short root feet and five
broad fleshy scarlet petals with fitted ivory mottling. The center is a real
hollow flared cup, pale annular throat and recessed dark nectar floor. The
flower is tilted toward the front by 0.27 radians; its lower support enters
the body rather than standing above it on an exposed stalk. Rooted petals
overlap the central cup. Outer sections taper to small rounded ends instead
of exposing broad flat cross-sections. Low angles still hide the far lobes
and throat; high views hide the face beneath the crown.

| Species | Vertices / triangles | Source / runtime parts | Joints | Accessors | GLB bytes | Authoring height / pack metres |
| --- | --- | --- | --- | --- | --- | --- |
| Gloom | 9,063 / 13,848 | 30 / 18 | 11 | 111 | 404,336 | 0.85 / 0.7874 |
| Vileplume | 8,644 / 12,508 | 34 / 23 | 12 | 129 | 360,920 | 1.12 / 1.1938 |

Previous meshes had 1,408/2,576 and 2,106/4,092 vertices/triangles. The original
20k vertex, 40k triangle, 64 runtime part/joint, 256 accessor and 1 MiB limits
remain intact. Coherent packing groups only same-material gaze and mottling
with a common rigid owner, retaining every logical source range and unique
bounded runtime names. Original authored heights, uniform physical scaling,
full motion envelopes and admission gates remain intact.

Both rigs have floor root, body, crown, two arms and two leg controls. Gloom
adds four independent sepal hinges; Vileplume adds five independent petal
hinges. Root feet stay fixed, lower body/leg weights blend out of ground
support, and arm roots blend from the body to each arm. Face and drool share
the body carrier. Bud and bud mottling share the crown carrier; every scarlet
petal and its spots share their own carrier. Cup, annular rim, dark floor and
stem move together. Existing world gait adds two complete walking support
chains, retains the two arm controls and preserves all upper-body bindings.

Idle lasts 3.4 seconds/85 keys; attack/hit last one second/65 keys, with
rotation-only tracks, neutral endpoints and source-cue ownership. Restrained
body yaw, crown response, arm movement and delayed radial petal/sepal bends
retain occupied attachments. There is no new bounce or hover offset.

The external pack's normal front/back sprites were inspected. The new
silhouettes follow the sleepy orange bud and broad red five-petal flower;
the 3D indigo/orange/red/cream materials are not exact palette parity. No shiny
verification belongs to this batch. Ignored normal references, old/new source
views and independently decoded attack poses are under
`target/pokemon-native-review/rafflesia-*`.

Both direct CLI and normal background builder match canonical GLBs exactly.
Strict Python readers/checker and Rust registry/GPU/world dispatch use the
same assets. Rebuild only into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/rafflesia_sculpt.py \
  --output target/rafflesia-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/rafflesia-rebuild --only GLOOM,VILEPLUME
```

Nine source checks cover closed connected outward
solids, finite normals/winding, original heights/floor and budgets, distinct
anatomy, occupied roots and detached negative controls, fitted surface vertices
and cap centers/edges, a genuinely open flower axis, and uncrossed curved
limbs/bud/petals/drool. Inverse review transforms return markings to their
authoring projection before testing clearance. Full geometric normals classify
front caps, avoiding rotated/rounded closing-wall noise; clearance tolerances
are unchanged.

Seven independently decoded stored-skin checks cover:
source/packing identity, hierarchy/inverse binds, every independent control,
coherent fitted carriers, fixed feet, positive posed winding/bounded strain,
neutral endpoints/source cues, occupied posed roots, uncrossed posed anatomy,
supported posed markings/open throat, and mutated source/bytes/species
rejection. Four final focused Rust checks pass in 1.61 seconds: two GPU admissions,
canonical registry/fixed supports across every clip, and two-support world
locomotion preserving upper bindings. World vertices stay above the floor and
within full locomotion bounds; contact stays within 0.005 authoring units.
The final 16 Python checks pass together in 58.858 seconds after rounding
the exposed outer petal ends. Gloom stays byte-identical; Vileplume retains
exact joint nodes and every clip time/quaternion, with unchanged heights,
vertex/triangle/part/accessor counts. Builder, direct CLI and canonical bytes
match again. These are scoped checks, not a full-suite or gameplay-completion
claim.
Shared anatomy/order/physical-size and exact elapsed-idle/current-attack-cue
checks pass in 105.20 and 175.59 seconds, making 22 scoped checks overall.
The shared checks precede the tip-only refinement, whose joint nodes and
clip keys remain exact. Initial native clients build in 57.90 seconds; the corrected-tip rebuild
passes in 1 minute 35 seconds.

| Species | Source SHA256 | Packed SHA256 | GLB SHA256 |
| --- | --- | --- | --- |
| Gloom | `ac149d788c449f8fda299e25d45cb7fd522d22d12b9c704bc37a32ff9b465025` | `1d5f919f3795501e874632168a05d657bb5bddab837adffb6c017924a16cdcad` | `f5f7b28cb7f7d3de218681d238217224f1fda5b4bf9d7ddf0952631b026bfe95` |
| Vileplume | `ef7abbebcd46edf5ed368ab5843546107efe9b41c070f2e3b6bcbac68462e16a` | `0df615733458f9a0fa2a45bce3bf68a390fd8c8da2cb0fcd329a9138411acb3c` | `dfc4085b4c60b383a9abf70213c1ac850de27e9e4c5d9a73c11b8435bdc200e2` |

Full-roster model and rig improvement remains active.

Six final same-state production 2D/3D pairs and two moving world frames were
inspected inline. Every capture used headless image targets and mute, with no
window, app switch or focus change. Gloom retains its original passing capture
group because its GLB is byte-identical after the tip-only correction. The
corrected Vileplume has a separate final group; the older Vileplume images are
superseded. Four front/world traces each contain 32 synthetic frames across
five Route36 positions. Both Route29 battles admit complete models at their
original 0.7874/1.1938 m sizes with uniform scale, zero support, unchanged
full-envelope/camera/terrain gates and no fallback. Both Pokémon names appear
in each battle HUD.

| Species / view | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| gloom front | 3016 / 515011569 | 3030 | 54.815530167 |
| gloom world | 2443 / 4266423177 | 2457 | 43.910353791 |
| gloom battle | 3099 / 3800535935 | 3817 | 74.912543877 |
| vileplume front | 3919 / 3615645277 | 3934 | 87.006360961 |
| vileplume world | 3532 / 1039717569 | 3540 | 67.978833706 |
| vileplume battle | 3569 / 1859672207 | 4277 | 82.060764793 |

Complete ignored receipts: `rafflesia-capture-receipts.json` for Gloom and
`rafflesia-rounded-capture-receipts.json` for corrected Vileplume, both under
`target/pokemon-native-review/`. Each group verifies its reviewed species’
exact compressed current GLB and headless/mute controls in both binaries.
Streamed SHA256, size and modification time stay unchanged within each group.
These are separate builds/scenery presentations, not pixel-identical
before/after frames. Binary identities:

- rafflesia / `render_at_location`: `9a52946aaa91394d69b547158d706c97e3d524c0f32ad502b7774f47965fe715` (407,984,272 bytes; mtime_ns 1792332967129841844).
- rafflesia / `immersive_battle_3d`: `9a544da9d6e9adaa3c1e8cfa91cbf8598e172f5f4109ec203c50e8a55e6d17a2` (407,733,208 bytes; mtime_ns 1792332965829457407).
- rafflesia-rounded / `render_at_location`: `ce778241315ff8ebd1256a7e2773d7c649a9fefe0b7cc22883ffd98541335799` (407,993,472 bytes; mtime_ns 1792334609220377092).
- rafflesia-rounded / `immersive_battle_3d`: `be204e1ed2f97bc0c496830b061d549fa95b53032a847dd27cc95d6fa0563551` (407,742,424 bytes; mtime_ns 1792334610793084790).

Gloom’s sleeping lids, lip and connected drool read in battle, with planted
feet and a distinct mottled bud. High/side views hide much of the face and far
sepals; tiny mottling fades at world distance. Corrected Vileplume’s close and
battle views show tapered petal ends without the broad flat cuts. Its side
battle view exposes a red eye and mouth; high views hide its face beneath the
crown, and low views hide much of the flower throat. The pale rim is partly
occluded by petal roots.

The final default Vileplume world view is substantially obstructed by nearby
foliage. Its moving frame exposes the model through existing dither fading.
That world-visibility issue remains open; this is not full visual acceptance.
Broader camera/browser/material/scenery coverage and palette parity also
remain open. No grass-height, AA/filtering, audio, pack publication or gameplay
changes belong to this batch. Registration reaches 254 skin variants and 21
static species files. Full-roster model and rig improvement remains active.
