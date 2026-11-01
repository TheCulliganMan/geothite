# Sudowoodo dedicated sculpture and skin

Sudowoodo replaces its missing exact battle body with a dedicated original
sculpture and nine-joint standard glTF skin. The sole canonical runtime file
is `crates/crystal-voxel-view/models/battle_species/sudowoodo.glb`.
`tools/sudowoodo_sculpt.py` authors the geometry and `tools/sudowoodo_glb.py`
authors the skin and clips. No icon or legacy overworld geometry was copied.
Plain `sudowoodo` source routing remains the existing actor prop; an explicit
`species:SUDOWOODO:sudowoodo` identity uses the new canonical battle body.

Inspected [official Sudowoodo artwork](https://sg.portal-pokemon.com/pokedex/0185/)
guided the short forked crown, bowed branch arms, three green rounded fingers
on each hand, small fitted eyes, curved smile, five golden oval markings and
spread supporting roots. The matte brown trunk is continuous; its front
markings follow actual stored triangles rather than floating face planes.
The crown has rounded embedded cut surfaces. Its geometry has 33 named closed
parts, 9,089 vertices and 11,376 triangles. Floor minimum is exactly zero and
neutral height is 1.04 authoring units. The pack's dimension owns world scale.
The neutral digest is
`010b92720972b99c2bee10df2b6a074d605fce9fef79d4f8eef8c49b2e079275`.

The root owns both foot pads. Torso and head provide a restrained survey;
separate branch-arm and hand joints keep the six finger orbs attached while
the hands settle behind arm movement. Two crown joints preserve the fork.
Attack braces at 18% of the existing source move cue, with hands/crown settling
at 28%; hit recoils. Idle lasts 3.8 seconds and returns to neutral at its seam.
The clips contain rotation keys only. Game programs still own movement,
projectiles, sound, HP, events and timing.

```sh
python3 tools/sudowoodo_glb.py
python3 tools/test_sudowoodo_glb.py
cargo test --locked -p crystal-voxel-view species_rig --lib
cargo test --locked -p crystal-voxel-view battle_view::skinning::tests --lib
```

Seven independent stored-model tests and the canonical routing check pass.
They check exact regeneration and mutation rejection, closed connected outward
anatomy, normals/winding, distinct fingers/markings/crown, weights/binds,
rigid face and fingers, loop seams, fixed pads across clips/blends, floor
clearance, triangle strain and early visible branch response. All 17 active
Rust rig tests pass (one existing migration audit ignored), including the new
Sudowoodo registry sharing and planted pads test and dense pose bounds.

The actual stored neutral GLB was inspected at
`target/pokemon-sudowoodo-review/neutral/review-00.png`; front, oblique and rear
neutral/idle/attack/hit decoded poses were inspected at `posed-angles.png` in
that directory. The fork, six fingers, eyes, smile and golden markings remain
readable. Arms flex and fingers settle without detached orbs; both pads stay
fixed. The broad simple brown surfaces and tiny smile/glints still need
production camera/lighting critique. These model views do not establish GPU
animation or browser acceptance.

The initial native pair was inspected at
`target/pokemon-native-review/sudowoodo-3d.png` and `sudowoodo-2d.png`. Its
receipt reports `state_preserved=true`, checksum hash 3004747631 and LCD frame
34. The source battle, levels, HP and menu remain held. Both models have
attached forks/fingers, grounded pads and readable faces. The brown shader
is darker than the flat source sprite; the gold markings follow the official
artwork rather than the sprite's white. In `gyarados-sudowoodo.png`, the new
opponent is a readable 3D body at the pack's dimension rather than a tiny flat
sprite. No scene size clamp was added.

The initial shiny captures `sudowoodo-shiny-first.png` and
`sudowoodo-shiny-later.png` correctly show green trunk and red fingers, but the
markings disappear into the trunk. Linear palette matching assigned the
saturated gold to the pack's brown anchor. The authored gold has been lightened
to a cream-gold highlight to preserve its contrast in both forms. Binary
comparison proves every vertex, normal, index, weight, bind and animation key
is unchanged: only one material and identity metadata differ. The seven model
checks pass again. Fresh GPU captures of that correction are pending the
rebuild. The previous build began before export completed and embedded stale
color data; it was stopped, and the clean-input build is now active.

All 13 battle skinning tests and the three Sudowoodo integration tests pass,
covering exact visible source sharing, rooted contacts, GPU skin activation,
physical scale and shared bind resources. The whole-registry loaded-model
check passes across 227 species. These runtime checks preceded the material
correction; source geometry and keys remain identical, but fresh render proof
is required for its final palette.

The completed initial reviewer's actual fresh-pack catalog reports 251 packed
species, 227 exact bodies and 24 missing bodies. Browser review, source-cue
GPU attack/hit inspection, Route36 overworld presentation and remaining
artwork refinement are still open. The full appearance/rigging goal stays active.
