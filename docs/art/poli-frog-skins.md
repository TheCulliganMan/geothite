# Poliwhirl, Poliwrath and Politoed sculptures and skins

Three generic low-detail frog prototypes are replaced by original distinct
pure sculptures and canonical GLB skins. `tools/poli_frog_sculpt.py` owns the
anatomy and `tools/poli_frog_glb.py` owns deterministic skin/clip export. The
normal builder, strict readers and Rust registry now use these assets; old
static JSONs are removed and the retired generic Blender branch rejects use.
Poliwag's existing sculpture and rig remain unchanged.

| Species | Logical / runtime parts | Vertices | Triangles | Joints | Accessors | GLB bytes | Reference height |
| --- | --- | --- | --- | --- | --- | --- | --- |
| poliwhirl | 25 / 25 | 11,464 | 20,648 | 12 | 134 | 457,120 | 1.10 |
| poliwrath | 27 / 27 | 11,964 | 21,420 | 12 | 145 | 485,652 | 1.20 |
| politoed | 36 / 32 | 15,317 | 27,556 | 15 | 163 | 601,864 | 1.20 |

The retired prototypes used 1,774/1,814/1,974 vertices and 3,472/3,544/3,864
triangles. New geometry adds the missing anatomy and smooth surfaces while
retaining all existing limits: 20,000 vertices, 40,000 triangles, 64 parts,
64 joints, 256 accessors and 1 MiB. Physical placement uses the loaded pack's
dimension and the model reference; authoring height is not a size clamp.

Poliwhirl has a rounded blue orb, raised round eye turrets, fitted ivory eyes
and vertical pupils, a fitted ivory belly and continuous dark spiral, bent
blue arms, padded ivory gloves with rooted thumbs and fitted knuckle seams,
and separate hindquarters, ankles and broad grounded pads. Poliwrath has a
heavier darker body, a raised boxing guard, larger arms and gloves, wider
planted feet, and actual fitted angular eyes
with raised severe blue lids. Politoed has its own green pear torso and
separate broad head, yellow/green spiral belly, rosy cheeks, fitted smile,
three rounded fingers per palm, three planted web digits per foot and a
continuous rooted curled crest. The chest/face fields fit actual stored
parent triangles; the spiral penetrates the belly along its entire length.

Both blue frogs have twelve controls: floor root, body, two eye turrets,
three controls per arm and two supported legs. Politoed has fifteen, adding
an independent head and two controls along the crest. Feet and complete toes
stay fixed in canonical battle clips; ankles blend into their leg controls.
Every belly/spiral vertex shares the body carrier. Each eye, pupil and lid
shares its own turret; gloves, thumbs and creases share their wrist.

Idle lasts 3.2 s with 81 keys; attack/hit last 1 s with 65 keys and neutral
endpoints. Source cues retain timing ownership. Poliwrath drives both gloves
forward, Poliwhirl makes a smaller punch and Politoed lifts its hands. At
attack progress 0.31, maximum forward displacement is about 0.112/0.330/0.072
authoring units, respectively. Separate regressions require both hands to
advance, preventing a token bone movement from satisfying the attack check.

Arm weights follow projection onto the authored limb centerline, with a broad
upper-arm/forearm blend and a supported wrist transition. Poliwrath uses a
broad five-control-point arm curve and arc-length weights around its elbow;
its raised gloves no longer mask the lower legs in neutral stance. The original simple
Y bands stretched under stronger punches; replacing those bands preserves the
existing strain threshold instead of weakening the motion. Shoulder roots
are completely buried in the torso, including all stored twenty-sided root
cap vertices throughout sampled clips. The early software review exposed
flat cutoff shoulders; corrected roots and the dedicated cap-burial check
prevent them returning. Politoed's toe assemblies are packed separately per
leg, preserving every logical source-solid range and order. World walking
therefore assigns each complete toe set to its own actual support tree.

## Verification

All thirteen Python anatomy/skin checks pass in 22.855 s. They inspect actual
stored closed/outward topology, shading, limits and geometry pins, hierarchy,
inverse binds, distinct anatomy, occupied-volume roots, uncrossed appendages,
all 161 spiral rings, fitted cap vertices/edges/centres against actual posed
parents, planted feet, clip strain/winding/endpoints, independent controls,
complete buried arm caps, per-leg toe packing, strict mutation rejection and
both-hand attack reach. No topology, area, fitting or strain tolerance is
relaxed.

Five focused Rust checks pass in 1.72 s: each skin activates GPU articulation
at the expected original size, the canonical parser retains source geometry,
support/floor/bounds and independent controls, and each world rig has two
proper support trees with fixed foot/toe ownership over 64 gait phases.
The current all-species schema and exact battle-cue GPU timing checks pass
against the final current assets in 77.02 s and 76.01 s, respectively. Twenty unique targeted checks pass in total. This does not
claim the complete gameplay test suite passes.

Background Blender reproduces all final GLBs exactly through the normal
builder, with the generic branch retired. Source comparison, neutral front
and independently decoded attack views were inspected inline. Source and
packed float32 geometry pins:

