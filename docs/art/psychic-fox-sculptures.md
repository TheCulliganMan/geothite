# Abra and Alakazam sculptures and skins

`tools/abra_sculpt.py` authors the original psychic fox anatomy.
`tools/psychic_fox_glb.py` writes the canonical skins consumed by the production
GPU and world rigs. Both old static JSON duplicates are removed, and the main
Blender builder and strict model readers use the GLBs.

The torso and chest cuirass use curved profiles. Palms and insteps have smoother
sampling, bent forearms and curled fingers use curved paths, and the tapered
fox skull retains its deliberate angular silhouette with connected shading.
Ear interiors and gaze patches fit the stored parent triangles; adaptive local
refinement keeps the linings above the ears' sharp folds. Upper arms extend
beneath the shoulder shells into the actual torso. Alakazam's moustache roots
extend inside the muzzle rather than relying on a narrow edge contact.

Abra's heavy tail follows a continuous curve. Its brown saddle band is authored
from the same sampled centerline and radii, with a small radial overlap.
Alakazam keeps both serrated moustache blades and both held spoons, including
the oval bowls and recessed faces. Each species has three curled fingers and
three splayed, clawed toes per side. All 43/53 named anatomical parts remain.

| Species | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Verified pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Abra | 7,254 | 12,040 | 15 | 185 | 335,248 | 1.12 | 0.889 m |
| Alakazam | 7,513 | 12,264 | 17 | 222 | 343,788 | 1.12 | 1.4986 m |

Both retain the unchanged 20,000-vertex, 40,000-triangle, 64-part, 256-accessor
and 1 MiB limits. Rest geometry SHA-256 pins:

- Abra: `0f62090ca5311bb42f9f3a8375859f0336df8bfe1e76752e34588c9deb257eb9`
- Alakazam: `9798ee114b47db333f11ac0dfddb761373146d62684df9051613f8585fdb4f5c`

Each rig has a floor root, blended torso/neck and limbs, coherent head and gaze,
independent ears, two three-joint arm chains and two leg carriers. Instep,
toe and claw surfaces stay fixed in battle. Abra adds two tail carriers whose
influences also drive the brown band. Alakazam adds two moustache carriers
pivoting inside the muzzle and two spoon carriers parented to the wrists and
pivoting at the finger grips. The spoons stay supported by the curled fingers
while moving relative to the wrists. World walking builds two complete support
chains and preserves tail, moustache and spoon weights.

Idle uses 81 keys over 3.2 seconds; attack and hit use 65 keys over one second.
Source battle cues own action timing, and all clips have neutral endpoints.

The eight skin checks and four anatomy checks pass. They cover canonical bytes,
strict rejection of mutations, hierarchy, weights, identity binds, independent
carriers, fixed feet, winding and bounded strain across clips, actual occupied
anatomical roots, connected fingers/toes/claws, fitted ear/gaze clearance,
held spoon support and bowl/inset joins. The three focused Rust checks pass for
verified-size GPU activation, production rig motion/culling and 64 world walking
phases with two support chains and preserved prop/tail influences.

Blender exports match both canonical files byte for byte. Front, side and rear
software sheets compare the earlier neutral models with final idle, attack and
hit poses; these were inspected as art previews. Native paired review evidence is recorded below. Browser visual acceptance remains open. Shiny checks
are skipped as requested, and grass height is untouched.


Both shared all-species geometry and exact source-cue timing checks passed
(2 tests, 133.82 seconds). The native client was rebuilt before capture.
The Abra/Pidgey Route 29 pair was inspected at preserved state: frame 3659,
checksum 293534093, LCD frame 4382 and virtual time 79.20587775. The closed gaze,
pointed muzzle/ears, chest armor, curled fingers, banded tail and clawed feet
remain readable. Both feet meet terrain and the HUD says ABRA. Normal terrain
admission passed at original pack size and the existing -110-degree review
yaw with normal support/omission/whole-body/frustum checks active, no fallback
and no shader errors. Ignored evidence: `target/pokemon-native-review/abra-normal`.


The native Alakazam/Sentret Route 29 pair was also inspected at preserved state:
frame 2118, checksum 4140350985, LCD frame 2820 and virtual time 49.969093667.
The focused eyes, pointed ears/muzzle, seated moustache blades, chest/shoulder
armor, curled fingers and both spoons remain readable. Both clawed feet meet
terrain, and the HUD says ALAKAZAM. Normal admission passed at original pack
size and existing -110-degree review yaw with normal support, omission,
whole-body and frustum checks active, no fallback and no shader errors.
Ignored evidence: `target/pokemon-native-review/alakazam-normal`. Browser
visual acceptance remains open. Shiny checks and grass-height edits remain
skipped.
