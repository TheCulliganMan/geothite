# Machop family: volume, posture and depth revision

Machop now has large vertical red eyes, a broad shallow muzzle and closed
smile, visible rib relief and long swept cranial plates. Its fists sit in a
compact raised guard. Machoke has a projecting muzzle and open mouth, heavier
shoulders and chest, low relaxed arms and long fingers. Machamp has a broad
recessed mouth with thick lips and a tongue; its upper fists rise beside the
head while the lower arms bend toward the waist. All three retain upright
supporting shafts, with toe projection formed in the feet.

The original procedural construction remains in `tools/machop_sculpt.py`.
No reference mesh, texture or rig is an authoring/export input. Full eight-angle
Battle Revolution volumes were inspected for
[Machop](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/290614/),
[Machoke](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/290615/)
and [Machamp](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339388/).
The captured front/back BW animation samples, Crystal pictures and X/Y front
pictures were inspected through their
[Machop](https://pokemondb.net/sprites/machop),
[Machoke](https://pokemondb.net/sprites/machoke) and
[Machamp](https://pokemondb.net/sprites/machamp) galleries. The actual completed
local pack's first two front frames and back pictures were also inspected.
The downloaded neutral Machop/Machamp arm and foot bind poses do not establish
battle posture; the authored stance follows captured game idle and Crystal
poses instead.

The torso, neck, head and pectoral/back relief form a continuous surface.
Machop's rib crests and troughs have actual sampled geometry. Mature mouths
cut into that surface, with fitted dark linings, occupied jaws and tongues.
Machoke has four small fangs. Eye layers are clipped to the actual stored
surface triangles, including the curved anterior Machamp skull. Each crest
has thickness, a tapered tip and enough longitudinal depth to read in profile.

One solid foot per side carries three juvenile or five mature toe lobes. Its
broad flat sole replaces the separate round pads. Concave toe caps use center
fans that cover each cap once. The short shaft above each foot remains nearly
vertical. Hands have beveled palms, curled juvenile or Machamp fingers, and
relaxed mature Machoke fingers. The mature hands have four fingers and a thumb;
Machop has three fingers and a thumb. Machoke's three upper and three lower
red arm strokes follow each arm's actual surface. The mature dark belts carry
raised gold studs and oval championship buckles with authored red P emblems.

`tools/fighting_glb.py` uses the same authored arm paths for shoulder, elbow and
wrist pivots. Machamp retains four independent arm chains. Fitted eye/mouth/
crest details share the head carrier; each hand shares its own wrist. The
continuous torso blends pelvis, chest, neck and head. Foot support remains
fixed in battle, and source cues retain ownership of idle/attack/hit timing.
The juvenile tail has its own joint.

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Machop | 32 | 11,179 | 16,304 | 13 | 166 | 495,092 |
| Machoke | 55 | 14,694 | 21,812 | 12 | 255 | 646,236 |
| Machamp | 53 | 16,806 | 24,648 | 18 | 248 | 723,600 |

Authoring heights remain 0.85, 1.15 and 1.45. Production pack heights remain
0.7874 m, 1.4986 m and 1.6002 m. Existing limits remain 64 parts, 20,000
vertices, 120,000 indices, 256 accessors and 1 MiB per GLB. Source geometry pins:

- Machop: `8ecc708258f7d11379cbba584594d05d59785a933c7efef426c0533c48d6c362`
- Machoke: `899ca475595bcf10908fa43abaec72a3814825e5b33ac57f1821c6377f70bcb1`
- Machamp: `71e409803fa06474215ea3bac8c59a53af7199737a3895d11564c03929a0df6d`

Nineteen focused Python checks pass against the exact stored assets, including
closed outward connected geometry, normals, source parity, budgets, binds,
owners, occupied roots, fully buried shoulder/hip caps, fitted gaze and red
markings through motion, garment coverage, floor contact, strain, winding and
clip endpoints. Additional independent checks reject crossed faces in the
major volumes and bent elbows. The elbow radii and paths were refined until
those checks passed while retaining the distinct raised and low hand carriage.

The new anatomy guards reject the prior forward-sloping shafts, short crests,
shared hand posture and absent rib relief. Replacing the revised mature skull
with its original uncut body also fails the mouth guard while the current lips
and lining remain present. Mouth probes derive the gap from the moving lips,
jaw and tongue in the head carrier's frame, require a real recess behind
adjacent skull tissue, and sample empty space in front of the actual backing.
They cannot pass from testing a former world-space mouth position in empty air
or projecting a rim in front of an uncut face.

All eight final neutral angles were inspected for every species. Both mature
models also received eight-angle underside reviews. The actual stored skin was
sampled at idle 0.25, attack 0.34 and hit 0.18 at 45 and 90 degrees, with fixed
studio height and framing. The immutable prior GLBs supplied matching before
orbits. The audit skill now explicitly covers bent-limb self-intersections,
concave sole caps and backing-relative mouth probes; its validator passes.

Seven focused Rust checks pass after verifying the exact compressed current
GLBs in the test executable: three GPU activation checks at verified pack
heights, independent arm/owner and complete leg support checks, the full species
schema and hip-centered biped steps. The 64-phase walking check also requires
upper-body, face, crest, garment and arm weights to remain unchanged when the
production walking support chains are built.

The unit executable was rebuilt with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run
```

The executable ran the `fighting_family`,
`species_schema_retains_anatomy_skin_order_and_physical_size` and
`pokemon_motion_biped_steps_are_centered_under_hips_not_toes` filters with the
completed external TUI pack and headless image capture enabled.

Only these three GLBs changed among the 276 canonical variants in this
revision; the other 273 are byte-identical. The retained ignored review is
`target/pokemon-fidelity-review-20261020-fighting/index.html`.

Both current production examples were rebuilt with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The compressed current GLBs were verified in each executable, including
decompression equality with canonical bytes, before running captures. Both
examples consume the existing ignored
`content-packs/core-modular.browser.crystalpack`; no content pack was rebuilt.
Captures set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, and `CRYSTAL_PREVIEW_MUTE=1`.
Each battle used a witnessed Route 29 walking encounter with ordinary `RALA`
inputs repeated twenty times, `CRYSTAL_RENDER_TEST_HOLD_BATTLE=0` and
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`. New Bark walking started at `(7,12)` at
hour 12 with ordinary `RR` inputs, both hold flags off, and supported cameras
`2 1 35 0` and `5 1 35 0`. Pack sizes and normal scene-admission gates remained
active.

All eighteen final native/classic PNGs were individually inspected. Nine
paired-state receipts preserve authoritative checksum, LCD frame and virtual
time across their corresponding 2D/3D views:

| Species / view | Frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Machop / battle | 663 | 2796127711 | 1316 | 23.244671751 |
| Machop / walking | 719 | 2964229825 | 789 | 15.505524624 |
| Machop / closer walking | 822 | 957659421 | 890 | 17.616722751 |
| Machoke / battle | 454 | 1977255057 | 1357 | 25.745342208 |
| Machoke / walking | 967 | 1941897473 | 1046 | 20.040390334 |
| Machoke / closer walking | 1293 | 3132542066 | 1392 | 27.271992083 |
| Machamp / battle | 1768 | 152903144 | 2410 | 42.519043750 |
| Machamp / walking | 513 | 36601132 | 548 | 9.788875209 |
| Machamp / closer walking | 396 | 1785830225 | 418 | 7.897784208 |

Battle receipts identify the requested modeled species and no fallback. Field
receipts identify the requested species as `PartyFollower`. Heads, mouths,
crests, shoulder and arm shapes remain readable; the closer field views show
the complete feet clearly, and Machamp's four arm silhouettes remain distinct.
A tree partly overlaps one raised fist in the wider Machamp field view; the
closer view provides the unobstructed review. The classic move menu hides the
player's back picture, and the classic field omits the added follower. Those
views prove state correspondence; the separately inspected pack pictures and
full orbits establish front/back likeness. Different runs can end at different
interpolated positions; each individual pair preserves its own exact state.

Selected images and receipts remain in the ignored gallery. Reference-only
archives, textures and meshes, draft exports and temporary verification output
were removed after review. Shared packs and Cargo caches were preserved.
Sampled views do not establish every animation, shiny variant, browser camera
or acceptance of the rest of the roster. Nothing was staged or committed.
