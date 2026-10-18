# Dratini, Dragonair and Dragonite skins

Canonical GLBs now replace the three static JSON exports. The normal background
builder, direct sculpture CLI, strict logical readers and Rust dispatch use the
same skins. Registration contains 232 skins and 43 static species files. The
full-roster model and rig improvement goal remains active.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Dratini | 14,722 / 24,092 | 10,976 / 18,524 | 17 / 13 | 7 | 85 | 502,844 |
| Dragonair | 18,065 / 30,492 | 14,274 / 24,676 | 21 / 17 | 7 | 96 | 585,304 |
| Dragonite | 40,536 / 61,996 | 19,917 / 31,312 | 60 / 34 | 15 | 184 | 833,400 |

The continuous serpent bodies retain their original sampling and curves.
Eye, forehead, pearl highlights and breast layers follow actual parent
triangles. Dragonite's smile and eight belly folds use bounded adaptive fitting
with maximum surface error 0.0006; direct clipping produced degenerate slivers.
Its body, head, limbs, antennae and claws use fewer sections. The tail retains
24 radial sections, six spline samples per link and its complete ventral ribbon.
Packing combines only coherent assemblies on the same controls: head markings,
belly folds, each side's wing ribs and complete hand/foot claws. Logical ranges
restore every original part in order. No geometry is discarded or shared mesh
limits increased. The 20k vertices, 40k triangles, 64 parts/joints, 256 accessors
and 1 MiB limits remain unchanged.

Dratini and Dragonair have seven controls: a weighted identity ground root,
upper neck, head, two ear fins and two tail hinges. Low coils remain fixed while
the neck leans, ears flex and tail turns. Dragonair's neck pearl follows its
head; both tail pearls follow the tail tip. The body and ivory ribbon derive
weights from the same actual sampled ring planes, radii and arc distances.

Dragonite has fifteen controls: ground root, body, head, two antennae, two
wings, two arms, two wrists, two legs and two tail hinges. All feet and foot
claws remain rooted in battle; complete palms and hand claws follow wrists.
Wing membranes, borders, spars and ribs share each wing's hinge. Its attack
raises both forearms and flares the wings. An initial wrist blend based on Y
stretched a forearm edge by 20.7%. The final blend follows the actual arm tube
rings and arc distances. It retains the stronger pose and the original 20%
strain limit. This fixes binding rather than weakening motion or validation.

Idle lasts 3.2 seconds with 81 keys. Attack/hit last one second with 65 keys,
source-cue timing, rotation-only tracks and neutral endpoints. Every non-root
control moves strongly weighted geometry independently. The existing world
Walk modes for the serpents and Fly mode for Dragonite remain. World bindings
match canonical indices and weights exactly. Dragonite's four wing beats per
idle cycle retain authored flight recognition. Grounded coils stay grounded;
the flying guardian keeps clearance through its global flight root.

Twenty-one checks pass against the final stored GLBs in 78.419 seconds: twelve
source anatomy checks and nine independently decoded skin checks. They cover
source and packing parity, hierarchy/inverse binds, budgets, closed outward
solids and corner normals, every independent control, fixed contacts, positive
winding and strain, neutral endpoints, occupied roots, supported fitted layers
and ventral ribbons, uncrossed posed bodies, and source/byte/species mutation
rejection. A displaced-ribbon negative control proves the support check fails
when the layer detaches.

Five focused Rust checks pass in 1.87 seconds: three GPU activations at original
pack dimensions, canonical contacts/faces/independent controls and complete world
locomotion bounds. Dratini and Dragonair use the existing authored body-length
reference `height * (2.24165 / 1.18)`, retaining pack dimensions 1.8034 m and
3.9878 m. Dragonite uses standing height and its original 2.2098 m dimension.
Authoring heights remain 1.18, 1.18 and 1.40. Uniform scale, complete animated
bounds, support, camera and geometry omission gates remain intact.

The ignored `target/pokemon-native-review/dragon-skin-review.png` was inspected
inline after the final binding fix. Rows name each species; columns compare
previous static, decoded optimized neutral, front, rear, side and decoded
attack at 0.34. The comparison shares fixed neutral fit bounds. Broad ivory
coils, all three Dragonair pearls, distinct ears/horns, Dragonite's friendly
face, antennae, teal wings, belly folds and twelve digits remain readable.
Attack views show neck leans, raised forearms and wing flares with fixed soles.
This authoring review does not by itself establish production acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only dratini,dragonair,dragonite
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_dragon_family_sculpt test_dragon_family_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Dratini | `1ba39a11cb3500c09cdac49d48156bdaa58811b71ed00d436398137c62d68b8a` | `53985a5d2655309199dcc14faf9c3e554d04e0fb273d8bdfcaa67b3cc64a75f7` | `b2c9414a8b51a8a5082a1dab1c94f2eab28f0380c3ca4b0f1288d210cacfaea5` |
| Dragonair | `d1025e2f13dc1785387b2f13a19ffb704a6eb3d617724aac1d49fd608f99cf3f` | `3effe7b64485d15be8a2b9d78c14ad80dd74dcab79c10b169f9145f13391efc2` | `7833111a97261f3585c894671cda8ec189c80362ff682158e235fadc97296d4a` |
| Dragonite | `519e2bdb4227b1705a81f67b1a2b632d5007433449243dd69f79aee2d16eac60` | `8fe63ca60d85a0dc97632a7f182a368a91d0f8fdba3fba2f9da92eb2d3524c3f` | `062412a2b4b5b2113754a578ea44ea231923885a700cf9ca8d6786468007d765` |

