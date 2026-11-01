# Lickitung sculpture

The current walking support packing and its revised runtime counts/pins are
recorded in [Bilateral walking support packing](world-support-packing.md).
The original logical sculpture, battle weights and clips are unchanged; the
earlier runtime packing and capture record below remains historical evidence.

The current canonical asset is `lickitung.glb`, built from
`tools/lickitung_sculpt.py` and `tools/lickitung_glb.py`. Its 33 original
source solids pack losslessly into 24 runtime primitives: 17,028 vertices,
29,256 triangles, 18 joints, 157 accessors and 758,680 bytes. This replaces
the static JSON's 61,784 vertices and 108,716 triangles, keeping the existing
20,000-vertex, 40,000-triangle, 64-part, 256-accessor and 1 MiB limits.
Normalized height remains 1.18; production physical scaling is unchanged.

Focused round sampling retains the stocky body, broad feet and white toe
tips, cream belly bands and haunch rings, small dark eyes with white glints,
physical oral recess, broad long tongue and heavy upward tail. The head keeps
its original pink palette. An adaptive closed lining follows the actual
recess. The final head has finer round sampling, and its closed oral lining
is clipped from the actual front head triangles with a 1 mm source-space
offset. This avoids raised chords across the concave bowl and the jagged
shadow fringe exposed by the first native close review. Independent rays
through the stored head prove a real depression rather than a flat painted mouth. A hollow
head or through-hole is not claimed.

Each arm is now one closed surface from its buried shoulder through the
wrist and blunt palm. The earlier separate `Hand /` solids are retired;
white nail tips stay rooted on their wrist carriers. Centerline and radius
profiles interpolate smoothly using the continuous-forelimb approach already
used by the turtles and shell bipeds.

The rig has body/neck/head motion, separate shoulder/elbow/wrist chains,
supported legs, three tail carriers and three tongue carriers. The fitted
tongue groove interpolates the actual stored, quantized tongue-triangle
weights, so its offset surface deforms with the parent. Fixed complete feet
and toe tips preserve battle floor contact. All seventeen non-root carriers
have independent motion. Idle is 3.2 seconds; attack and hit are one-second,
rotation-only source-cue clips.

The logical stored-buffer geometry pin is
`3662e8ec4d4b4c20173929a640da889ff077b0f8e57d81d6696df70736825a88`;
the packed geometry pin is
`b50f3620835bb937778fcb92300eddc4e2d729f2e8129527d82b9e20f8bb3dc0`.
Background Blender dispatch reproduces the exact current canonical bytes.

All 20 Python regressions pass in 34.833 seconds. They independently read
stored geometry, skins, inverse binds, hierarchy, animation keys and lossless
part ranges; check physical mouth depth, palette, budgets, normals, supported
floor contact, edge strain and winding; articulate every carrier; retain
occupied anatomical roots and uncrossed limbs/tongue/tail through the clips;
and verify fitted groove/haunch surfaces over the actual posed parents.
The front-cap clearance check includes vertices, edge midpoints and face
centers at the unchanged 0.0001 model-space threshold. A new standoff regression also limits the
oral lining to 0.1% of body height above the actual recess, preventing
bridged or hovering front faces. Closed markings have buried backs
and sidewalls, so exposed caps are selected from their authored surfaces,
rather than mistaking a rotated sidewall for a front cap. Large closing fans
are rejected outside the buried shoulders. All three production Rust GPU,
battle-support and two-foot walking checks pass in 0.74 seconds.

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest \
  test_lickitung_sculpt test_lickitung_glb -v
