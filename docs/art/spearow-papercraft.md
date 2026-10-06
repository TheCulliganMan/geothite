# Spearow papercraft sculpture and animation

Spearow now has one canonical asset at
`crates/crystal-voxel-view/models/battle_species/spearow.glb`. Its editable source
is the deterministic standard-library recipe `tools/spearow_glb.py`; there is no
parallel neutral JSON or new Blender geometry source.

The sculpture follows the Crystal battle silhouette: a tapered chestnut head,
swept brown crown and cheek points, a short pink wedge beak, small slanted ivory
eyes, a ragged throat edge, broad coral wings with cream upper patches, short
brown tail feathers, and planted pink toes. The body uses a shaped loft with
smooth normals; the crown, coverts, flight feathers and tail have intentional
solid paper folds. Eye surfaces follow the head loft instead of floating flat
panels. Overlapping flight-feather layers share their anatomical hinge.

The neutral model has 2,139 vertices, 1,929 triangles, 28 named anatomy parts and
10 joints. Its bounds are `(-0.368, 0, -0.425)` to `(0.368, 0.75, 0.425)`. The
original 0.75 authored height remains the full Y reference span. Runtime physical
size continues to come from the loaded content pack and the shared meter scale.
No per-species readability multiplier is introduced.

The skin uses the existing bounded glTF contract: one mesh and skin, normalized
U8 weights, exact inverse binds, an identity root, and rotation-only clips.

- `spearow.idle`, 2.8 seconds: an alert avian glance, chest breath, a small folded
  wing adjustment and a delayed tail twitch
- `spearow.attack`, cue relative: a short braced peck, lower-beak opening, and
  counterbalancing wing spread
- `spearow.hit`, cue relative: head and chest recoil with a lagged wing and crown
  response

Every clip begins and ends at the neutral pose. All foot geometry stays on the
identity root. The game still owns battle decisions, original move timing,
effect placement and HP changes; these clips supply only local presentation.

Rust registers the asset in `species_rig` and routes the species catalogue to
the same rig's exact neutral surface. The existing generic `battle_skinning`
path owns GPU joint instances, inverse-bind sharing and cue sampling. Spearow
uses the ordinary full-Y size reference; a pack value of 0.3048 m therefore
remains 0.3048 m under the shared world scale, independently of the animation
envelope used for camera fitting.

The Rust regressions check the decoded neutral geometry and stored clip keys,
each joint's local motion and its own skin influences, planted feet, neutral
size/contact across viewports, and actual generic GPU skin activation with
separate live joints for two Spearow instances. Spearow also participates in the
shared clip-clock, closed-endpoint, envelope and immutable-buffer regressions.

Regenerate and check from the repository root:

```sh
python tools/spearow_glb.py
python tools/test_spearow_glb.py -v
python tools/check-battle-species.py
python tools/check-johto-models.py
cargo test --locked -p crystal-voxel-view species_rig::tests
cargo test --locked -p crystal-voxel-view battle_view::skinning::tests
cargo test --locked -p crystal-voxel-view battle_species_models::tests
```

The all-species Blender entrypoint routes Spearow directly to this recipe before
its scene batching. `--only spearow` emits the same GLB and no static JSON or
Blender copy. Model checks independently decode the stored vertices, normals,
weights, hierarchy and clip samples to verify winding, bounded motion, neutral
endpoints, fixed toes and meaningful head/wing articulation.