Native captures must set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
The production path disables Winit and macOS primary-window retention and uses
the schedule runner. No windows, focus changes, shiny checks, grass-height,
AA/filtering, audio, pack or gameplay changes belong to this batch.

Both current native examples built successfully in 63 seconds with
`location-tester,bevy/zstd_rust`. Binary preflight checks headless/mute tags and
the exact compressed current GLBs before captures. The captures retain original
physical dimensions and require paired authoritative state preservation.

The shared schema/physical-size check passes in 101.35 seconds and the shared
elapsed-idle/exact-current-attack cue check in 101.28 seconds. Together with the
21 stored Python and five focused Rust checks, 28 scoped checks pass. The normal
background builder, direct sculpture CLI and final candidates match canonical
GLB bytes exactly. No full-suite or complete gameplay claim follows from these
model checks.

Example offscreen world capture from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=DRAGONAIR \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/dragon-dragonair-world.png
```

## Native paired review

Nine current production 3D/2D pairs and three moving-world frames were inspected
inline, all offscreen and muted at 1000×700. Every pair preserves authoritative
state. Six front/world captures retain 32 actual synthetic-input movement rows
across five Route36 positions, (20,8) through (20,12). Named walking battles
admit complete bodies at original 1.8034/3.9878/2.2098 m dimensions, uniform scale
and zero support offset, with no fallback or relaxed support, camera, authentic
ground or omitted-geometry/shadow gate. The first battle audit omitted the
correct trace variable; it was recaptured with `CRYSTAL_ENCOUNTER_TRACE=1`.

The serpents retain their distinct noses, ear fins, continuous ivory undersides
and grounded curves. Dragonair's three pearls and horn remain attached through
motion. Dragonite retains the friendly face, antennae, teal membranes, complete
claws and segmented cream belly. Its source attack raises forearms and flares
wings; its world presentation retains the existing intentional Fly mode. Side
views can conceal Dratini's coil and far eye or Dragonair's tail pearls. The
underside of the head receives a strong dark shadow, and thin belly folds and
wing edges alias at distance. Broader lighting/AA work remains open.

Dragonite's initial world camera at zoom step 3 clipped the tail. The final
world review uses normal camera zoom step 1, retaining physical scale, FOV and
all bounds. The complete final pose fits the frame; its moving frame has normal
tree occlusion. Dragonair's moving frame overlaps the sign along the follower
path. This remains a broader follower/scenery placement issue; neither that
frame nor the wider Dragonite review establishes all-camera/world acceptance.
Browser, broader materials/lighting coverage and full gameplay remain open.

Final captures took 35–80 seconds each. Both executable identities stayed
unchanged throughout capture and recapture; exact current compressed GLBs were
checked in both binaries. Completed producer scripts, candidate/source copies,
CLI outputs and redundant intermediate frames were removed. Final raw/paired
images, receipts, six movement CSVs, three moving-world frames, metrics and logs
remain ignored under `target/pokemon-native-review`. No shiny checks, grass,
AA/filtering, pack, audio or gameplay changes belong to this batch. The full
model and rig goal remains active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Dratini front | 1880 / 420444217 | 1890 | 32.537831083 |
| Dratini world | 1469 / 4016297989 | 1480 | 25.66833525 |
| Dratini battle | 1966 / 3705006783 | 2587 | 44.646506668 |
| Dragonair front | 2348 / 2877616458 | 2357 | 40.583294084 |
| Dragonair world | 1778 / 2733506254 | 1785 | 30.500430167 |
| Dragonair battle | 3001 / 1301380982 | 3700 | 63.721003084 |
| Dragonite front | 2317 / 1010126641 | 2328 | 41.294861625 |
| Dragonite world | 1385 / 438106105 | 1392 | 23.714606708 |
| Dragonite battle | 1657 / 4108062778 | 2325 | 40.09267454 |

Stable executable SHA-256:
- `render_at_location`: `b9c2d18ad8aea6fa80391fbbac672bc41d6786962e176d5163cf39b979284113`.
- `immersive_battle_3d`: `26c43d0637aefbe73bc6ead9656ba415299f39f77d5cf45cee531b78da916074`.
