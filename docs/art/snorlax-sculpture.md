# Snorlax dedicated sculpture and skin

The [October 21 volume revision](mantis-sleeper-fidelity.md) supersedes the
earlier dimensions and inspection below, with a deeper barrel, larger hood
and paws, bent resting arms and verified buried roots through motion.

Snorlax replaces its missing exact battle body with an original seated sleeping
bear sculpture and nine-joint standard glTF skin. The sole canonical runtime
source is `crates/crystal-voxel-view/models/battle_species/snorlax.glb`.
`tools/snorlax_sculpt.py` authors geometry and `tools/snorlax_glb.py` authors
skin and source-cue clips. Neither legacy `big_snorlax` nor `icon_snorlax` was
copied or relabeled. Explicit species identity shares the immutable neutral
battle body; legacy prop identities retain their existing models.

The inspected [official Snorlax artwork](https://sg.portal-pokemon.com/pokedex/0143/)
guides the broad seated torso, notched cream sleeping face, tiny smile/fangs,
pointed ears, drooping blue arms, cream belly, rounded feet, brown sole pads and
three ivory claws on each hand and foot. The first neutral review exposed
faceted cream patches and blue peeking through the belly. Increasing fitting
density and smoothing welded normals corrects those defects without adding a
second shell asset. A front silhouette review also caught an overly upright
stance and tall narrow ears. The final body is broader and shorter beneath the
hood, with outward tapering ears and corrected transformed normals. Materials are matte. Shininess remains the existing flag
and source-pack palette operating on the same body and skin.

The canonical model has 31 closed named parts, 13,680 vertices and 21,820
triangles. Its exact floor minimum is zero and neutral height is 1.40 authoring
units; source-pack dimensions own physical size. Neutral identity digest:
`91058a0c15e86e5e6ff07f98ff22bafbc6cd0752f2e861fe80c74ea58a74426c`.
The root fixes soles, sole pads and foot claws. Torso drives the body/belly;
head owns face and fangs. Separate arm, paw and ear joints keep attachments
consistent through breathing, attack brace and hit recoil. Idle lasts 4.4
seconds; attack starts early and braces at 18% of the source cue, with paws and
ears following at 28%. All keys are rotations. Source game scripts still own
movement, projectiles, audio, damage and timing.

```sh
python3 tools/snorlax_glb.py
python3 tools/test_snorlax_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view species_rig --lib
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view snorlax --lib
```

Review images and build/test output remain ignored under
`target/pokemon-native-review/`. Verification results and production critique
are recorded in `docs/art/animated-models.md`. Neutral software review alone
does not establish native source-cue animation or browser acceptance.

The final six independent stored-model checks, focused canonical routing and
seated-width/short-ear silhouette regression pass. Native GLB metadata matches
the canonical asset. The held native battle pair at
`target/pokemon-native-review/snorlax-3d.png` / `snorlax-2d.png` was inspected;
its receipt confirms unchanged battle state (hash 690171214, LCD frame 40).
Scale, grounded soles, sleeping eyes and twelve claws are readable. The source
view uses Snorlax's back sprite. The native cream mask edge remains visibly
faceted, and lighting darkens blue to teal; full visual acceptance is still
open, along with source-cue attack/hit, browser and overworld review.

Both Cargo Snorlax integration tests pass, proving immutable neutral registry
sharing, planted soles across clips and generic GPU skin activation at source
pack size (2.1082 m). The freshly compiled Cargo test executable also passes
all 19 active rig tests (one existing migration audit ignored), all 15 shared
skinning tests, and the 229-species reusable model loading check. The fresh
native catalog independently reports 251 packed species, 229 exact bodies and
22 missing. This is progress toward the full goal, not all-species completion.
