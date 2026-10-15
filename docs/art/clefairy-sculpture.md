# Clefairy shared sculpture and skin

Clefairy's existing detailed sculpture now has an eleven-joint standard glTF
skin. Its sole canonical source is
`crates/crystal-voxel-view/models/actor_props/icon_clefairy.glb`.
The existing icon and explicit Clefairy battle/world species paths share that
same neutral asset. The former JSON was removed; no second species geometry
copy was introduced. `tools/clefairy_sculpt.py` remains the pure geometry recipe,
while `tools/clefairy_glb.py` supplies the skin and source-cue clips. Both the
sculpture CLI and `build-actor-props.py` now export the canonical GLB.

The original round pear body, pointed brown-tipped ears, rolled forelock and
supported curl, narrow eyes with reflections, rosy cheeks, delighted smile,
small fang, raised hands and six ivory claws, grounded feet and toenails,
six scalloped wing lobes and thick curled pink tail are preserved. The
[official Clefairy artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/035.png)
guides the original shape and color. No external geometry or game image is
embedded. Shininess retains the existing flag and pack palette on the same skin.

Before removing the JSON, its float32 neutral geometry digest was compared
with the new GLB recipe and matched exactly:
`0d173aea43c62408317aabf296a57c3a202a674df5464cd94b29fc04e872accc`.
Positions, normals, indices, part order and materials remain unchanged.
The canonical asset has 35 closed parts, 15,257 vertices, 24,216 triangles
and 715,428 bytes, with floor zero and authoring height 0.82. This verified
count replaces the stale higher triangle count in the earlier document.
Pack dimensions still own physical size.

Feet and toenails stay on the fixed root; the fitted face follows the body.
Hands, ears, paired scalloped wings and forelock have separate joints. Ear
tips share each ear's owner, the three lobes of each wing stay together, and
the forelock crease stays rigid with its volume. Two tail joints provide
curl follow-through. Attack braces the body and hands at 18% of the cue;
wings and curl follow at 28%. Idle surveys gently and hit recoils. Source
programs retain movement, audio, projectiles and timing ownership.

```sh
python3 tools/clefairy_glb.py
python3 tools/clefairy_sculpt.py --output target/clefairy-review
python3 tools/test_clefairy_glb.py
python3 tools/test_clefairy_sculpt.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view clefairy --lib
```

Seven stored-GLB checks and all eight existing sculpture checks pass. They
cover exact generation, bounded schema, closed topology and shading, anatomy,
actual-triangle face support and clearance, physical joins, forelock curl,
binds/weights, rigid appendage assemblies, fixed feet/toenails and deformation
through all clips, neutral endpoints and early brace/delayed curl response.
Native and runtime evidence is recorded in `docs/art/animated-models.md`.
Neutral review alone does not establish production animation, browser or
world acceptance.
Seven GLB checks and all eight existing sculpture checks pass. Focused canonical
routing and both duplicate-source rejection tests pass. The authoring CLI was
executed and its output compared byte-for-byte with the shipped GLB. Blender
is unavailable on this host: its batch route was updated and syntax-checked,
but the Blender process was not executed. The pure Python exporter and the
native client build were executed successfully.

The native catalog now reports 238 dedicated catalog bodies out of 251 packed
species, with 13 outside that catalog. Its wording was corrected: those 13
may have shared or fallback meshes rather than no model. Clefairy already had
a detailed shared sculpture; this step adds its rig and explicit catalog entry,
not a new geometry copy. Earlier catalog counts should be read in that narrower
sense, not as proof of complete visual fidelity or physical model absence.

The held native pair `target/pokemon-native-review/clefairy-3d.png` /
`clefairy-2d.png` was inspected. Receipt: `state_preserved=true`, hash
1910648216, LCD animation frame 37, virtual time 0.943902333 seconds. Brown ear
tips, rolled forelock, narrow face, raised hands/claws, grounded feet and
scalloped rear wing lobes read at source size. Tail curl and opposite wing are
partly occluded from this camera. Native pink is muted relative to the source
back sprite. Source-cue motion, browser and world review remain open; full
model/rigging acceptance is unfinished.

The new sharing test first used an incorrect crate-root actor module path,
which was corrected before the local test build. The GPU integration passed,
but the sharing fixture then caught an assumed battle-directory path for the
actor-space canonical GLB. That fixture now points to the actual actor asset.

Both Clefairy Rust integration checks pass: canonical icon/species neutral
sharing, planted feet/toenails throughout clips, and eleven-joint GPU skin
activation at 0.6096 m. The compiled executable passes all 28 active species
rig regressions (one existing audit ignored), all 24 skinning regressions and
the 238-species reusable model check. The broader actor suite initially found
an outdated unsupported-Pidgey assertion; it now recognizes Pidgey and Clefairy
as canonical while retaining unsupported-species guards. All 12 actor tests
pass on the fresh retry. Targeted whitespace checks pass.

There are 21 registered standard species skins. This does not prove all 251
species have improved rigs or accepted production visuals. The full goal
remains active, with remaining rigs and visual fidelity checks unfinished.
