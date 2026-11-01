# Pikachu dedicated sculpture and skin

Pikachu replaces its missing exact battle body with an original golden mouse
sculpture and eleven-joint standard glTF skin. The sole canonical source is
`crates/crystal-voxel-view/models/battle_species/pikachu.glb`.
`tools/pikachu_sculpt.py` authors its anatomy and `tools/pikachu_glb.py` authors
skin and source-cue clips. No icon, surf mount or legacy prop was copied into
the new body. Explicit species identity shares the same neutral geometry in
world presentation. Legacy prop identities retain their existing assets.

The inspected [official Pikachu artwork](https://sg.portal-pokemon.com/pokedex/0025/)
guides the full mouse torso, cheeked head, round eyes and white glints, tiny
nose/smile, red electric pouches, asymmetric long black-tipped ears, five digits
on each paw, long grounded foot pads, two brown back bands, brown tail stem and
flat squared lightning bolt. The first review caught a narrow torso beside an
oversized tail; widening the body and narrowing the bolt corrected that ratio
before runtime integration. All surfaces are closed and matte. Shininess uses
the existing flag and source-pack palette on this same body and rig.

The GLB has 40 named parts, 11,568 vertices and 13,876 triangles. Floor minimum
is zero and neutral height is 1.00 authoring unit; source-pack dimensions own
physical size. Neutral identity digest:
`4bdd281a376a6427e9ee966c69b650d349367033a58f70614987a2e8b1065387`.
The torso and head survey subtly; separate paw joints brace early, two joints
per ear preserve the shared gold/black seam, and two tail joints provide delayed
bolt follow-through. Both pads and toe creases stay rooted. Idle lasts 3.6
seconds. Attack braces at 18% of the existing source cue, with ears/tail following
at 28%; hit recoils. Keys contain rotations only. Game programs still own
movement, projectiles, audio, damage and timing.

```sh
python3 tools/pikachu_glb.py
python3 tools/test_pikachu_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view pikachu --lib
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view species_rig --lib
```

Seven independent stored-model checks pass, covering canonical generation and
mutation rejection, bounded storage, closed outward anatomy, normals, distinct
features, weights/binds, rigid face, fixed pads, deformation/floor clearance,
neutral endpoints, early brace, delayed tail and coincident gold/black ear seams
through all clips. Native review and runtime regression results are recorded
in `docs/art/animated-models.md`; software neutral review alone does not prove
native source-cue animation or browser acceptance.

Focused source routing and both Cargo Pikachu integration tests pass, proving
canonical neutral sharing, fixed mouse pads and eleven-joint GPU skin activation
at source size (0.4064 m). The freshly compiled Cargo test executable in
`target/pokemon-models/debug/deps/` passes all 20 active rig tests (one existing
migration audit ignored), all 16 skinning tests and the full 230-species model
loading check. The rebuilt native client passes and independently reports
230 exact bodies out of 251 packed species.

The actual held native 3D/2D pair was inspected at
`target/pokemon-native-review/pikachu-3d.png` / `pikachu-2d.png`; its receipt
confirms unchanged battle state (hash 3247630517, LCD frame 60). The face,
red cheek pouches, black ear tips and paws read clearly at the pack's small
physical size beside Sudowoodo. The flat bolt tail overlaps the head/ears in
this camera view and remains a readability limit. The source view is the
back sprite. Front/rear neutral and decoded idle/attack/hit poses were also
inspected in `pikachu-poses.png`. Browser, overworld and native source-cue
attack/hit verification remain open; this is not full visual acceptance.
