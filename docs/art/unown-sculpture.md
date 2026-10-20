# Unown A–Z forms and standard glyph skins

Unown needs 26 distinct DV-selected bodies. The production battle frame now
publishes a one-based `unown_form` from the same visible DVs used for its source
picture. `VisualBattleBattler::model_id()` selects `UNOWN_A` through `UNOWN_Z`.
The renderer caches bounds, geometry, rigs and actor identity by this key, while
species labels, party slots and pack dimensions remain attached to `UNOWN`.
Missing or invalid forms resolve no mesh and retain the source picture.
Transform retains the original identity until the source picture-change
boundary, then publishes the transformed target's letter and shiny appearance.

`tools/unown_sculpt.py` authors the 26 letter silhouettes as smooth volumetric
strokes surrounding a continuous dark eye socket. Each has one fitted white
front lens and pupil, with a solid dark reverse surface matching the backpics.
B/G's secondary loops and O's outer ring were rounded after comparison with
the source sprites.
Closed loops use continuous tube topology without overlapping end caps; F/Y
feet use separate buried branches rather than a reversing tube.

The supplied pack establishes height 108: 0.5080 m (1 foot 8 inches). Normal
RGB555 `(7,7,8)` supplies dark RGB8 `(57,57,66)`; eye and pupil use source white
and black. The higher gray palette anchor `(15,15,16)` remains available to
normal/shiny remapping. All forms use an authored full-silhouette height of 1.0.
Each canonical GLB has three joints: identity root, coherent eye and glyph.
The eye and reverse surface share one owner; strokes blend from that owner
into delayed glyph rotation. Idle lasts 3.6 seconds with 85 keys; attack/hit
last one second with 65 keys. Clips are rotation-only and return to neutral.
Their timing consumes the existing source battle cue clock.

The final family totals 4,819,624 bytes. Each file has 5–8 named parts,
3,970–4,610 vertices, 7,088–8,168 triangles and 32–45 accessors. Existing reader
limits remain unchanged. All 26 CLI exports reproduce canonical bytes.

The battle registry publishes one Pokémon species; its generic `icon_unown`
world representation shares the I skin, preserving that icon's existing glyph.
The canonical model family lives at `models/battle_species/unown_a.glb` through
`unown_z.glb`. The old world JSON is removed. Both builder branches publish
all 26 forms; Python catalogue routing rejects incomplete families and counts
Unown once. No separate shiny geometry is authored.

Production extraction tests pass for all 26 letters and both Transform sides.
The API key check rejects unresolved forms without changing species identity.
Eight final stored-model checks pass across all 26 forms, including positive
closed topology and genus-one secondary loops, normals, fitted pupil clearance
against stored parent triangles, exact pack colors, owners, floor clearance,
strain and closed clip endpoints. Three canonical-routing checks pass.

The first full rig suite found excessive camera padding for the thin glyphs.
The existing 90 Hz envelope sampling fixes that without changing authored clips,
physical scale or tolerance. The full suite then passed 42 active checks, with
one existing ignored check. After the final O correction, four focused
registry/GPU-form-replacement/size/envelope checks pass, as does the complete
schema check. Builder branches are syntax checked; full Blender batches were
not executed.

The first native captures were nearly edge-on, making A resemble I. A new
letter-plane hint opens only Unown toward the establishing camera, choosing
the nearest front/back side while keeping the opponent as its primary facing.
The pose stays fixed through the shot, and the full animated bounds are refitted.
Other participants retain their poses; scale, support and hit registration stay
coherent. The dedicated layout regression passes for both sides and four
corridor directions. Readability was inspected again in a repeated A pair.
The final native examples rebuilt successfully. All 22 layout checks and
40 GPU-skinning checks pass after the presentation correction; five actor-prop
checks pass. A repeated native 3D/2D pair was captured and inspected for every
A–Z form, plus shiny I and V. All 28 receipts preserve state. The letters and
negative spaces are readable, in frame and separate from the HUD at the pack's
0.5080 m size. The classic backpics naturally show the dark reverse side;
the oblique native camera can see the white front lens. Shiny I/V match the
source blue palette using the same canonical geometry and skin.

Representative final receipts: A checksum 4060871309, O checksum 99271016,
shiny I 2943203603 and shiny V 1045820805. Each pair has its own held LCD frame
and virtual time; temporary receipts and review images remain ignored.
This verifies the held native views and form selection, not full source-frame
motion or browser fidelity. Those captures predate the subsequent full catalog;
current coverage is recorded in the [species guide](battle-species.md). Packs
and audio are unchanged.

## Y foot attachment

A later three-angle roster review found that Y's lower stem stopped above its
separate horizontal foot. The supplied pack's front animation frames and
back picture show a continuous connection. The stem now ends at the foot's
center instead of 0.07 authoring units above it. Only that stroke's positions
and position-dependent weights change. The foot, other strokes, eye, normals,
materials, complete bounds, hierarchy, inverse binds and decoded animation keys
remain identical; the other 25 forms are preserved.

The stored-triangle regression checks occupied stem/foot overlap at nine times
in each of the three clips. A deliberately detached copy fails, and the
previous Y GLB also fails its neutral contact check. Overall normalized height
remains 1.0, so the pack's 0.5080 m presentation size is unchanged.
All nine stored-model checks pass across the 26 candidate forms, and Y's
installed bytes reproduce on Python 3.9.6 and 3.14.6. The corrected front,
side and rear windowless GPU views were inspected again.


```sh
python3 tools/unown_glb.py --form all
PYTHONPATH=tools python3 -m unittest test_unown_glb -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  unown --lib -j2 \
  --config profile.test.opt-level=0 \
  --config profile.test.package.offset-allocator.opt-level=2
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_HEADLESS=1 \
  CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  ./target/pokemon-models/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --wild-species UNOWN \
  --unown-form A --size 1000x700 \
  --screenshot target/pokemon-native-review/unown-a-3d.png \
  --second target/pokemon-native-review/unown-a-2d.png
```

`--unown-form` belongs only to the disposable location-tester preview. It
creates valid form-selected DVs through the normal Pokémon constructor.
The `--shiny` preview supports Crystal's I/V DV forms. The production renderer
uses the existing shiny boolean and pack palette remapping, with no alternate
shiny mesh files. The preview never opens or modifies a user save.
