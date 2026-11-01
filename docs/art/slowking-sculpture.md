# Slowking sculpture

The current canonical asset is `slowking.glb`, authored by
`tools/slowking_sculpt.py` and `tools/slow_pair_glb.py`. Its 60 named source
solids pack losslessly into 33 runtime primitives, with 18,831 vertices,
31,512 triangles, 18 joints, 199 accessors and 829,852 bytes. The former
static JSON is retired. Physical scale and the existing geometry, accessor,
primitive and byte limits are unchanged.

Each pink forelimb is now one closed surface from its buried shoulder through
the elbow, wrist and palm. The earlier overlapping round `Hand /` solids are
retired, removing their visible wrist ring. The centerline and both radius
profiles interpolate smoothly, following the continuous-forelimb approach
already used for the turtles. Eased elbow bends prevent self intersections.
All six original white hand claws remain seated in the actual pink palm
volume. Separate shoulder, elbow and wrist joints still articulate each arm.
The 58 non-arm/palm solids retain their exact stored f32 geometry, normals,
indices and colors from the previous GLB.

Both complete feet and their claws stay planted in battle. Body, neck, head,
supported legs and a two-joint tail articulate while facial features move
together. All seventeen non-root carriers have independent motion. Idle is
3.2 seconds; attack and hit are one-second rotation-only clips driven by the
production battle cues. Complete feet and lower limbs follow production
walking support without rebinding facial or held assemblies.

The crown, brim, horns, crest, turns and complete ruby setting move together
on the head. Three independently moving collar sectors preserve each closed
red or ivory panel and its buried neck attachment. The muzzle remains closed;
no open oral aperture is claimed.

The logical geometry SHA-256 over stored f32 buffers is
`207d131b852e541371aee7b2b863fc9142487a0e5b60a64a86c591a8d34d81d2`. The packed geometry pin is
`c66499f5fae82d1d849da07acc4b7d5e48ce75270cac872bdee5681bb1f5320b`. Background Blender dispatch reproduces
both canonical GLBs byte for byte.

All 34 final Python regressions pass in 199.811 seconds. They independently
read stored buffers and exercise hierarchy, inverse binds, lossless packing,
budgets, supported floor contact, edge strain and winding through the source
clips. Actual occupied-volume roots include every claw; the continuous arms
remain uncrossed through sampled poses. A stored-surface regression rejects
large closing fans anywhere outside the buried shoulder. Existing fitted
layers, shell coil/crown turn inner and outer faces, mouth voids and detached
attachment negatives remain covered. All four production Rust GPU, battle
support and two-foot walking checks pass in 3.15 seconds.

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest \
  test_slowbro_sculpt test_slowking_sculpt test_slow_pair_glb -v
```

The shared 204-entry model-schema and source-cue timing regressions pass in
108.78 and 148.11 seconds. Native examples build successfully. Preflight checks
confirm both executables support headless capture and embed both exact
current canonical compressed GLBs.

All six production captures render directly to 1000×700 offscreen images with
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`
and `CRYSTAL_PREVIEW_MUTE=1`. No OS windows or focus changes are used. The
Route36 portrait uses ordinary wide first-person controls `0 0 -10 1`;
the elevated view uses `3 2 50 0`. Each was individually inspected against
the corresponding locked 2D image. The portrait shows continuous smooth
wrists and readable facial anatomy; the elevated and full Route29 battle
views retain the species' silhouette and rooted appendages. The battle HUD
names match SLOWKING and PIDGEY, with no model fallback.

The named `CRYSTAL_ENCOUNTER_TRACE=1` diagnostics confirm terrain admission
at the actual unchanged 2.0066 m pack height, computed from the production
uniform scale and reference height. Admission reports `accepted=true` and
`reason=None`; body support remains at zero. No size reduction, scenery
change or admission/omission relaxation is used. Paired receipts are:

