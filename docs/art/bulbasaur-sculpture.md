# Bulbasaur dedicated sculpture and skin

Bulbasaur replaces its missing exact battle body with an original spotted
quadruped and nine-joint standard glTF skin. The sole canonical runtime source
is `crates/crystal-voxel-view/models/battle_species/bulbasaur.glb`.
`tools/bulbasaur_sculpt.py` authors anatomy and `tools/bulbasaur_glb.py` authors
its skin and source-cue clips. No icon or evolved-species body was copied or
relabeled. Explicit world species identity shares the exact neutral geometry.

The inspected [official Bulbasaur artwork](https://sg.portal-pokemon.com/pokedex/0001/)
guides the broad flat saur head, triangular ivory/red eyes, pointed hood ears,
small nostrils, broad mouth and short fangs, squat teal torso, four supporting
limbs, twelve ivory claws and fitted angular dark markings. Its bulb has six
actual geometric lobes and a folded tapered tip, rather than a sphere with
painted grooves. Initial review caught an overly tall bulb/long-legged stance;
shortening the bulb and compressing the lower silhouette corrected that ratio.
Stored tests then caught crossed/inward-smoothed bulb cap triangles. Planar
end rings and centered radial cap fans close the scalloped seed correctly.
All surfaces are closed and matte. Shininess remains the existing flag and
source-pack palette acting on the same body and skin.

The final model has 44 named parts, 19,587 vertices and 31,896 triangles,
within the existing runtime reader limits. Floor minimum is exactly zero and
neutral height is 0.95 authoring units; source-pack dimensions own world scale.
Neutral identity digest:
`2789ad2986d637dbe83e6b3b4eb899dfe9ccd06d0746c3332053e72f3bd033df`.
The root fixes all four pads and twelve claws. Torso/head brace early while
four limb joints compress above those supports. Two bulb joints follow the
chest with delayed seed-tip settling. Face and ears remain rigid on the head;
leg markings use their corresponding limb weights. Idle lasts four seconds;
attack braces at 18% of the source cue and the bulb follows at 28%. Hit recoils.
All keys are rotations. Source game programs own movement, audio, projectiles,
damage and timing.

```sh
python3 tools/bulbasaur_glb.py
python3 tools/test_bulbasaur_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view bulbasaur --lib
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view species_rig --lib
```

Seven independent stored-model checks pass, covering exact generation, bounded
storage, closed outward anatomy, normals, physical bounds, distinct features,
weights/binds, rigid face, planted pads/claws, floor clearance, deformation,
neutral endpoints, early brace and bulb follow-through. Native/runtime results
are recorded in `docs/art/animated-models.md`. Neutral software review alone
does not establish native source-cue motion or browser acceptance.

Focused canonical routing and a stored-ring test proving six lobes/six valleys
pass. The final native build embeds the corrected GLB. Its held 3D/2D pair was
inspected at `target/pokemon-native-review/bulbasaur-3d.png` / `bulbasaur-2d.png`.
The receipt confirms unchanged battle state (hash 355223347, LCD frame 47).
The body is grounded; red eyes, angular markings, folded bulb and claws remain
readable at the pack's scale. The source view is its back sprite. Native
lighting darkens the mint-teal body relative to the sprite's bright green, a
remaining fidelity limit. Native source-cue attack/hit, browser and overworld
verification remain open.

Both Cargo Bulbasaur integration tests pass, proving neutral registry sharing,
fixed pads/claws across clips and GPU skin activation at source-pack size
(0.7112 m). The freshly compiled Cargo test executable in
`target/pokemon-models/debug/deps/` passes all 21 active rig tests (one existing
migration audit ignored), all 17 skinning tests and all 231 reusable model
loading checks. The native catalog independently confirms 231 exact bodies
and 20 missing species. The full appearance/rigging goal remains unfinished.
