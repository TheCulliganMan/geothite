# Nidorina and Nidorino sculptures and skins

The original hornless blue Nidorina and horned purple Nidorino sculptures now
use standard canonical GLB skins. `tools/nido_middle_glb.py` exports the two
existing species recipes through the normal builder, strict reader and Rust
registry routes. Retired static JSON assets are removed.

| Species | Logical solids | Runtime primitives | Vertices | Triangles | Joints | Accessors | GLB bytes | Reference height |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Nidorina | 59 | 35 | 19,478 | 32,064 | 15 | 201 | 808,672 | 1.12 |
| Nidorino | 74 | 39 | 19,860 | 32,172 | 13 | 189 | 748,348 | 1.12 |

The retired static versions used 56,828/30,014 vertices and 93,956/48,108
triangles respectively. Redundant planar-ear subdivision is removed while
retaining the exact authored notches. Facial, belly and spotted markings clip
actual stored parent triangles, with parent-derived cap normals and closed
outward walls. Nidorino's four rounded quarters use 28 radial/20 vertical
samples instead of 36/24. The resulting vertex reductions are about 66% and
34%, within the unchanged 20,000-vertex, 40,000-triangle, 64-runtime-part,
64-joint, 256-accessor and 1 MiB limits. The original normalized heights and
physical-size path remain unchanged.

Nidorina retains a broad hornless head, low muzzle, orange eyes, thick notched
navy-lined ears, pale rounded breast, bent forelimbs, spotted hindquarters,
rooted dorsal barbs, short tail, two hind paws and twelve hand/foot claws.
Its 15-joint skin has body/head articulation, independent pinnae, three
carriers per forelimb (upper arm, forearm, wrist), paired haunches and two
rearward tail carriers. Complete hind paws and their six claws remain fixed
in battle. Hands and their six claws follow the corresponding wrist. Haunch
spots follow their own haunch. World locomotion now treats this authored
biped as two-legged, matching the two planted supports and elevated hands.

Nidorino retains the low long purple body, lowered wedge head and jaw,
forward nasal horn, tall ragged teal-lined ears, head/flank/leg spots, four
rooted quarters and paws, twelve toes, dorsal barbs and short tail. Its
13-joint skin has body/head, two pinnae, a jaw, a horn, four supported limb
fields and two rearward tail carriers. Complete paws and their twelve claws
remain fixed in battle. Leg spots occupy rigid middle-leg regions and follow
their corresponding limb. World gait retains four complete leg trees.

Both have identity floor roots, coherent fitted facial layers, monotone
rearward tail weights, neutral clip endpoints and rotation-only channels.
Idle lasts 3.2 seconds; attack and hit last one second. Production source cues
continue to own timing. Support markings and claw groups stay separate per
limb. Runtime packing preserves every logical solid's stored buffers and
source order. No external geometry, rig, clip or reference image is imported.

Hitmonlee imports Nidorino's existing side-patch helper; that helper retains
its original behavior while the new Nidorino recipe uses a local fitted-triangle
helper. Hitmonlee and both small Nidoran exports remain byte-identical. Public
attachment helpers borrowed by other model tests retain their original contract;
the new species checks use a separate occupied-interval helper at the original
0.0002 attachment threshold and the same 24×24 interior grid at i/25, j/25.

Current logical stored-buffer geometry pins:

- nidorina: `cfc3978374144332ac6db5a703ee33ac0252cd51ebb33db26e4603f4b48594ae`.
- nidorino: `1443837cd4fc174eb9f440a9a329b4533758245eb5ec3099d576fcd26530a505`.

Current packed geometry pins:

- nidorina: `85c4a8e56049919dc4ca944a15f2711111399da8b3542cf1d4e38fffdd3f9b5e`.
- nidorino: `6a11581fdc1fd23d94214c2430bf1c33f8bd92bccc9e94e7abbffdf8f9ccd688`.

Current full GLB SHA-256:

- nidorina: `52133a70bc4dc7f5f49161ff38f86a3b76421aaf794f8007a0e68bca0b8f199a`.
- nidorino: `6add2393ec3cf4f2dd8d96c616aff5949d58eef7dc1ef0e5e5c45792660f4ce6`.

