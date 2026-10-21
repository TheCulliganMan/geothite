# Noctowl crowned owl skin

The [2026-10-21 owl fidelity revision](owl-fidelity.md) supersedes the shape,
palette and measurements below. Noctowl now has a deep breast and folded cases,
buried shoulder caps, broad gold three-point plumes joined by solid gold brows,
red irises, upright gold shanks and pink bill/claws. The current skin has 7,706
vertices, 10,076 triangles, 52 parts, 14 joints and 372,248 bytes. The remaining
text records the earlier migration and its limited indexed-palette assumptions.

The existing production body was inspected against its matching 2D battle
state. The protruding eye disks, cone-shaped brows, floating breast marks and
thin pink feet lose Noctowl's owl silhouette. The preserved-state baseline
receipt records checksum 281572228, LCD animation frame 91 and virtual time
2.881794166 seconds. The pack's front and back sprites and
[official Noctowl artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/164.png)
provide the anatomy references.

The verified local pack specifies a height of 5 feet 3 inches (1.6002 metres).
Its normal RGB555 anchors `(18,12,4)` and `(11,5,6)` expand to RGB8
`(148,99,33)` and `(90,41,49)`. The new recipe preserves those colors alongside
white and black. It uses a continuous head/breast surface, fitted eye disks and
dark rings, a swept brow bridge, forked crown horns, folded wings with seated
flight feathers, six fitted breast marks, a hooked bill, a layered fan tail,
and two broad feet with three forward digits, curved claws and rear halluces.

The first authoring view prompted a broader neck and crown and higher wing
shoulders. A face-clearance test also exposed a triangle clipping through the
head. The revised fitted layers clear the actual parent surfaces. Excess eye
and breast subdivisions were reduced; the revised body has 52 parts and
14,815 vertices. All three draft geometry tests pass:

```sh
PYTHONPATH=tools python3 -m unittest test_noctowl_sculpt -v
```

The standard GLB draft has 14 joints, 22,660 triangles and 224 accessors in
641,160 bytes, with a normalized silhouette height of 1.30. Its neutral digest
is `fc6025dcce69b1c6b5a9dd2a79254190efe7692e600171dddf3d316e29a1a713`.
Pelvis, chest, neck and head drive the body and fitted face. Paired crown,
wing and leg joints and a three-stage fan-tail chain provide articulation.
Both feet, all forward toes and claws, and both halluces remain on the identity
floor root. Idle, attack and hit use rotation keys and return to neutral at
their endpoints. Neutral and decoded attack authoring views were inspected.

All seven stored-GLB checks pass, including closed outward geometry, shading,
face clearance, source palette, canonical bytes, hierarchy, fixed digit owners,
floor contact, winding, edge strain, endpoints and visible articulation:

```sh
PYTHONPATH=tools python3 tools/noctowl_glb.py \
  --output target/pokemon-native-review/noctowl-rig.glb
NOCTOWL_GLB=target/pokemon-native-review/noctowl-rig.glb \
  PYTHONPATH=tools python3 -m unittest test_noctowl_glb -v
```

The canonical GLB replaces the production Noctowl JSON and is registered with
the exact-species catalogue and the standard skinning path. Its real focused
Blender builder produced identical canonical bytes. All eight combined stored
GLB and catalogue-route Python checks passed. Two production Rust tests passed:
playback keeps all sixteen support parts fixed while the face, crown, wings and
tail articulate, and fourteen-joint GPU skinning uses the verified 1.6002-metre
pack height. The schema check passed for all 69 standard rig variants.

The complete authoring check passed with 59 editable Blender sources,
609 runtime models in 609 files, and 731 embedded references. Catalogue coverage
remains 251 Pokémon species; this model replaces its existing static body.

A fresh native build produced paired 3D/2D captures from the production battle
renderer. Both normal and shiny pairs were inspected and preserve their own
state. Normal: checksum 3005264234, core frame 0, LCD animation frame 56,
virtual time 1.365802917 seconds. Shiny: checksum 1530325495, core frame 0,
LCD animation frame 53, virtual time 1.370406417 seconds. Each pair is its own
fixture with independent DVs and stats. The entire silhouette fits, feet and
claws remain grounded, face layers stay fitted, and folded wings and the fan
tail remain distinct. Normal brown/dark and shiny green/red anchors match the
paired classic views. The flat crown reads broader in the frontal authoring
view and naturally thinner in the native profile. No facing override was added.

This accepts the current Noctowl battle-model milestone. The standard registry
now covers 44 species through 69 variants, plus the separate legacy Pidgeotto
rig. Most catalogue species still need dedicated rigs; world motion and browser
fidelity also remain open. The full all-Pokémon goal is not complete. Reference
scratch files were removed after acceptance; final review captures stay ignored.
