# Elekid and Electabuzz sculptures

The current [volume and posture audit](elemental-fidelity.md) supersedes the
older front/quarter-only review. `tools/electric_pair_sculpt.py` authors original
geometry; `tools/elekid_glb.py` and `tools/electabuzz_glb.py` export and strictly
read the canonical GLBs under `crates/crystal-voxel-view/models/battle_species/`.
Reference game meshes and textures are never authoring inputs.

Elekid has a round, deep egg body, two black waist bands, fitted bolt and narrow
gaze, shorter open contact blades, dark upright shins and broader yellow feet.
One bent striped forearm rises while the other bends low and forward, following
Crystal's alternating fist posture. Three short claws finish each fist. Nine
joints retain coherent arms, open blades, fitted face and fixed battle feet.
Normal colors retain the local pack's yellow (255,255,41), black and white.

Electabuzz has a deeper round belly, a bell-shaped flared skull, unequal pointed
scalp tufts, occupied ball-tipped antenna roots, broader fang-bearing jaws and a
tongue. Both heavy striped forearms bend beside the head. The supported shins
are upright in profile; the toes project from the broad feet. A fitted dorsal
stripe, chest bolt, flank sweeps and curved striped tail remain. Nineteen joints
follow the revised shoulder, elbow, wrist, leg, crest and antenna anatomy.
Continuous arm fields keep stripes coherent across bent elbows.

| Species | Logical/runtime parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Elekid | 33 / 33 | 11,517 | 19,024 | 9 | 148 | 452,776 |
| Electabuzz | 70 / 41 | 19,747 | 33,880 | 19 | 243 | 879,676 |

Authoring heights remain 0.91 and 1.28; production pack heights remain 0.6096 m
and 1.0922 m. Existing geometry, accessor, part and byte limits are unchanged.
The combined audit records exact pins, executed checks, references, full orbits,
pose samples and production coverage. The earlier
[source review](electric-pair-source-review.md) is historical reference context.