| Species | Logical geometry SHA-256 | Packed geometry SHA-256 |
| --- | --- | --- |
| poliwhirl | `331c07cd646e2dd21412c2243507420d69862a69a0acaac3086eb3cafbbe4394` | `331c07cd646e2dd21412c2243507420d69862a69a0acaac3086eb3cafbbe4394` |
| poliwrath | `3628cbf3a335a904aebdfa48f9ecc48d68033f9449107546b6fe256e7364d09e` | `3628cbf3a335a904aebdfa48f9ecc48d68033f9449107546b6fe256e7364d09e` |
| politoed | `dce0949835912403468cae4938d451fe36f18b0393a754c5a1108aab46df69ab` | `8d04df99bf27b790d08bd554c5cbc4762bcc345acd6cba1fea600867cc5c8c44` |

Canonical GLB SHA-256:

- poliwhirl: `ef8092cdb35a462f80101ee0413720d7f633ce0966c29a5f9a2efe5384a6044f`.
- poliwrath: `ade93529b3eedf8a5f5c9faf762ec49b1696424d0c9efd136263243de470ead8`.
- politoed: `6fe7447ab4a3950f661307612631ee26063d4d94c973e5d491b36e915171e668`.

Reproduction commands:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v test_poli_frog_glb
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view poli_frog --lib -j1 \
  --features bevy/webgl2,bevy/bevy_winit,bevy/x11,bevy/bevy_gilrs,bevy/zstd_rust,crystal-assets/test-fixtures \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
```

Browser, broader world/camera/scenery and moving-animation acceptance remain
open. The existing Surf save/reorder fixture guard failure and other unrelated
project-wide issues are not changed or reported as passing here. No shiny
verification, grass, AA/filtering, gameplay, audio or pack changes are included.

## Final production review

The final native build completed in 54.34 s. Nine 1000×700 production captures
were compared inline with their same-state 2D partners: ordinary first-person,
world orbit and battle for each species. Three actual moving world frames were
also inspected. All renders used offscreen image targets and muted audio:
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`
and `CRYSTAL_PREVIEW_MUTE=1`. No visible application window, browser tab or
focus change is needed. No shiny checks were run.

World captures use Route36 (20,8), hour 12, synthetic walking D; front camera
`0 0 20 1`, world camera `3 2 35 0`. Each of the six movement CSVs contains
32 samples and five actual authoritative positions from (20,8) to (20,12).
Battle captures use the Route29 walking encounter, held production battle,
`--walk RALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALA`,
and the external `content-packs/core-modular.browser.crystalpack`.

Every paired receipt records `state_preserved=true`. Last terrain admissions
accept complete bodies with no fallback and zero support offset. The stage
scale/reference calculations retain 0.9906 m Poliwhirl, 1.2954 m Poliwrath and
1.0922 m Politoed. Opponents in these final scenes are PIDGEY, RATTATA and
SENTRET respectively; both production HUDs name their actual species.
No model shrinking, support adjustment or relaxed admission gate is involved.

| Capture | Authoritative frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| poliwhirl front | 1715 / 3802917928 | 1721 | 29.225916834 |
| poliwhirl world | 1572 / 2265959886 | 1574 | 26.80766075 |
| poliwhirl battle | 2327 / 2492373290 | 2939 | 50.335904415 |
| poliwrath front | 1622 / 3528868880 | 1627 | 27.649879125 |
| poliwrath world | 1391 / 723995193 | 1397 | 23.757424375 |
| poliwrath battle | 1480 / 1177713701 | 2149 | 36.782044292 |
| politoed front | 1607 / 2415119787 | 1613 | 27.359619125 |
| politoed world | 1845 / 1464947921 | 1852 | 31.898265376 |
| politoed battle | 1745 / 3351836796 | 2312 | 39.662419959 |

Both executables were checked before launch for headless/mute support and
exact current compressed GLB payloads, decoded equal to the canonical assets.
Their streamed hashes, byte sizes and modification timestamps remained stable
through the final captures:

| Executable | SHA-256 | Bytes | mtime ns |
| --- | --- | --- | --- |
| render_at_location | `834ed97260516f4fc2a4308eddcb6ccc8e13a31797ca7817320fe84a6bbb5f69` | 408415936 | 1792299859103866502 |
| immersive_battle_3d | `1d42452c924becc817dfc8a6f9eec67d8ede1e2267cc1122f7bbd11f26b33250` | 408181416 | 1792299860049482875 |

Poliwhirl reads as a friendly round frog with clear turrets, white gloves and
continuous spiral. Poliwrath has distinct severe eyes and heavier proportions;
its revised raised guard leaves the legs visible in front and battle views.
Politoed retains a separate head, visible smile/cheeks, curled crest, fingers
and complete web digits. All three fit the reviewed ordinary camera at their
original physical sizes. Profile views naturally reduce eye/belly readability.
Foreground foliage overlaps the blue frogs' left gloves in the front scene;
Poliwrath's moving-world frame also approaches a sign. Those broader scenery
and camera issues remain open, rather than being hidden by resizing the models.

Final ignored evidence lives under `target/pokemon-native-review/poli-*`:
full-resolution `*-pair.png` comparisons, raw images, paired-state receipts,
world movement CSVs, selected world moving frames, final build/test logs and
executable identity. The software source/neutral/attack comparison is
`poli-software.png`. Temporary producers and duplicate intermediate exports
were removed after review. This is scoped model verification, not a complete
playable-game or browser acceptance claim.
