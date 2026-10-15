# Entei armored volcano-beast body and skin

Entei fills a battle-body gap with one canonical GLB at
`crates/crystal-voxel-view/models/battle_species/entei.glb`.
`tools/entei_sculpt.py` replaces the old small feline sphere proxy with a heavy
torso, raised neck, hanging mane, pointed chest beard, red cheek plates,
silver muzzle shield, swept golden crown, fitted stern eyes, broad pale paws,
thick ankle bands, six silver dorsal blades and connected white smoke trails.
The anatomy was checked against the supplied Crystal sprites and
[official Entei artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/244.png).

The verified external pack supplies height digits 607: 2.0066 m (6 feet 7 inches).
Its normal/front/back palettes agree. Body/crown RGB555 `(24,14,0)` becomes
RGB8 `(198,115,0)`; red face plates `(21,0,0)` become `(173,0,0)`. Source
white/black supply cloud and eye anchors. Cream/silver anatomical accents and
a darker amber mane retain the sculpture's material separation.

The GLB contains 59 named parts, 15,542 normal-split vertices, 26,368 triangles,
16 joints, 249 accessors and 626,672 bytes. Authored height is 1.42. Neutral digest:
`db453e0d91aa89189660d76f63e17f62e26c4d80e750455f9fb7e066cbcc249c`.
Existing reader limits remain unchanged. Front and oblique neutral geometry
reviews were inspected before integration.

The identity root owns all four paws, twelve toes and four ankle bands. Upper
legs blend into four leg bones. The torso, neck and coherent armored head have
separate owners, with paired mane and dorsal-blade bones and two chains for the
upper cloud and trailing smoke. Tail weights follow the nearest actual authored
centerline chord. Idle lasts four seconds with 85 keys; attack/hit last one
second with 65 keys. Rotation-only clips return to neutral and use the existing
source battle cue clock. The paws and ankle bands remain fixed throughout.

The first native view exposed flat-looking face and mane panels. The red/silver
mask surfaces now wrap across the head, the mane curves around the shoulders,
the muzzle sits closer to its shield, and crown tips sweep back. The silhouette,
materials and existing rig stay intact. Updated oblique geometry was inspected
again before rebuilding the native examples.

Eight final stored-model checks pass: canonical bytes and limits, closed positive
geometry and shading, distinctive anatomy, fitted eye-layer clearance against
stored parent triangles, exact pack body/face colors, hierarchy/owners, floor
clearance, support contacts, winding/strain and head/mane/smoke articulation.
The pure CLI reproduces canonical bytes. Builder branches are syntax checked;
full Blender batches were not executed.

Battle and world routing share the same canonical skin, and the old world JSON
is removed. The other two beast recipes remain unchanged. Before the wrap correction,
44 active rig checks (one existing ignored), 41 GPU-skinning checks, five
actor-prop checks and two source-routing checks passed. After the wrap correction, both focused Entei registry/GPU checks and the
complete model-schema check pass. Canonical routing and CLI byte reproduction
pass again. The native examples rebuilt successfully.

Repeated native 3D/2D captures at 1000×700 (image target at scale 2) preserve
state for normal and shiny Entei. Normal checksum 979079861, LCD frame 54,
virtual time 1.330801542; shiny checksum 1399097452, LCD frame 52, virtual time
1.303138791. The corrected cloak sits around the shoulders, crown prongs sweep
back, and the shield is integrated with the face. The complete body, blades,
smoke and planted paws fit the frame beside the default wild Sudowoodo at the
pack's physical scale. Shiny color changes agree with the paired source palette
and reuse the same geometry and skin. This verifies the held native views;
source-frame motion, further angles and world/browser fidelity remain open.

The catalogue now contains 249 battle bodies; Raikou and Suicune remain gaps.
The standard rig registry contains 65 variants covering 40 Pokémon species. The full all-Pokémon goal remains
active; remaining static rigs, source-frame motion, additional native/world
views and browser fidelity remain open. Packs and audio are unchanged.

```sh
python3 tools/entei_glb.py
PYTHONPATH=tools python3 -m unittest test_entei_glb -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```
