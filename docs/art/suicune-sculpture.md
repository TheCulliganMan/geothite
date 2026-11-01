# Suicune sculpture and skin

`tools/suicune_sculpt.py` authors a distinct slender quadruped, with fitted
white flank diamonds, a narrow muzzle, a white throat marking, an open diamond
crest, a continuous purple wind mantle, and two broad streaming ribbon tails.
Anatomy is checked against the local pack's front/back sprites and
[official Suicune artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/245.png).

The verified pack gives a Pokédex height of 6 feet 7 inches, or 2.0066 metres.
Its normal, front, and back palette anchors agree: RGB555 blue `(11,23,31)`
and purple `(14,2,20)`, expanded to RGB8 `(90,189,255)` and `(115,16,165)`.
The authored normalized silhouette height is 1.60. Production integration
must apply the verified physical height.

The current draft has 47 parts, 10,500 vertices, and 17,216 triangles. Its
diamond crest is a closed annular solid with a real opening. The ribbons have
broad elliptical sections rather than round rope sections. Four paws meet
the floor. White flank and hind-leg diamonds, the throat marking, and eye
layers fit the actual parent triangles.

The four independent draft regressions passed:

```sh
PYTHONPATH=tools python3 -m unittest test_suicune_sculpt -v
```

The tests cover closed outward geometry, shading, the crest's topology and
opening, ribbon proportions, source palette anchors, floor contacts, and
parent-triangle clearance. An oblique neutral review exposed a partly buried
throat marking; fitting it to the neck fixed the edge, and the revised render
was inspected again.

`tools/suicune_glb.py` now exports a standard eighteen-joint skin. The torso,
neck, head, and hollow crest have a coherent hierarchy. A three-joint chain
drives the wind mantle, and separate three-joint chains drive the left and
right ribbons. Four leg joints blend from the identity floor root; paws and
toes remain fixed to that root. Fitted leg diamonds share the leg weight
field, and the white throat follows its neck. Idle, attack, and hit clips use
rotations, return to neutral at both endpoints, and retain source-owned cue
timing.

The reviewed draft GLB has 187 accessors in 420,052 bytes and preserves the
neutral geometry digest
`c3634acaa906645bb52cef6dcc37411a87b9dbd3862b908b3d4235e7c933743e`.
The export and all eight stored-model regressions were executed successfully:

```sh
PYTHONPATH=tools python3 tools/suicune_glb.py \
  --output target/pokemon-native-review/suicune-rig.glb
SUICUNE_GLB=target/pokemon-native-review/suicune-rig.glb \
  PYTHONPATH=tools python3 -m unittest test_suicune_glb -v
```

The tests additionally check decoded joint ownership, independent ribbon
chains, floor contacts, winding, edge strain, closed clip endpoints, and
crest/mane/ribbon articulation. Neutral and attack poses decoded from the
stored GLB were rendered and inspected.

The production registry now resolves `SUICUNE` through `Species::Suicune`.
Battle and world prop routing share `models/battle_species/suicune.glb`;
the old world JSON proxy is removed. Both focused Blender build paths were
executed and reproduced this GLB byte for byte. The canonical routing checks
passed with 251 battle bodies. The standard registry has 67 variants across
42 species; this does not mean that the entire catalogue has been rigged.

The two focused production regressions passed at 2.0066 metres, preserving
planted paws while the crest, mane, and ribbons move. The registry schema
check, all 43 GPU skinning regressions, and all five actor-prop regressions
also passed. Native capture examples were built and normal/shiny paired
views inspected against their matching 2D states. The complete silhouette
fits the frame, the paws meet the floor, and the mantle, crest profile,
diamonds, and flowing ribbons remain readable. The crest opening is visible
in the frontal/oblique authoring view; the production side view naturally
shows its narrow profile.

The normal pair preserved checksum `2670416232`, LCD frame 64, at virtual
time `1.676269708`. The shiny pair preserved checksum `2162880981`, LCD frame
64, at virtual time `1.658163541`. Both report `state_preserved=true` and use
the same stored geometry and rig. These checks establish this model milestone,
not a completed playthrough, browser acceptance, or complete catalogue rigging.
Temporary references and reviews remain under ignored `target/`.
