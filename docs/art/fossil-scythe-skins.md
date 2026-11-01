# Kabuto and Kabutops fossil skins

The [October 21 full-volume revision](fossil-pair-fidelity.md) supersedes
Kabutops's initial geometry, pigments, posture and counts below. Kabuto remains
byte-identical; the source and skin history is retained here.

The old primitive assemblies are replaced by original closed sculptures and
six/thirteen-joint canonical GLBs. Local normal front/back sprites inform the
silhouettes; the earlier 3D palette's red Kabuto underside eyes and ivory
scythes remain, while the packed sprite palette itself has only two browns,
white and black. This is a 3D interpretation rather than an exact palette match.
All pack-derived references stay ignored under `target/pokemon-native-review`.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Kabuto | 424 / 816 | 5,970 / 9,356 | 23 / 20 | 6 | 88 | 243,224 |
| Kabutops | 966 / 1,692 | 9,130 / 14,756 | 42 / 27 | 13 | 147 | 401,264 |

Kabuto has a broad smooth carapace, dark recessed underside, two supported
upper optic pits, two fitted ivory/red underside gazes, four bent claw stems,
four planted roots, four hooked ivory tips and a fitted rear inset. Its dome,
underside, pores and gaze share one coherent body control. Four independent
limb controls blend to the identity floor root below their upper stems; all
resting foot roots and hooked tips remain fixed.

Kabutops has a broad angular hammer shell, two swept crown blades, narrow
fitted ivory glare, a rooted neck, tapered torso, five fitted ivory rib plates,
three dorsal spurs, bent shoulders/forearms, two wrists, two broad curved ivory
scythes, rooted haunches/shins, two planted pads, six pointed talons and a
connected tapered tail. Its controls comprise ground root, torso, skull,
two shoulders, two elbows, two blade hinges, two hind limbs and two tail
controls. Crown blades and complete gaze follow the skull; ribs and back spurs
follow the body; each complete scythe follows its wrist hinge. The tail blends
between its two controls. Feet/talons remain fixed in battle, and lower hind
limbs blend up to their own controls.

Idle lasts 3.2 seconds / 81 keys. Attack and hit last one second / 65 keys,
use source-cue timing and have rotation-only tracks with neutral endpoints.
Packing combines only coherent same-material groups, retaining complete
logical ranges and unique bounded runtime names. Original 20k vertex, 40k
triangle, 64 part/joint, 256 accessor and 1 MiB limits remain intact. Authoring
heights remain 0.60 / 1.20; local pack heights remain 0.5080 / 1.2954 m.
Uniform scale, complete animated envelopes and original admission gates apply.

The initial scythe bend crossed itself. A wider bend and narrower root section
remove the actual intersection without weakening the geometry test. Source and
independently decoded pose sheets were inspected inline. The final strict
readers reconstruct all source parts and reject mutated bytes/species/geometry.

Rebuild with the direct CLI or background normal builder:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/fossil_scythe_sculpt.py \
  --output target/fossil-scythe-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/fossil-scythe-rebuild --only KABUTO,KABUTOPS
```

These two routes and the canonical GLBs produce identical bytes.
Rebuild output and packs stay ignored and must never be staged.

Seventeen checks pass against the final stored GLBs in 12.466 seconds:
nine source checks and eight independently decoded skin checks. They cover
closed connected outward solids, finite normals/winding, anatomy counts,
actual occupied attachments, parent-supported face/rib/inset vertices and
triangle centres/edges, front-visible gaze, uncrossed curved solids, detached
negative controls, exact source geometry, hierarchy/inverse binds, independently
moving controls, coherent carriers, fixed ground vertices, positive posed
winding/strain, neutral endpoints, packing/names/limits and source-byte/species
mutation rejection. Candidate checks also pass after the scythe correction.

Four focused Rust checks pass in 0.92 seconds: original pack-size GPU admission
for each species, registry/canonical identity with fixed support vertices across
all clips, and four/two complete walking supports. World motion keeps canonical
upper-body bindings, has zero/two arm chains, remains above the floor and inside
complete locomotion bounds, and keeps ground contact. The test metadata was
corrected to distinguish 23/42 source parts from 20/27 runtime parts and to use
the revised scythe's final 9,130 vertices. No geometry tolerance was weakened.
Both native review examples build in 47.47 seconds.

| Species | Source SHA256 | Packed SHA256 | GLB SHA256 |
| --- | --- | --- | --- |
| Kabuto | `fcaf686170a83249dfb2edc88f529cbdcd6f8eafb731bcc617dc44f05d6c7be7` | `46446b487607972de83a305147cd15a15e66dc7e9b15d3dda2c226c112864fa6` | `14ddb3225b91c14a4366e6b4051ea5fa8aa2b2a169f147f3f9c46fb94f6e2809` |
| Kabutops | `647542191c7253e934841d51db5bc9b6e60782440f1c39803be0434ee4fafe55` | `425d7b4409372617d60635989f3d8993b5733b4c91754397bb39c056a27ce7b6` | `1f1fa161ff757cc15ac9019eb1e647024cb792dbdad33e03b3ae0b266401f1bc` |

The shared schema/physical-size check passes in 91.57 seconds; the shared
elapsed-idle/exact-current-attack-cue check passes in 131.09 seconds. Together
with the 17 Python and four focused Rust checks, this is 23 scoped checks.
It does not establish full-suite or gameplay completion.


Six final same-state production 2D/3D pairs and two moving world frames were
inspected inline. Captures used headless image targets and mute throughout;
no window, GUI review, focus change or audible playback was requested.
Four front/world traces each contain 32 synthetic frames at five Route36 grid
positions. Both Route29 battles admit complete original-size models with
uniform scale, support zero, original full-envelope/camera/terrain gates and
no fallback. Both Pokémon names are visible in each battle HUD.

| Species / view | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| kabuto front | 1912 / 3701813612 | 1920 | 33.001153417 |
| kabuto world | 2011 / 521393683 | 2017 | 34.477807376 |
| kabuto battle | 2679 / 818281449 | 3367 | 58.200013251 |
| kabutops front | 2098 / 2111844756 | 2108 | 36.267162833 |
| kabutops world | 2027 / 550248271 | 2033 | 34.676203376 |
| kabutops battle | 2630 / 4075100723 | 3373 | 58.256754999 |

The complete receipt is ignored at
`target/pokemon-native-review/fossil-scythe-capture-receipts.json`.
Both binaries contained the exact current compressed GLBs and headless/mute
controls; streamed SHA256, size and modification time stayed identical before
and after every capture. Binary identities:

- `render_at_location`: `b50a3f68a76787af158f1d55f3b1b1392865a5606166bd37c61d0125c7f0cd20` (407,520,400 bytes).
- `immersive_battle_3d`: `db4dfb10795126e0a43badf1b84bac8a16b076d1292c47d9b076c66e2b69f144` (407,269,368 bytes).

Kabuto's dome, underside and hooked claws read well from the battle angle.
High views partly hide the underside gaze; side views hide the far-side claws
and reduce small pupil detail. Kabutops's angular gaze, rib plates, scythes,
spurs and tail remain coherent. Near foliage overlaps part of one scythe in
the front camera; exact side views foreshorten the blades and hide the far
crown/limbs. The inspected world frames retain ground contact. Brown materials
remain subdued under strong shadows, and broader browser/camera/material/scenery
coverage remains open. No shiny verification, grass-height, AA/filtering,
audio, pack publication or gameplay changes belong to this batch.

Registration reaches 247 skin variants and 28 static species files.
Full-roster model and rig improvement remains active.