All 25 Python anatomy/skin checks pass in 35.435 seconds. They inspect stored
closed/outward topology and shading, exact recipe/hierarchy/buffers, inverse
binds, occupied attachment roots, fitted caps against actual posed parents,
independent carriers, uncrossed appendages, floor contact, winding, strain,
neutral endpoints, limits and strict mutation rejection. Four Rust checks
pass in 1.80 seconds: original-size GPU activation for each species, battle
support/bounds and independent carriers, and world gait with correct two/four
support counts and per-leg markings/claws. Background Blender reproduces both
canonical GLBs exactly. Front/oblique/rear software comparisons and decoded
idle/attack/hit pose sheets were inspected offscreen.

The shared registry check passes in 82.87 seconds and production source-cue/
GPU timing passes in 78.00 seconds, for 31 targeted checks across this batch.
Native examples rebuild in 1 minute 4 seconds with `crystal-assets` at
optimization level 2. Every launch explicitly uses headless image capture,
preview mute and normal coloration. Preflight requires the headless parse
guard, mute support and exact current gzip payload for both canonical GLBs
in each executable. Executable identities stay unchanged after all six runs.
No window, tab or application activation is used.

All six selected 3D/2D pairs and both production walking frames were inspected
inline. Frontal views use Route36 `(20, 8)`, hour 12, camera `0 0 20 1`.
Walking views use the same start, camera `5 2 30 0` and ordinary Down input
for 32 review ticks. Both traces stay on Route36 and change authoritative
position. Moving screenshots are requested at tick 12; asynchronous readback
supplements the independent per-leg/64-phase numerical checks rather than
providing exact-frame 2D animation comparisons. Endpoint pairs freeze the same
authoritative state across 3D and 2D.

The hornless blue head, orange gaze, pale breast and rooted hand/foot claws
remain readable on Nidorina. Its spotted haunches and short tail retain their
outline during the two-legged walking pose. Nidorino retains the low purple
barrel, forward horn, teal-lined ragged ears, head/flank/leg markings and four
complete clawed paws. Frontal, profile and battle views retain the original
scale, matte materials and ground contact. The smaller far limbs are partially
hidden in profile; the per-leg numerical checks cover both sides and both rows.
Broader moving-world, follower-spacing and browser acceptance remain open.

| Pair | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| nidorina frontal | 1994 | 1745396327 | 1994 | 33.897588625 |
| nidorina world | 2212 | 3309294478 | 2221 | 37.778299458 |
| nidorina battle | 2473 | 4249971725 | 3091 | 52.75879625 |
| nidorino frontal | 1767 | 1933895906 | 1767 | 30.019250249 |
| nidorino world | 1874 | 2729146443 | 1882 | 32.077018 |
| nidorino battle | 1809 | 3717436609 | 2455 | 41.848481209 |

| World trace | Review ticks | Column | Visited rows | Tick-12 interpolated center Y |
| --- | --- | --- | --- | --- |
| nidorina | 32 | 20 | 8–12 | -36.166 |
| nidorino | 32 | 20 | 8–12 | -39.680 |

Ordinary Route29 walking encounters retain named modeled actors with
`fallback=None`, zero support offset and final `accepted=true reason=None`
at the original external-pack heights. Whole-body support, admission and
omission gates remain unchanged. Stage counters are candidate observations,
not support percentages.

| Player / opponent | Pack height (m) | Uniform scale | Reference height | Support Y | Stage checked / unsupported |
| --- | --- | --- | --- | --- | --- |
| nidorina / SENTRET | 0.7874 | 1.8630447 | 1.12 | 0.0 | 50 / 1 |
| nidorino / RATTATA | 0.8890 | 2.1034374 | 1.12 | 0.0 | 80 / 12 |

Executable SHA-256:

- render_at_location: `d87b47a989e76af6647fdc5b10a11c8acda8b77c86f5c61268707a57a44b8894`.
- immersive_battle_3d: `08622535365d8bd32357161d2c1288e965668753ebb7e9264fa8e4e3f0be1513`.

The ignored `target/pokemon-native-review/nido-middle-` evidence prefix retains
final software/native PNGs, paired-state receipts, world movement CSVs,
build/test logs, coupling checks and executable identities. Temporary source
backups, old JSON copies, export duplicates and helper scripts are removed.
Shiny checks are skipped; grass, AA and filtering are unchanged.
