# Electrode coherent orb skin

Electrode now uses the existing Voltorb orb authoring/export path and a
canonical two-joint GLB. Its normal red, white and ink match the local content
pack's RGB555 palette. The angled brows are thicker, pupils are longer and
closer to the centre, and the broad curved grin retains two short tooth seams.
The front/back pack sprites and source/decoded pose sheets are reviewed only
through ignored images; no game content is copied into tracked source.

The closed upper and lower hemispheres retain identical equator vertices.
The body carries both equators and the entire fitted expression. The identity
root fixes the lower support ring; lower-shell weights blend to the body over
1.5–15% of authoring height. Nothing adds feet, hover, or a second face rig.
Idle is 3.4 seconds / 85 keys. Attack and hit are one second / 65 keys with
early source-cue emphasis, rotation-only tracks and neutral endpoints.

Geometry remains 17,210 vertices / 23,480 triangles / ten parts, with two
joints, 48 accessors and 675,972 bytes. All existing reader and admission
limits remain intact. Original authoring height is 0.94; the existing local
pack physical height is 1.1938 m. Uniform scale and full animated bounds are
preserved. Voltorb's stored GLB remains byte-for-byte unchanged.

The normal background builder, direct exporter CLI and canonical Electrode
GLB match exactly. Both strict Python model readers and Rust registry/GPU/world
paths use that asset. Source digest:
`cc8828f83e653417b3a084b8e0ca908deb1f3a5fe4acfea67e831e87050e7571`.
GLB SHA256:
`cf3357ebe9c1d740ad000a6589792040c3ad01ae1b4540d031646325c6994315`.

Rebuild without opening a window:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/voltorb_glb.py --species electrode
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/electrode-rebuild --only ELECTRODE
```

Generated rebuild output is temporary and ignored. Do not stage it or packs.

Eighteen stored Python checks pass in 133.497 seconds across
`test_electrode_glb`, `test_voltorb_glb` and `test_orb_sculpt`: exact canonical
geometry, palette, closed outward solids/normals, actual-parent face fitting,
shared posed equators, fixed support vertices, floor clearance, positive posed
winding, bounded strain, moving controls, source cues and neutral endpoints.
The support-ring assertion requires at least 64 vertices; Electrode has exactly
64 below 0.001 authoring units. The initial copied strict-greater assertion was
corrected without changing geometry or floor tolerances.

Three focused Rust `electrode_skin` checks pass in 0.78 seconds. They prove
registry/canonical identity, fixed shell contacts across all clips, original
pack-size GPU admission, and unchanged canonical bindings during legless world
locomotion. All world vertices stay above the floor and inside full locomotion
bounds. Shared schema/physical-size and elapsed-idle/exact-attack-cue checks
pass in 119.96 and 160.17 seconds. This is 23 scoped checks, not a full-suite or
gameplay-completion claim. Both native examples build successfully.

Three same-state production 2D/3D pairs and one moving world frame were viewed
inline. Every launch used headless image capture and mute; no visible windows,
focus changes or GUI review were used. Front/world movement traces each contain
32 synthetic frames over five Route36 positions. Battle admits the complete
1.1938 m model with uniform scale, ground support zero, original camera/gates
and no fallback. Both Pokémon names are visible in the battle HUD.

| View | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Route36 front | 3043 / 2131856280 | 3059 | 54.715225293 |
| Route36 world | 2724 / 1591836743 | 2733 | 47.069318333 |
| Route29 battle | 2428 / 2490216498 | 3094 | 53.355667542 |

Ignored evidence is under `target/pokemon-native-review/electrode-*`, including
`electrode-capture-receipts.json` with complete status. Both binaries contained
the exact compressed current GLB and headless/mute controls; streamed hashes,
sizes and modification times stayed identical before and after all captures.
The render binary SHA256 is
`8874cd1538dbcf21b722205683854cec7ec995cfa8bcb011ab6e4fe59c95945f`;
the immersive battle binary SHA256 is
`6e5358b9ce8fe2357ee3d2bdca51ebd7360e2a719b7163258f82f7765f0abd29`.

The round silhouette, white/red equator and expression read clearly from the
front. High views foreshorten the lower-hemisphere grin; side views hide the
far eye and most of the grin. Small tooth seams remain subtle under production
lighting. The inspected world frames retain ground contact, and the battle
sphere clears the grass. Broader browser, camera, material and scenery coverage
remains open. No shiny verification, grass-height, AA/filtering, audio, pack
publication or gameplay changes belong to this batch. Registration reaches
245 skin variants and 30 static species files; full-roster improvement remains
active.
