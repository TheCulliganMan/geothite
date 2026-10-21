# Graveler and Golem reference and posture revision

The October 20 review reopened the earlier neutral-angle acceptance. The old
Graveler looked like a smooth egg with small clasped lower fists. Golem held
both hands above its shoulders and carried an oversized round head high on a
regular tiled shell. Those poses survived numerical skin checks but did not
resemble the reference volumes.

Reference-only Battle Revolution turnarounds were inspected at 0, 45, 90, 135,
180, 225, 270 and 315 degrees for [Graveler](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339395/)
and [Golem](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339396/).
Four frames apiece from the captured X/Y [front](https://www.pkparaiso.com/xy/sprites_pokemon.php)
and [back](https://www.pkparaiso.com/xy/sprites_pokemon_espalda.php) idle animations
provided a separate stance check. The extracted neutral pose and X/Y idle are
different poses; the revision uses their anatomical and support cues rather
than treating an extracted arm spread as the required battle idle.

All shipped geometry remains original procedural work. Game meshes and textures
were used only in a disposable external reference directory. They are not
inputs to either authoring recipe or runtime GLB.

`tools/graveler_sculpt.py` now authors a broad uneven boulder, angular arms and
beveled stone palms. Lower forearms extend beside the belly, and bent fingers
point forward and down. Shorter flat supports, a continuous stone-colored brow,
narrower dark grin and larger seated rear stones replace the toy-like volumes.
The four shoulder/wrist chains move with their revised anatomy.

`tools/golem_sculpt.py` carries a smaller, flatter skull low in front of the
shell. Its mandible closes about the jaw hinge while keeping a real mouth gap,
tongue and teeth. Lowered elbows and forward/downward claws replace the raised
palms. A continuous angular warp preserves the shell's closed cell graph while
uneven raised plate centers change its silhouette through the full orbit.
Skull, jaw, shoulder, elbow and wrist pivots follow the revised volumes.

Canonical physical reference heights remain 1.10 and 1.25; actual production
sizes come from the pack. Loader limits remain 20,000 vertices, 40,000 triangles,
64 packed parts, 256 accessors and one MiB per GLB. The current assets contain:

| Model | Vertices | Triangles | Packed parts | Joints | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Graveler | 8,670 | 7,928 | 22 | 10 | 370,312 |
| Golem | 19,722 | 30,140 | 29 | 12 | 809,688 |

Before/after studio views use shared orthographic camera scales, eight angles
and identical lighting. Additional stored-skin reviews sample idle 0.25, attack
0.34 and hit 0.18 at quarter and profile angles. These are shape and sampled
pose evidence; they do not establish every animation, shiny variant, camera,
browser or the quality of the remaining roster.

The posture guards reject both immutable old GLBs. Golem's aperture samples
now sit inside the revised gape and travel with the skull during animation;
sampling a former mouth position in empty world space would give false proof.
Occupied roots, face/plate clearance, closed volumes, normals, floor contacts,
skin strain and strict canonical byte/source parity remain separately checked.

The broader fresh overview inspected one quarter view of all 276 canonical
variants. It is triage only. Ampharos still looks too slender, Granbull retains
stiff spread arms, Mewtwo has weak limb/body transitions, and the Machop family
still needs a reference-based stance review. These observations reopen work;
they are not full-turnaround audits or blanket roster acceptance.

Thirty-three focused Python checks pass against the final stored GLBs, including
the moving aperture and vertical jaw-boundary guards. The existing geometry and
storage validators also pass for both assets. Five Rust checks pass: both GPU
instances at actual pack size, the stone-pair support/control regression,
two-support walking, and the complete species schema. The test executable was
verified to contain both current compressed model payloads before execution.
The other 274 canonical GLBs stayed byte-identical during this revision.

Both production reviewer examples built successfully. Their exact embedded
Graveler/Golem payloads were checked against the current canonical files before
capture. Headless muted Route 29 walking encounters reached the real move menu
with the requested models and no fallback. New Bark walking captures show both
followers at their real 0.9906 m and 1.397 m pack sizes. All four paired-state
receipts report `state_preserved=true`; each 3D image and its classic pair were
individually inspected. Golem's lower head, claws and supported stance remain
readable in the battle view; Graveler's profile and field quarter view retain
their distinct upper/lower arms and fitted gaze. The classic move menu covers
the player's back picture, and the classic field omits the added follower.
Those pairs prove matching state; the separately inspected game back animation
and full studio orbit provide the rear likeness evidence.

The executed native build was:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The capture driver used these inputs for each of `GRAVELER` and `GOLEM`:

```sh
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack"
export CRYSTAL_RENDER_TEST_HEADLESS=1
export CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image
export CRYSTAL_PREVIEW_MUTE=1
export CRYSTAL_RENDER_TEST_PARTY_SPECIES=GRAVELER
export CRYSTAL_RENDER_TEST_PARTY_SHINY=0
export CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic
export CRYSTAL_RENDER_TEST_HOLD_BATTLE=0
export CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1
target/pokemon-models/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --walking-encounter \
  --walk "$(python3 -c 'print("RALA"*20)')" --size 1000x700 \
  --screenshot target/graveler-battle-3d.png --second target/graveler-battle-2d.png
CRYSTAL_RENDER_TEST_CAMERA="2 1 35 0" \
  target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map NewBarkTown --x 7 --y 12 --view both --hour 12 --walk RR \
  --screenshot target/graveler-world.png
```

The retained ignored gallery is
`target/pokemon-fidelity-review-20261020-stone-pair/index.html`. It includes
labeled authored/game eight-angle sheets, matched before/after views, sampled
motion, native/classic pairs and the one-angle roster overview. Disposable
archives, meshes, textures, drafts and raw capture output were removed after
verification. Packs and audio were not changed; no files were staged or committed.
