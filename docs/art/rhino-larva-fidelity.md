# Rhyhorn, Rhydon and Larvitar volume and posture revision

The October 20 audit replaces Rhyhorn's thin dorsal fins and blank flanks,
Rhydon's raised palms and forward-sloping shins, and Larvitar's flat bill,
invented fingers and three upward tail leaves. This is another reviewed batch;
the full Pokémon fidelity goal remains open.

## Reference evidence

Eight-angle Battle Revolution model views were inspected for
[Rhyhorn](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/296503/),
[Rhydon](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339429/)
and [Larvitar](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339555/).
Both profiles, front/rear quarters and the rear establish the depth and joins.
Captured [Rhyhorn](https://pokemondb.net/sprites/rhyhorn),
[Rhydon](https://pokemondb.net/sprites/rhydon) and
[Larvitar](https://pokemondb.net/sprites/larvitar) Crystal front/back,
Black/White front/back idle frames and X/Y front views establish the stance
and surface details. The supplied pack's two actual front frames and back
picture were also inspected for every species.

Rhydon's extracted spread-arm pose is a bind pose. Its modeled resting arms
instead follow the captured idle's unequal lowered hands. Larvitar's extracted
texture puts a stray white eye island on the tail and renders the belly as
separate lobes. Captured sprites resolve the tail's green center and red
belly diamond. Those artifacts are not anatomical features.

The Crystal front/back pictures use the pack's purple/olive rhino palette,
also present in the captured Crystal pictures. This revision retains the
existing gray/cream/red 3D rhino materials and green Larvitar materials.
The pack comparisons establish shape and state, not exact RGB matching.
Physical heights are unchanged.

Reference meshes and textures are disposable external viewing inputs. The
original recipes do not read them. No downloaded geometry, textures, rig or
animation enters the authored GLBs or shipped bundle.

## Original corrections

Rhyhorn has a deep low tapered skull, fitted brow, slanted red gaze, triangular
ear openings, a backed mouth gap and two small rooted fangs. Four thick shields
wrap down both flanks, with angular projecting corners and actual peaked
crests. The first wrap candidate formed smooth hoops; a second still had flat
roofs along its length. Longitudinal crest taper fixes those defects. Its
short stocky pillars meet complete flat soles and twelve fitted blunt claws.
The small rear point, jaw, ears and all support pivots follow the new volume.

Rhydon has a larger skull, shorter thick pinnae and six cheek points over a
heavier upright barrel. Four cream bands sit lower on the belly. Its drill
projects forward and upward, while its shorter shins stand over complete heels.
The hands occupy different heights and depths below the shoulders; forward
and downward claws follow the palms. Arm weights follow each bent arm's arc
instead of using absolute horizontal distance. A narrower actual jaw gap has
occupied lining and tongue tissue behind it. Three restrained gray collars
follow the thick curved tail, and the dorsal points have real flattened depth.
All shoulder, wrist, ear, drill, head, tail and hind-limb pivots move with the
revised anatomy.

Larvitar has a fuller pear-shaped head, deeper back skull, shorter muzzle and
shorter crown relative to its head. Narrow fitted red eyes, long black tears,
small closed mouth and belly diamond follow the actual stored triangles.
Two short conical arms replace the invented palms and six fingers. Spatial
weights preserve useful independent wrist controls within those stubs. Short
feet and small toes support the compact lower body. A short thick stem ends
in six angular radial teeth around a solid depressed green hub. Rear-flank
black diamonds accompany the front vents. Pupitar is byte-identical.

## Stored assets and executed checks

| Model | Runtime / logical parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rhyhorn | 51 / 51 | 11,214 | 15,636 | 11 | 217 | 457,568 |
| Rhydon | 48 / 64 | 17,699 | 27,280 | 15 | 255 | 755,720 |
| Larvitar | 21 / 43 | 7,394 | 10,816 | 11 | 127 | 340,684 |

Authoring heights remain 1.11, 1.40 and 1.04. Production pack heights remain
0.9906 m, 1.905 m and 0.6096 m. Strict source/packed pins, readers and existing
model budgets remain enforced; material packing preserves every logical solid.

Full canonical GLB SHA-256 values:

```text
rhyhorn a6712d060cb60b45f4dcf1ac5dfbfce7208f8512bbd1f9bda053a6acf8f47666
rhydon a67c2e43318220af197d62275798e036dd26e7978f6c619cc0a9877e4461cbb7
larvitar 985b4cdb811c8584530b4d2d793b47eb9a17c5814df78b873cf2e7ee3f9bfecd
```

The 62 distinct focused Python checks cover closed outward geometry, normals,
fitted surfaces, occupied neutral/posed attachments, uncrossed curved anatomy,
exact recipe parity, logical packing, binds, control ownership, independent
articulation, fixed supports, clip endpoints, positive posed winding and bounded
strain. Ten independent fidelity checks measure flank wrap/depth, longitudinal
crest peaks, a backed mouth, hand posture, actual shin sections, heel support,
drill axis, current jaw clearance, skull/crown ratio, tapered arms and radial
tail distribution. The nine initial fidelity guards all reject the immutable
baseline; the later crest guard rejects flat longitudinal roofs. Mouth samples
derive from the actual revised lip surfaces and remain in the head carrier's
frame during articulation. No numerical tolerance was relaxed.

Blender's regular builder reproduces all three canonical GLBs byte for byte.
All eight angles of each final neutral orbit were inspected against the
baseline and game orbit at the same studio camera scale. Fixed-camera 45/90
views cover idle at 0.25, action at 0.34 and recoil at 0.18. These are sampled
poses, not acceptance of every move or animation frame.

```sh
PYTHONPATH=tools python3 -m unittest -v \
  test_rhyhorn_sculpt test_rhydon_sculpt test_rhino_glb \
  test_rock_pupa_sculpt test_rock_pupa_glb test_rhino_larva_fidelity
/Applications/Blender.app/Contents/MacOS/Blender --background \
  --factory-startup --threads 2 --python-exit-code 1 \
  --python tools/build-battle-species.py -- \
  target/rhino-larva-builder-review --only rhyhorn,rhydon,larvitar
```

## Runtime verification

Ten focused Rust checks pass: the four original-pack-size GPU selections for
the two rhinos and both rock pre-evolutions, rhino and rock-pupa skin ownership
and articulation, their two world-support checks, biped hip centering and the
full species schema. Rhydon and Larvitar are included in the hip-centering
regression. Rhinos and Larvitar preserve canonical upper-body bindings while
whole supports walk. The existing 64-phase floor/contact/culling checks pass.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run \
  --config profile.dev.package.crystal-voxel-view.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The exact ten test names are retained in the ignored gallery. Tests ran from
the final built unit executable, with exact revised compressed GLBs verified
inside it. Both native executables also contain gzip streams that decompress
to the exact current canonical assets. The native review retains executable
and canonical hashes so stale assets cannot establish visual acceptance.

## Production views and limits

Twelve selected native/classic pairs were inspected, including all 24 PNGs and
their preserved-state receipts. Every battle shows the requested authored model,
`source=[]` and `fallback=None`; the actual optional encounter trace reports
`accepted=true` with checked terrain anchors. Native executable hashes stayed
unchanged through the captures.

Real Route 29 encounters use `RALA` repeated 20 times and the existing production
move-menu hold. The first unpaced Larvitar run failed terrain-anchor admission
and timed out; it is excluded. The retained Larvitar run uses the existing
`CRYSTAL_RENDER_TEST_WALK_STEPS=1` keyboard pacing, waits through real movement
and passes admission. No admission predicate, controller or game state is
bypassed. This review does not resolve that fast-reviewer timing failure.

Field views use NewBarkTown start `(7,12)`, hour 12 and `RR`. Wide views use
camera `2 1 35 0`; closer views use `5 1 35 0` except Rhydon, which uses
`3 1 35 0` to keep its tail in frame. Static rear views use EcruteakCity hour 12,
rotation 4 and pitch 35: Rhyhorn `(20,25)` at zoom 3, Larvitar `(20,25)` at zoom 5,
and Rhydon `(20,27)` at zoom 1. The wider Rhydon view preserves its complete
original-size body and tail. The earlier `(20,24)` static fixture omitted the
large follower because its required third seed tile was occupied. The final
placement keeps the actual source corridor clear and avoids an NPC over the
tail tip. These are camera/placement choices, not changes to physical size.

Directly lit fronts/obliques show the fitted gazes, lower hands, full supports,
armor volume and short larva tail. These shape-review captures retained very
dark native back faces. The subsequent [outdoor sky-fill
revision](pokemon-lighting-fidelity.md) corrects that lighting defect with
matched before/after rear views. The studio orbit establishes rear geometry;
small Larvitar details remain distance-dependent.
Classic field views omit the added follower, and the battle move menu covers
the player's back picture. Those pairs establish matching state/presentation;
the separately inspected pack pictures and studio rear views establish the
back-sprite likeness.

Selected evidence is ignored at
`target/pokemon-fidelity-review-20261020-rhino-larva/index.html`: nine full
turnaround sheets, six sprite/pack sheets, three pose sheets, 24 native/classic
PNGs, twelve receipts and concise verification/asset-identity records. The
personal `game-model-fidelity-audit` skill now explicitly checks longitudinal
crest shape and records the actual optional admission-trace flag.

The other 273 canonical GLBs, including Pupitar, remain byte-identical to batch
start. Reference downloads and this task's scratch exports/captures are removed
after verification; shared packs, the task cache and selected ignored gallery
remain. Nothing is staged or committed. Browser, shiny, every-move and full
roster acceptance remain open.
