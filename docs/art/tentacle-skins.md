# Tentacool and Tentacruel: fitted bells and grounded tentacle skins

Both old static assemblies are replaced by continuous blue bells with rounded
crowns, flared rims, recessed dark mantles and fitted angular ivory eyes.
Large red lobes follow the actual bell surfaces and extend into their volume;
their white highlights follow the stored lobe meshes. Tentacool has two long
curled blue tendrils. Tentacruel has a wider bell, larger red lobes, ten visible
radial tendrils and two swept blue side feelers. The visible tendrils represent
the illustrated silhouette, without claiming to model the Pokédex's full count.

The local external pack's normal front sprites were inspected directly.
Bell/tentacle blue is RGB555 (11,20,31), Tentacool pink is (30,6,10), and
Tentacruel red is (26,2,2). These materials are checked against the exact
expanded, linearized palette values. Original pack dimensions are retained:
Tentacool `heightDigits=211`, 0.889 m; Tentacruel `heightDigits=503`, 1.6002 m.
Both authoring heights remain 1.13; production uses uniform physical scaling.
Camera, collision, full animated bounds and admission gates are preserved.

`tools/tentacle_sculpt.py` authors the anatomy and
`tools/tentacle_glb.py` exports the canonical standard skins. The shared reader,
background builder, bounded Rust registry and world/battle GPU renderers all
consume those GLBs; no duplicate static JSON remains. Direct sculpture CLI,
background builder and canonical assets match byte for byte.

Tentacool has six joints: root, bell and two controls per tendril. Tentacruel
has 24: root, bell, twenty tendril controls and two feeler controls. Upper
attachments blend into the bell, each tendril flexes independently, and distal
supports blend into the fixed floor root. Complete mantle faces and bell lobes
stay coherent with the bell. Idle uses phase differences; attack and hit use
source-relative cues. All clips rotate only and have neutral endpoints.

The old generic Float motion added 16 percent of body height as a root lift;
world initialization also required four percent minimum clearance. These
explicit lifts are removed only for Tentacool and Tentacruel, whose new
sculptures rest on tendrils. Their swimming classification is retained. Both
battle clips and composed world motion preserve real tip contact. Intentional
hovering for other species and Lapras's seated battle pose remain covered by
passing regressions.

| Asset | Vertices | Triangles | Parts | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Tentacool | 4,519 | 7,540 | 13 | 6 | 75 | 210,828 |
| Tentacruel | 7,641 | 12,260 | 23 | 24 | 175 | 416,700 |

All existing budgets remain unchanged. Geometry SHA256 values are
`acb31c9286cd89fc4218aec6bf9706b51fad8a1d0a967ac0e1a158be4b9a1798`
(Tentacool) and
`c21b730c9a3ea4b05d3a7031fff2c30ace66889c6948516b36fe2698a225117b`
(Tentacruel). GLB SHA256 values are
`159af4aea8365d491292e3f359f73abeb0c0cd738f949b045bbd9344470ce5e6`
and `70890f1c044d196dff86325bc81d70602e326da1ff18d049ab9cbbacc0361aed`.

Fifteen independent Python checks pass in 13.302 seconds. They cover canonical
geometry and existing limits; closed outward parts and unit normals; species
proportions/counts; actual volume roots for tendrils and lobes; fitted gaze and
highlights; actual eye visibility against the bell at three camera angles with
a recessed-gaze negative control; visible pupil area against bell, mantle and
eye surfaces at three angles; normal pack palette; hierarchy and inverse binds; independent
controls; pinned tips, positive posed winding and bounded strain through every
clip; neutral endpoints/source cues; and mutated geometry/bytes/species
rejection. Four focused Rust checks pass in 0.98 seconds for canonical geometry,
original-size GPU admission, battle tip contact and unchanged world bindings,
swimming classification and full bounds. Ten related magnet/gas/bivalve/Lapras
checks pass in 3.30 seconds. Native capture clients build in 70 seconds.
Shared schema and exact elapsed-idle/current-attack-cue checks pass in
126.28/126.07 seconds, bringing the batch to 31 scoped checks. These checks
do not claim full-suite or gameplay completion.

The decoded before/after, front/rear/side and attack sheet was inspected inline.
Initial production close views exposed upper-gaze occlusion by the bell, which
made the white eyes read as teeth. The mantle's front surface is expanded
forward while retaining its rear tendril roots; the gaze is raised and pupils
widened so they remain readable in the close production camera. Every joint, inverse bind,
animation key and cue metadata remains unchanged by this refinement. Initial
captures are diagnostic evidence, not final visual acceptance. Final front pairs at EcruteakCity (20,21) pass visual review in
54.943/43.739 seconds for Tentacool/Tentacruel. Both show readable black pupils
in the fitted ivory gaze, complete models and real tendril contact. Final
world and battle pairs completed in 33.797/76.220 seconds for Tentacool and
52.193/96.679 seconds for Tentacruel. All six final pairs and both moving world
frames were inspected inline. City front/world views clearly show the models
at original size and grounded tendrils; Route29 battles show the full models
without fallback and readable TENTACOOL/TENTACRUEL and PIDGEY HUD names. Each
pair preserves the same authoritative game state between 2D and 3D. The two
completed capture groups use exactly the same binaries, with SHA256, size and
modification time unchanged throughout and independently audited afterward.
The exact current compressed GLB is verified in both executables. Binary
SHA256 values are
`c365a03520839c0810d1d9b8ce3b03127bc2d927385c3e6f6a73f9d5a44c3771`
(render_at_location) and
`4932732d9ebae9c11b785db185967b4c18242fe204d7a107693ea292b7c1c1a5`
(immersive_battle_3d). Broader scene, palette and full-roster acceptance remain
open. All rendering is headless, offscreen and
muted. No shiny checks, grass-height, scenery, AA, audio, pack or gameplay
changes belong to this batch. Full-roster work remains active.

Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/tentacle_sculpt.py \
  --output target/tentacle-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/tentacle-rebuild --only TENTACOOL,TENTACRUEL
```
