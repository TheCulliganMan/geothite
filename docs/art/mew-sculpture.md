# Mew soft psychic cat and looping tail skin

`tools/mew_sculpt.py` authors an original dedicated psychic cat guided by
[the official Mew artwork](https://sg.portal-pokemon.com/pokedex/0151/).
`tools/mew_glb.py` exports its canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/mew.glb`. The common Blender
battle-species builder regenerates identical bytes. The old 27-part generic
upright torso/button-snout/block-ear JSON was removed. Mewtwo's subsequent
dedicated replacement is documented in `mewtwo-sculpture.md`. No reference pixels or third-party geometry are imported.

A compact pink pear-shaped torso supports a short throat and a rounded feline
head with soft cheeks and a tapered chin. Fitted dark eye rims carry tapered
ivory lenses, blue irises, deep-blue pupils and white highlights. Soft pointed
ears have fitted dusky-pink interiors. A tiny fitted smile replaces the button
snout. Short arms end in little three-finger hands. Rounded thighs lead to long
tapered hind paws with two shallow toe creases per paw. A thin continuous whip tail
loops behind the body and ends in an oval pink terminal. Materials are matte.

The first paw shape read as slippers and was replaced by a long downward taper.
The initial straight eye outline was rounded with authored Bezier curves, and
the tail's centerline sampling was increased to smooth the loop. Its source
geometry has 40 closed parts, 18,486 vertices and 30,224 triangles, 13 joints
and 190 accessors, in 758,636 bytes. All renderer limits remain unchanged.
The neutral/source SHA-256 is
`36d3fc53223d21ac492d3d93879589a130773ba3d884c34a65f6173b3980d37e`.
Authoring height is 0.78 with minimum zero. The ears, rather than the looping
tail, establish that height. The verified user-supplied local pack gives height
digits 104 (1 foot 4 inches), or 0.4064 metres, classification `NEW SPECIE`.

The torso, short neck, complete gaze, ears, hands and hind limbs articulate
separately. Inner ears share their ear joints; all three fingers share each
hand's arm carrier; paw creases share the foot carrier. Three yawing tail
joints blend by nearest-centerline arc length through the loop. The oval terminal
shares the fully weighted distal carrier. Tail height is preserved. Rotation-only
idle, attack and recoil return literally to neutral and follow source cue time.
The existing render-only hover lift remains unchanged. An actual posed-vertex
check proves Mew's current conservative fitting envelope already covers that
lift, so no renderer or physical-size change was needed.

Ten stored-model Python checks pass for canonical parity/schema, closed outward
solids, normals, fitted front triangle/edge clearance, occupied roots and a
detached-tip negative control, coherent owners, three tail carriers, posed
floor, winding/strain, independent motion and literal neutral endpoints.
Three focused Rust checks pass for exact registry routing, skin ownership,
independent anatomy, culling bounds, rendered-hover coverage and GPU activation
at verified pack size. Blender regeneration is byte-identical. Front/rear
neutral, idle, attack and recoil software views were inspected against the old
placeholder. Shiny verification is skipped at the user's request.

```sh
python3 tools/mew_glb.py
python3 tools/test_mew_glb.py
```


The wider exact-species grounding/volume and elapsed-idle/source-attack timing
regressions pass. The rebuilt production Route 29 encounter against Sentret
passes terrain admission (`accepted=true`). Its inspected 3D/2D pair preserves
state at core frame 1148, checksum 1008858066, LCD animation frame 1686 and
virtual time 29.109839834. The feline face, soft ears, small hands, long paws,
looping whip and oval tail tip fit fully in view. The standard hover clearance
remains visible at verified pack size. Original grass and forest remain in
place; no shader errors were reported. This verifies the held native view;
browser visual acceptance remains open. Verification files use prefix
`target/pokemon-native-review/mew-normal` and remain ignored.
