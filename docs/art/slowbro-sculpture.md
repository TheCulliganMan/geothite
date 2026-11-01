# Slowbro sculpture

The current canonical asset is `slowbro.glb`, authored by
`tools/slowbro_sculpt.py` and `tools/slow_pair_glb.py`. Its 72 named source
solids pack losslessly into 38 runtime primitives, with 19,322 vertices,
32,532 triangles, 18 joints, 220 accessors and 855,076 bytes. The former
static JSON is retired. Physical scale and the existing geometry, accessor,
primitive and byte limits are unchanged.

Each pink forelimb is now one closed surface from its buried shoulder through
the elbow, wrist and palm. The earlier overlapping round `Hand /` solids are
retired, removing their visible wrist ring. The centerline and both radius
profiles interpolate smoothly, following the continuous-forelimb approach
already used for the turtles. Eased elbow bends prevent self intersections.
All six original white hand claws remain seated in the actual pink palm
volume. Separate shoulder, elbow and wrist joints still articulate each arm.
The 70 non-arm/palm solids retain their exact stored f32 geometry, normals,
indices and colors from the previous GLB.

Both complete feet and their claws stay planted in battle. Body, neck, head,
supported legs and a two-joint tail articulate while facial features move
together. All seventeen non-root carriers have independent motion. Idle is
3.2 seconds; attack and hit are one-second rotation-only clips driven by the
production battle cues. Complete feet and lower limbs follow production
walking support without rebinding facial or held assemblies.

The whole toothed shell follows its tail attachment with a distinct lower
biting jaw. Coils, spines, upper teeth and fitted gaze stay on the shell;
lower teeth follow its jaw. Each ear carries its fitted inset and curl.
Samples in both the hero and shell mouth openings remain empty against every
posed solid.

The logical geometry SHA-256 over stored f32 buffers is
`d530ae799750a2e83b3474fc00986df88a53935c12baaf152c3916c5056905b2`. The packed geometry pin is
`33741cd0059c20dfaee09820fe9898e00eee22e21fce8e153eda4985ed38fc14`. Background Blender dispatch reproduces
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
names match SLOWBRO and PIDGEY, with no model fallback.

The named `CRYSTAL_ENCOUNTER_TRACE=1` diagnostics confirm terrain admission
at the actual unchanged 1.6002 m pack height, computed from the production
uniform scale and reference height. Admission reports `accepted=true` and
`reason=None`; body support remains at zero. No size reduction, scenery
change or admission/omission relaxation is used. Paired receipts are:

| View | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| first | 2759 | 3133142272 | 2759 | 47.256526458 |
| elevated | 2153 | 1930752588 | 2153 | 36.881305583 |
| battle | 4225 | 1654057903 | 5051 | 94.636115041 |

The earlier tight first-person setting `1 0 20 1` still crops the upper
head; the review uses the ordinary wider controls rather than changing
camera behavior or species scale. Browser and moving-world visual acceptance
remain open. Shiny checks are skipped. Grass height, AA and filtering are
untouched by this correction.

## Earlier static geometry review

The following records the earlier static asset and its original source review.
Current rig validation and production capture receipts are recorded above.

The canonical `slowbro.mesh.json` now uses original geometry from
`tools/slowbro_sculpt.py`, replacing the generic oval head, round protruding
eyes, mitten arms and ball-shaped shell. Its 74 named closed parts contain
76,932 triangles, with a grounded floor and normalized height 1.20.
The final SHA-256 is
`d41f80791ac48c331106e0ae521dcbe112b784dd916e26e79d41e806cc950d8a`.

A continuous pink head, neck and pear-shaped torso carries fitted tall white
eyes with tiny dark pupils, rounded ears with fitted inner curls, a broad
rounded cream upper lip, a separate rounded lower jaw, recessed dark oral
chamber, visible pink tongue and two small upper canines. Four fitted cream
bands follow the actual lower torso. Rounded shoulders and hanging forelimbs
end in short palms with three white claws each. Heavy thighs connect through
short ankles to two grounded feet with two white claws each.

The thick pink tail roots into a separate gray-blue Shellder creature. Its
progressively tapering shell has three slanted fitted overlapping turns,
seven rooted spines, an angular fitted white gaze, a separate lower biting
jaw, eight rooted teeth and a solid rear wall around its recessed oral
chamber. The front aperture remains open around the tail. This is a modeled
shell creature rather than a sphere with a decorative spiral.

The candidate passed fourteen checks in 47.579 seconds after the final
shoulder and rear-wall changes. Checks cover stored recipe parity, finite
closed outward surfaces, face-aligned normals, triangle budget, named anatomy,
actual occupied-volume roots for limbs/claws/ears/teeth/tail/shell, grounded
feet, fitted vertices and face/edge clearances, actual shell coil outer/inner
face occupancy, uncrossed curved limbs, twenty actual unoccupied sample
points across the hero and shell apertures, filled-mouth negatives and
detached attachment negatives. Earlier failures identified a weak canine
root and a tail obstructing a shell-mouth sample; geometry was corrected
without relaxing the checks. The tongue was raised from inside the lower lip
and the rear oral chamber covered after visual review.

All three final software views (front, oblique and rear, 800×824) rendered
successfully and were individually inspected. Installed canonical bytes
match the reviewed candidate. The dedicated pure builder dispatch preserves
the existing exact Rust Slowbro route; the legacy Blender function explicitly
rejects both Slowbro and Slowking. The
[source review](slow-pair-source-review.md) records official artwork and
specific externally licensed candidates whose archive endpoints returned
HTTP 401. No external geometry, rig, skin or animation was retrieved or
imported. This is original static geometry.

All fourteen installed canonical regressions passed in 44.908 seconds.
The matching native build finished with exit zero in 4m44s. Its full final
Slowbro hash and mute switch were verified before launching the explicit
image-target battle capture with `CRYSTAL_PREVIEW_MUTE=1`. The capture exited
zero; its full 2000×1400 GPU frame was inspected for the visible anatomy,
floor contact, arena and HUD. Executable identity
`(217163579, 200875920, 1791804654471009916)` and canonical hash stayed stable
through the launch.

The all-251 Cargo job finished with exit zero in 6m27s and ran one roster
test, passing in 1.92 seconds. A direct exact check then passed in 2.52
seconds with the full final Slowbro hash verified and stable executable
identity `(217174951, 216558656, 1791804819123248078)` and unchanged canonical
bytes. Both completed executables lacked the newer Slowking hash, so these
checks do not verify that replacement. The native image target and roster
loading pass for Slowbro; live-window, browser, animation and gameplay paths
remain unapproved. The later exact all-251 check passed in 1.39 seconds
with both final species hashes verified in stable executable identity
`(217198810, 218441024, 1791805361793194901)` and unchanged canonical bytes.
Slowking also passed its own final-hash muted GPU capture. Completed shared
temporary review images and logs have been removed.
