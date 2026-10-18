# Mankey and Primeape: rooted fur, angry gaze and articulated fists

Both static sphere assemblies are replaced by continuous round white pelts,
rooted pointed fur locks, fitted angry eye sockets/whites/pupils, broad pig
snouts with paired nostrils, pointed ears with fitted inner pinnae, continuous
bent limbs and formed paws. Fists have three folded knuckles and a rooted
thumb; each grounded foot has three projecting toes. Mankey retains a long
curled brown tail. Primeape has a wider pelt, denser crown/chin fur, larger
fists and visible dark wrist/ankle bands.

The normal local-pack front sprites were inspected directly. White, black,
warm midtone and brown materials match each species' four-color palette:
Mankey RGB555 (29,20,11)/(19,9,7), Primeape (31,18,12)/(16,8,5). Decoded
material checks verify the expanded, linearized values. Pack heights remain
`108` (Mankey, 0.508 m) and `303` (Primeape, 0.9906 m). Both authoring heights
remain 1.02, with uniform production scaling and full animated bounds. Camera,
collision, source placement and admission rules are unchanged.

Mankey has 12 controls and Primeape ten: fixed floor, the single body/face,
two forelimb/fist chains, two ears and two supporting shins, plus Mankey's tail
base and tip. Pelt, fitted gaze, snout and nostrils share the body; knuckles,
thumbs and wrist bands share the appropriate fist. Feet and toes remain pinned
in battle clips. Legs and ears blend at their roots, and the long tail has
separate base/tip articulation. Idle, attack and hit rotate only, use source
cues and have neutral endpoints. Production world motion builds two complete
walking support chains, retaining the canonical fur/face/fist bindings.

| Asset | Vertices | Triangles | Logical/runtime parts | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mankey | 11,992 | 19,964 | 52/38 | 12 | 176 | 493,904 |
| Primeape | 13,633 | 23,004 | 59/45 | 10 | 185 | 526,680 |

Coherent packing combines each fist's digits and each foot's toes without
changing source f32 anatomy, normals, colors or triangles. All existing model
limits remain intact. `tools/pig_monkey_sculpt.py` authors the anatomy;
`tools/pig_monkey_glb.py` exports the canonical standard skins. The shared
reader, builder, checker, bounded registry and GPU renderers consume those
GLBs; both legacy JSON assets are removed.

Mankey source/packed/GLB SHA256:
`15aa326ed611cbb9226add1d0fdda0e0cfe567d565e6bdc09998ed410856e4ce`,
`13470b899782dfb2cb9de04a9579f5d061061c75dbbb95cfe913ed0e01500e52`,
`2187abad2dee588aba0055c58416611a608193741b43206b40f88cd51a0098e2`.
Primeape source/packed/GLB SHA256:
`3bc3bdcfdadf6699cca07fa2a056a240dfad8af39cc863bc99a91efb2040970e`,
`45c0c8e59be21979e46ab5fe5c7ffbeae3c3ec9ad54858740b918aac26ee564f`,
`5f7dbe5d7318b7f8c265f2fa1e1f073f8ba05436fc9efc0ed5d7becce32b20c4`.

Nineteen Python checks pass in 11.936 seconds: ten source anatomy and nine
independent decoded-skin checks. They cover closed outward solids, normals,
regeneration/grounded dimensions/budgets, distinct silhouettes and appendages,
actual fur/ear/snout/limb/knuckle/thumb/toe/cuff/tail roots, fitted surface
clearance, self-intersections and detached negative controls, exact logical
packing, hierarchy/inverse binds, coherent ownership and independent controls,
fixed paws, positive posed winding/bounded strain, actual posed attachments,
normal palettes, neutral endpoints/source cues and mutated data rejection.
The original tight knuckle bend crossed itself; a broader C-shaped arc fixes
the geometry without relaxing intersection or strain checks.

Three focused Rust checks pass in 1.75 seconds for canonical geometry and
fixed paws through clips, original-size GPU admission, two complete walking
support chains, canonical non-leg bindings, real ground contact and full
world bounds. Shared schema and exact elapsed-idle/current-attack-cue checks pass in
93.08/92.67 seconds, bringing the batch to 24 scoped checks. Both native
clients build in 52.02 seconds. CLI and background exports match both
canonical GLBs byte for byte. These checks do not claim full-suite or
gameplay completion.

The decoded before/after, front/rear/side and attack sheet was inspected inline.
Front production pairs at EcruteakCity (20,21) completed in 36.499/33.797
seconds for Mankey/Primeape and were inspected inline. Both preserve the same
authoritative state between 2D and 3D, with complete original-size models,
readable pupils/nostrils, formed fists/toes and grounded paws. Primeape bands
and denser pelt distinguish it from the smaller tailed Mankey. World and battle pairs completed in 33.798/104.127 seconds for Mankey
and 57.204/112.107 seconds for Primeape. All six production pairs and both
moving world frames were inspected inline. World companions are complete
and grounded at their original sizes, although facial detail is small at
the wide camera distance. Both battle views show clear silhouettes, fitted
gaze/snouts, grounded paws and actual species names (MANKEY/PIDGEY and
PRIMEAPE/SENTRET), with no fallback. Same-state receipts are complete, and
both groups retain identical SHA256/size/mtime binary identities matching
the current executables; canonical GLB hashes also remain unchanged.
Rendering stays
headless, offscreen and muted. No shiny, grass-height, scenery, AA, audio, pack
or gameplay changes belong to this batch. Registry reaches 264 skin variants
and 11 static species files; full-roster improvement remains active.

Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/pig_monkey_sculpt.py \
  --output target/pig-monkey-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/pig-monkey-rebuild --only MANKEY,PRIMEAPE
```