| View | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| first | 2408 | 3182260831 | 2408 | 41.69839325 |
| elevated | 1931 | 93659353 | 1931 | 32.768660166 |
| battle | 3393 | 1880318339 | 4145 | 73.546807124 |

The earlier tight first-person setting `1 0 20 1` still crops the upper
head; the review uses the ordinary wider controls rather than changing
camera behavior or species scale. Browser and moving-world visual acceptance
remain open. Shiny checks are skipped. Grass height, AA and filtering are
untouched by this correction.

## Earlier static geometry review

The following records the earlier static asset and its original source review.
Current rig validation and production capture receipts are recorded above.

The canonical `slowking.mesh.json` now uses original geometry from
`tools/slowking_sculpt.py`. Its 62 named closed parts contain 65,164 triangles,
with a grounded floor and normalized height 1.25. The final SHA-256 is
`912f4d45bcfecffb281acb335c0b9375b9f44962f7e011ad0baf371e0019984d`.

The original royal biped has a slender pink torso and neck, rounded rooted
head, fitted tall white eyes with tiny black pupils, broad cream muzzle,
subtle fitted nostrils and curved smile seam, and two small rooted canines.
Three cream lower-abdomen bands follow the actual torso surface. Short bent
arms with rounded shoulders and palms carry three white claws per hand.
Round thighs connect through short ankles to two grounded feet with two
white claws each. A thick pink tail tapers into an upturned pointed tip.
The muzzle is closed with a fitted smile marking; no open oral aperture is
claimed for this species.

Ten alternating red/ivory closed collar panels flare from buried neck roots.
Their curved sectors replace flat triangular leaves. A separate gray shell
crown roots into the head and carries a broad pale brim, two curved lateral
horns, two slanted fitted overlapping turns and five irregular upper crest
points. Its round red jewel sits within a fitted pale setting and carries a
small supported ivory glint. This replaces the baseline oval crown and two
straight cones.

The candidate passed twelve checks in 10.460 seconds; the installed canonical
model passed the same twelve in 8.979 seconds. Checks cover stored recipe
parity, closed outward finite solids, face-aligned normals, height/floor and
triangle budget, named anatomy, actual occupied-volume attachment roots for
collar/limbs/claws/teeth/tail/crown/horns/crest/turns, both grounded feet,
fitted vertices and face/edge clearances, actual shell-turn outer/inner face
occupancy, uncrossed curved anatomy and detached attachment negatives.
The first collar failed the normal check on its thin edges; separating edge
normals with the existing crease helper fixed the shading without relaxing
the check. Final front, oblique and rear software views all rendered at
800×824 and were individually inspected after this correction.

Canonical bytes match the reviewed candidate. The dedicated pure builder
preserves the existing exact Rust species route, and the retired generic
function rejects both Slowbro and Slowking. The
[source review](slow-pair-source-review.md) records official artwork and
specific licensed external candidates whose archive endpoints returned HTTP
401. No external mesh, rig, texture or animation was retrieved or imported.
This is original static geometry.

The matching native build finished with exit zero in 3m46s. The executable
contained the full final Slowking hash and the mute switch before launch.
The explicit image-target battle capture used `CRYSTAL_PREVIEW_MUTE=1`,
exited zero, and its full 2000×1400 GPU frame was individually inspected for
crown/collar/face/limb appearance, floor contact, arena and HUD. Executable
identity `(217192507, 201817104, 1791805153510236568)` and canonical hash
remained stable through the capture.

The earlier Slowbro test executable lacked the newer Slowking hash and was
excluded. The fresh Cargo all-251 job finished with exit zero in 7m20s,
running one roster test and passing in 1.85 seconds. A direct exact check
then passed in 1.39 seconds in an executable containing both final Slowbro
and Slowking hashes, with stable identity
`(217198810, 218441024, 1791805361793194901)` and unchanged canonical bytes.
Native image-target rendering and roster loading now pass for both species;
live-window, browser, animation and gameplay paths remain unapproved.
Completed shared temporary review images and logs have been removed.
