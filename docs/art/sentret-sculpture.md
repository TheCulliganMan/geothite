# Sentret sculpture and standard skin

Sentret fills a battle-catalogue body gap with one original canonical GLB at
`models/battle_species/sentret.glb` under `crates/crystal-voxel-view`.
`tools/sentret_sculpt.py` authors its pear-shaped body, long ears, raised paws,
small face, suspended hindfeet and curled tail. The white belly marking is a
closed annulus fitted to the actual body surface, leaving the brown center
visible. Tail bands use the same centerline and transported ring frames as the
parent tail. The terminal tail cap was rounded after the first front geometry
review showed an abrupt flat end. A first native paired review then showed
pointed ear tips; those shells were replaced with round-tipped ellipsoids. The
rounded-ear stored anatomy/skin suite passes. A rebuilt native paired battle
view shows the round ear tips and readable belly ring, paws, feet and tail.

The verified local pack supplies height digits 207, or 0.7874 m (2′7″). Normal,
front and back palettes agree. RGB555 `(22,15,10)` maps to RGB8 `(181,123,82)`
and `(15,7,4)` maps to `(123,57,33)` by runtime bit replication. White and black
complete the authored palette. Temporary reference sprites are QA inputs only.

The final asset contains 38 named parts, 9,293 normal-split vertices, 16,152
triangles, 13 joints, 181 accessors and 443,440 bytes. Authored reference height
remains 1.0; no reader limit is widened. Neutral digest:
`0a15f7e364cdec1b70a33e5e68df64620b7824223129e72dc9184606b8ce4485`.

The lowest tail support retains the identity floor root. One tail chain carries
the upper body; a second follows the curled free tip. Both painted bands and
parent tube use the same closest-centerline weight field. The complete fitted
face and belly annulus have the body owner. Ears, forepaws and hindfeet have
separate bones and follow the tail-supported body. Idle lasts 3.4 seconds with
85 keys; attack/hit last one second with 65 keys. All clips use rotations,
identity endpoints and the existing source battle cue clock, with an early
brace and trailing ears, paws and free tail tip.

Eight stored anatomy/skin checks and two routing/catalogue checks pass. They
cover topology, normals, the open annulus center, palette conversion, actual
stored tail cross sections, bindings, pinned support, floor clearance and
winding/strain. The stored-asset suite reproduces the canonical export byte for byte. The
pure authoring CLI was executed before the final ear correction. The Blender batch branch was syntax checked but not fully run.

Before the ear correction, fresh broad Rust suites passed: 36 rig checks with
one existing ignored check, 32 skin checks and 13 actor checks. After the ear
correction, both focused Sentret registry/GPU-skin regressions and the full
33-species anatomy/skin/physical-size schema check pass. The stored
support-contact test tolerates floating-point roundoff below 1e-12; the largest
observed residual was about 3e-17, while the floor clearance guard remains 1e-7.

The rebuilt native paired battle view at 1000×700 preserves state, checksum
3743123137, LCD animation frame 95, virtual time 2.785530208. The oval ear tips
now match the rounded source silhouette. The white belly annulus remains clear
at the native side angle, with the brown center intact. Raised paws, airborne
feet and the rounded striped tail support are visible. The default supported
wild fixture keeps Sudowoodo opposite Sentret. Browser rendering, more camera
angles and detailed source-cue motion acceptance remain open.

The first build encountered a shared scenery initializer missing
`hall_terminal_pixels`; the concurrent intended remapping was already present
when inspected and was preserved. No scenery edits or pack/audio changes were
made for Sentret.

```sh
python3 tools/sentret_glb.py
PYTHONPATH=tools python3 -m unittest test_sentret_glb -v
PYTHONPATH=tools python3 -m unittest \
  test_battle_model_assets.CanonicalSources.test_sentret_resolves_one_new_canonical_skin \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

The catalogue now declares 242 bodies and 33 standard skins. Nine other
battle-body gaps and the remaining model/rig/visual requirements stay within
the full all-Pokemon goal.
