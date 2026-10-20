# Golbat and Crobat eye fitting

The pupil recipes previously sampled the body surface with a fixed offset,
leaving a small gap above the actual ivory or yellow eye. Both now sample the
stored eye mesh directly. Their narrow pupil outlines and normal palettes stay
the same, while the backs overlap their supporting eyes.

Golbat retains 32 parts, ten joints, and 4,784 triangles; Crobat retains 28
parts, fourteen joints, and 4,116 triangles. Their complete neutral bounds and
1.0 reference heights are unchanged. The new assets contain 3,762 and 4,653
vertices and occupy 200,928 and 248,628 bytes respectively. Sampling the actual
surface changes attribute welding without adding triangles. Non-pupil triangle
positions, normals, joint assignments, and weights are identical after index
expansion. Golbat's ivory eyes have equivalent reindexed buffers. Hierarchy,
inverse binds, and decoded animation keys are unchanged. Zubat is byte-identical.

Thirteen Python checks passed, including actual pupil/eye overlap and exposed
cap clearance through all source clips at three sample times. Reintroducing
either old asset fails the new contact regression. The Rust bat family check
and full 276-variant anatomy/skin/size schema check also passed. Both CLI exports
reproduce the integrated assets exactly:

```sh
python3 tools/bat_glb.py --only golbat --out target/bat-eye-review
python3 tools/bat_glb.py --only crobat --out target/bat-eye-review
python3 -m unittest discover -s tools -p 'test_bat_*.py'
```

Four muted headless native pairs were inspected at Ecruteak `(18,21)`, noon:
front and world views for each species, alongside their same-state 2D maps.
One actual moving field frame per species was also inspected. Pupils remain
readable, hollow mouths retain their depth, and the articulated wing silhouettes
remain intact. World captures recorded 32 synthetic movement samples through
the expected five positions. Crobat's outer wing reaches the moving frame edge;
that frame is a motion/attachment review rather than a full silhouette plate.
The current GLBs were checked inside the native binary before capture. No
windows were opened, and no shiny variants were tested. This scoped review
does not establish completion of every Pokémon model.
