# Magby live-coal sculpture and skin

The current [volume and posture audit](elemental-fidelity.md) records the full
reference comparison and verification. `tools/magby_sculpt.py` authors original
geometry; `tools/magby_glb.py` exports and strictly reads
`crates/crystal-voxel-view/models/battle_species/magby.glb`.

The body widens at the low hips and narrows into the dark collar. A deeper,
tapered head supports five embedded cloud lobes, broad pupils, fitted tall eyes
and a short open bill. Arms sit low with upturned clawed palms; each wider foot
has three claws. Short shins remain upright in both profiles, and the plain
pointed tail curves behind the hips. There is no tail flame.

The bill has an actual empty aperture before occupied dark backing. Decoded
pose checks move probes with the head, derive their location from the current
lip, and test every solid through idle, attack and hit samples. Feet and claws
stay fixed in battle; the walking adapter retains complete support chains.
Eleven joints follow the crown, neck, low arms, legs and two-stage tail.

The canonical skin has 40 parts, 12,864 vertices, 22,336 triangles, 164 accessors
and 473,284 bytes. Authoring height remains 0.77; the local pack's production
height remains 0.7112 m. Normal colors retain the pack's purple-pink (189,16,90),
warm orange (255,148,74), white and black. Later red game references establish
anatomy without replacing this chosen normal palette. Existing loader limits
remain unchanged. No downloaded geometry, textures, rigs or clips are imported.