```

Both shared 205-entry registry/schema and source-cue timing regressions pass
in 70.56 and 120.88 seconds. The final native build succeeds. Preflight checks
confirm both executables support headless capture and embed this exact current
canonical compressed GLB.

All final native review uses `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`,
with 1000×700 offscreen images and no OS windows or focus changes. The Route36
close view uses ordinary wide first-person controls `0 0 20 1`; the elevated
view uses `3 2 50 0`. Each 3D image was individually inspected against its
corresponding locked 2D reference. The full Route29 battle names LICKITUNG
and SENTRET without model fallback. The broad tongue, continuous palms,
cream rings and heavy tail retain their rooted volumes and floor support.

The first native close review exposed a jagged mouth fringe from the earlier
raised cap. Finer head sampling and a tightly conforming lining substantially
reduce it. Faint mouth-rim shadow aliasing remains at close range; close
shading polish stays open. The extended tongue naturally dominates the close
portrait; the elevated, profile and battle views expose the remaining body.

The named `CRYSTAL_ENCOUNTER_TRACE=1` diagnostics confirm terrain admission
at the actual unchanged 1.1938 m pack height, derived from production scale
2.6809916, reference height 1.18 and 2.65 world units per meter. Admission
reports `accepted=true`, `reason=None`, and zero body support offset. No
size reduction, scenery change, omission or admission relaxation is used.
The current paired receipts are:

| View | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| first | 2532 | 1722144724 | 2532 | 43.603166457 |
| elevated | 1930 | 1626082511 | 1930 | 32.788047249 |
| battle | 3238 | 3268376239 | 3943 | 69.034304875 |

Browser and moving-world visual acceptance remain open. Shiny checks are
skipped, and grass height, AA and texture filtering are untouched.

## Earlier static geometry review

The following records the original static sculpture and its source review.
Current rig validation is recorded above.

The canonical `lickitung.mesh.json` uses original geometry from
`tools/lickitung_sculpt.py`: 35 named closed parts, 108,716 triangles,
grounded floor and normalized height 1.18. SHA-256:
`1e9f8516822bee309daa3163b94c32c312500ace0ebbb3070e17fe378c07d469`.

A stocky pink torso has four separate fitted cream belly bands and rooted
heavy haunches, each with a fitted cream ring and pink center. Broad feet
carry white toe tips. Rounded shoulders, broad short forelimbs, blunt palms
and small white hand tips retain actual volume roots. A broad pink tail
curves upward behind the body with a tapered end. Its initial pinched turn
was rejected visually and replaced by a smoother ascending curve.

A rounded pink head has small fitted dark eyes, white glints, subtle nostril
marks and an actual localized recess beneath a broad fitted dark oral lining.
The long pink tongue is separately modeled with a flattened section and a
shallow fitted center marking. Closed recessed geometry is used; a hollow
head or through-hole is not claimed. A reused surface helper rebuilds its
part, so the head palette is explicitly restored after that rebuild. This
corrected the first candidate's mismatched head color without changing
Cubone, Marowak or their helpers.

The first tongue looked supported in the software view but failed an actual
volume-root check against the head. Its root now extends into the head and
passes the check. A face/edge clearance check also rejected the tongue's
center marking; increasing its surface offset corrected the failure without
relaxing the clearance threshold. All ten final candidate checks passed in
35.632 seconds. They cover closed outward surfaces, actual face-aligned
normals, recipe parity, palette, height/floor/budget and anatomy counts,
head/neck/tongue/tail roots, all limb/palm/nail/haunch/foot/toe roots, fitted
vertices and face/edge clearance, uncrossed arms/tongue/tail, broad tongue
and heavy tail dimensions, and detached tongue/appendage negatives.

Final front, oblique and rear software renders each exited 0 at 800×824 and
were individually inspected after the final tongue-root and marking changes.
Canonical bytes match the reviewed candidate. The pure builder now uses the
original dedicated recipe, and its previous generic implementation rejects
Lickitung. The existing exact Rust species route is preserved. All ten installed canonical checks passed in 23.016 seconds. The fresh
native build exited 0. A battle capture with explicit
`CRYSTAL_PREVIEW_MUTE=1` exited 0 at 2000×1400 and was inspected in full:
the broad tongue, cream bands/rings, white tips and upward tail read on the
actual arena floor. Frozen renderer `(217899252, 209925056,
1791815178450414724)` (inode, bytes, modification nanoseconds) contained
the full final canonical hash and mute switch; executable identity and
source hash stayed stable through the capture.

The union-feature Rust integration exited 0, with its exact all-species
model check passing in 3.65 seconds. A direct exact rerun passed in 3.76
seconds against frozen `crystal_voxel_view-dc3731eb4a201fc3` executable
`(217922395, 234722816, 1791815375146727944)`, containing the final canonical
hash. Both executable identity and source hash stayed stable through the
rerun. This proves model loading and bounded geometry, not every species'
art. An image-target capture does not establish ordinary live-window,
browser, animation or gameplay behavior.

The [source review](lickitung-source-review.md) records official anatomy and
specific external model candidates. No external archive, mesh, texture, rig
or clip was retrieved or imported. This is original static geometry.
Software renders do not establish ordinary live-window, browser, animation
or gameplay behavior. All Lickitung verification processes are terminal. Task-owned ignored
scratch was removed after the final proofs.
