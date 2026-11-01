# Meowth and Persian: feline bodies, coherent faces and curled tails

The old sphere assemblies are replaced by original continuous bodies and fitted
cat faces. Meowth has a slender upright torso, large white lenses and vertical
black pupils, tall engraved forehead coin, two pointed dark ears, six rooted
whiskers, a smile with two small fangs, two beckoning formed palms, broad paws
and a curled brown tail. Persian has a long lean quadruped trunk, flowing
raised throat, broad feline face with narrower gaze, six cheek whiskers,
forehead jewel and glint, four complete supporting legs and a long raised
curl tail. Each paw has three formed toes.

Normal front sprites and RGB555 palettes were inspected from the local pack.
Meowth uses white/black plus (31,27,13)/(20,8,3); Persian uses white/black plus
(31,28,10)/(12,9,4). Decoded material checks verify expanded linear colors.
Original pack heights remain 104 (0.4064 m) and 303 (0.9906 m), and both
original authoring heights remain 1.01. Uniform physical scaling, full animated
bounds, camera controls, scenery and admission rules are unchanged.

Meowth has 15 controls; Persian has 13. Both have fixed floor, body, throat,
head, independent ears, supporting shins and a three-control tail chain.
Meowth adds two forelimb/palm chains. Head, gaze, muzzle, nose, smile, fangs,
whiskers and coin/jewel share one coherent head transform. Ear insets share
the pinnae's blend field. Hands and their three fingers share one palm owner.
Paws and toes remain pinned in the battle clips; continuous supporting legs
blend body, knee and fixed floor. World motion constructs two/four actual
walking supports and retains canonical non-leg ownership. Idle/attack/hit
rotate only, use source-relative cue metadata and have neutral endpoints.

| Asset | Vertices | Triangles | Logical/runtime parts | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Meowth | 12,615 | 20,788 | 51/39 | 15 | 200 | 515,760 |
| Persian | 11,977 | 20,220 | 48/36 | 13 | 178 | 463,596 |

Packing combines each palm's fingers and each paw's toes, preserving source
f32 positions/normals/colors/triangles and logical anatomy ranges. Existing
vertex, triangle, accessor, joint, primitive and file-size limits remain intact.
The shared asset reader, strict checker, background builder, bounded rig
registry and production GPU renderers consume canonical GLBs. Legacy JSON
files are removed. `tools/feline_sculpt.py` authors the anatomy;
`tools/feline_glb.py` authors the standard skins.

Meowth source/packed/GLB SHA256:
`3950b34e0106f4064a55ca86890ddc0eaf2f51208cc8f916edf80b90d74efb4a`,
`c517d3dbd7ce7b9ebfecbbd4e0d02ded0530254c2a6bef293e92f112d3affdb8`,
`08b8dc591fca8aaa460e2c21b798a866ffd511467c2a36f840b9575693313d82`.

Persian source/packed/GLB SHA256:
`7b251532fe735c7f21d60622dde0c8bd9418a2b27d6b24c3e765620f227462d2`,
`d0cd25924eb6b4a3c97ab852a94cf24b53e6f65d54dd04ee3b0584804c1fc69b`,
`9315e464e45a04db10adb2588d8ad5e8eefeed5636dfd50927724590d690b191`.

Nineteen Python checks pass in 19.882 seconds: ten source anatomy and nine
independent decoded-skin checks. These cover closed outward solids, shading,
regeneration, distinct anatomy, original grounded dimensions/budgets, real
volume roots (including whiskers, brows, engraved coin, palms, digits and
four-legged supports), fitted parent vertices/edges/centers, visible pupil
area across three pitches, curved-solid self-intersections, detached negative
controls, canonical geometry/packing, hierarchy/inverse binds, used independent
controls, coherent face/hand ownership, fixed battle paws, posed roots, positive
winding and bounded strain, normal palettes, neutral endpoints/source cues and
mutated data rejection. The initial Persian pupil extended below its angular
white lens and the first cheek whisker root missed its muzzle volume; fitting
the pupil and moving the actual whisker roots fixed both without weaker checks.

Three focused Rust checks pass in 1.69 seconds for canonical geometry, fixed
battle paws, original-pack-size GPU admission and two/four complete walking
support chains with actual floor contact and full animated bounds. CLI and
background builder exports match both canonical GLBs byte for byte.
Both native clients build in 63 seconds. Shared schema and exact elapsed-idle/
current-attack-cue checks pass in 116.65/116.30 seconds, bringing this batch to
24 scoped checks. The decoded before/after, front/rear/side and attack sheet
was inspected inline.
Production front pairs at EcruteakCity (20,21) complete in 60.047/47.659 seconds
for Meowth/Persian. Both were inspected inline: complete original-size grounded
models, readable pupils/whiskers/charms and distinctive silhouettes. World and
battle pairs complete in 54.026/104.494 seconds for Meowth and 46.084/99.013
seconds for Persian. All six pairs and both moving world frames were inspected
inline. World models and walks are complete and grounded, with small facial
detail at the wide camera distance. Battle views show readable feline anatomy,
planted paws and actual MEOWTH/PIDGEY and PERSIAN/PIDGEY names, with no fallback.
Both capture groups have complete same-state receipts, identical binary SHA256/
size/mtime identities matching current executables, and unchanged canonical
GLB hashes. Rendering stays headless, offscreen and muted. No shiny, grass,
scenery, AA, audio, content-pack or gameplay changes belong to this batch.
No full-suite or gameplay completion is claimed. Registry reaches 266 skin
variants and nine static species files; full-roster work remains active.

Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/feline_sculpt.py \
  --output target/feline-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/feline-rebuild --only MEOWTH,PERSIAN
```
