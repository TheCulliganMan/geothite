# Pidgey family fidelity revision

Further user feedback reopened Pidgey's face after this family pass. The
[facial follow-up](pidgey-face-fidelity.md) records its current geometry,
newly captured comparisons and verification. The earlier native/family results
below describe that earlier pass; its original gallery is no longer present
locally. Pidgeotto and Pidgeot are unchanged in the facial follow-up.

This revision replaces the shallow raised-wing family silhouettes with deeper
individual bird bodies, grounded folded resting wings and upright short shanks.
Pidgey's face also changes from a round brown forehead, far-lateral eyes and
tiny bill to a tapered skull, diagonal cream brows framing a brown forehead,
narrow eyes near a broad short pink bill, a continuous cream throat and a real
backed mouth opening. The overall roster improvement goal remains active.

## Reference and pose

The complete eight-angle Battle Revolution model orbits were inspected for
[Pidgey](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339351/),
[Pidgeotto](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339352/)
and [Pidgeot](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/303320/).
These supply volumes, profiles, rear surfaces and pigments. Some extracted
materials contain black/alpha artifacts; those texture artifacts do not become
anatomy. Foreign geometry is reference-only and never feeds the authoring
recipes or shipped assets.

All supplied Crystal front frames were inspected: four Pidgey, five Pidgeotto
and four Pidgeot, plus each back picture. Pidgey has raised and folded wing
frames; its folded frame supplies the current rest. Pidgeotto and Pidgeot
retain grounded folded resting wings throughout their front animations.
The semi-open extracted game meshes do not override that source posture.
The actual [Home Pidgey render](https://img.pokemondb.net/sprites/home/normal/pidgey.png)
from the [sprite gallery](https://pokemondb.net/sprites/pidgey) additionally
resolves the cream brow, eye spacing, broad bill and throat. The gallery's
X/Y and Let's Go thumbnails are pixel art, not captured 3D references.

The retained review contains labeled 4-by-2 full turntables of the reference,
initial body and revised body. Baseline and revision share camera elevation,
lighting and orthographic scale; the smaller folded-wing silhouette is not
independently enlarged in that comparison. Pidgeotto/Pidgeot additionally have
matching closer reference/revision sheets for the head, crest and folded wing.
Pidgey's rejected draft and corrected face have their own matching comparison.

## Original geometry and skin

Pidgey's brown folded wing cases cover actual cream sagittal vanes, rather than
an XY fan that becomes a line from the side. Three short swept brown crest
feathers replace the upright ear-like points. Brow quills sweep backward;
the brown forehead remains visible between the diagonal cream brows. Its
broader shorter bill opens in front of recessed occupied lining. The head,
eyes, brows and lining share a coherent head carrier; the lower jaw articulates
independently. Pads, toes and talons remain fixed through the battle clips.

Pidgeotto now has an authoritative pure recipe in `tools/pidgeotto_sculpt.py`.
It no longer reads its own canonical GLB as an authoring input. A deep stocky
breast supports a shorter forward head, five swept red crest blades, brown
folded cases, broad cream profile vanes and a raised red/cream tail. The existing
34 public anatomical names and 14 controls remain. Shoulder ownership, head
mask, wrist field and all three tail-layer pivots follow the revised anatomy.

Pidgeot has a deeper breast and shorter neck, fitted lateral face, brown upper
and cream lower folded wings, three broad gold and two red swept crest feathers,
short upright shanks and a raised red tail. A rigid wing frame aligns the old diagonal fan axis before folding the actual
feather plane down beside the body. Closed brown cases cover the feather roots; inverse-frame wrist weights preserve that anatomy during motion.
Crest blades have width in front as well as profiles, rather than parallel
ribbons that become needles. It retains 105 source parts, 25 packed parts and
20 controls. Fearow is unchanged byte for byte.

| Model | Stored vertices / triangles | Bytes | Authoring height | Verified pack height |
| --- | --- | --- | --- | --- |
| Pidgey (current face) | 8,757 / 12,600 | 440,812 | 0.82 | 0.3048 m |
| Pidgeotto | 9,353 / 14,116 | 414,368 | 1.0 | 1.0922 m |
| Pidgeot | 18,266 / 24,648 | 834,244 | 1.35 | 1.4986 m |

All retain floor zero and the existing loader budgets. Shading is recomputed
from final stored f32 triangles. The background builder reproduces the exact
canonical assets. Individual recipe/skin contracts are in the
[Pidgey](pidgey-sculpture.md), [Pidgeotto](pidgeotto-skin.md) and
[Pidgeot/Fearow](raptor-bird-skins.md) guides.

## Verification

Seventy-six Python checks pass for the current assets. They cover source and
normal-builder parity, strict malformed readers, topology and shading, actual
closed components, occupied roots and fitted layers, head coherence, fixed
supports, floor clearance, positive winding, original strain bounds, used
controls and cue timing. Family regressions independently measure depth,
shank direction, feather plane, crest volume and near-loop continuity. Four
facial checks reject the actual user-rejected Pidgey draft. The old raised-wing
shape assertions were replaced by evidence from the folded Crystal frame.

```sh
PYTHONPATH=tools python3 -m unittest test_pidgey_glb test_pidgeotto_glb \
  test_pidgeotto_skin_glb test_raptor_bird_sculpt test_raptor_bird_glb \
  test_pidgey_family_fidelity -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run
```

The production f32 sampler passes the focused family and complete-roster
near-loop checks with the unchanged absolute 10-microsecond offset and
0.00001 positional tolerance. This resolves the Pidgeotto seam recorded during
the earlier fossil-pair review. Folded rests use small settling motion with a
flat-ended wave; separate world travel still completes upstroke and downstroke.

A world-motion check found proximity welding was blending independently owned
Pidgeot crest roots. World setup now repairs split weights only after travel
actually changes bindings, preserving the authored controls for untouched skins.
This keeps overlapping buried feather roots independent without weakening the
existing exact-binding test. Pidgeotto's travel test now requires a folded rest
and the real levelled wing stroke instead of the obsolete airborne idle beat.

Ninety-three final Rust checks pass, including all 73 shared travel checks,
family rig/support and GPU checks, malformed loaders, exact neutral/schema
checks, dense off-grid bounds and the focused and complete-roster idle loop.
Exact compressed current GLB payloads are found in the tested executable; its
hash is unchanged across the run. The normal background builder reproduces
Pidgey, Pidgeotto, Pidgeot and unchanged Fearow byte for byte. Across the 276
canonical variants, only these three birds changed during this batch.

An unrelated supplied-cuff review helper needed matching `people-review`
feature guards so the normal test build could compile. Its review-only entry
and supplied-rig check now share the existing accessor's feature/platform
condition; the synthetic cuff test remains available to normal unit tests.

## Native production review

Fifteen headless native/classic pairs were captured and inspected: quarter,
profile, rear and ordinary walking in Ecruteak City, plus a witnessed Route 36
battle for each bird. Every paired receipt preserves authoritative state.
Each final battle scene reports `accepted=true`, `reason=None`, modeled
identities, an empty source fallback list and `fallback=None`. First-stage
body data independently recovers the original 0.3048, 1.0922 and 1.4986-metre
pack heights from uniform scale and reference height; support height is zero.
No physical-size, camera admission, numerical or geometry limit was relaxed.

Exact current compressed GLBs were found in both capture executables and the
tested executable. Their hashes stayed fixed across captures/tests. The retained
[comparison gallery](../../target/pokemon-fidelity-review-20261021-pidgey-family/index.html)
contains 30 final native/classic PNGs, 15 paired-state receipts, compact battle
admission excerpts, executable/asset identity and state/physical-size evidence.
Draft movement captures, downloaded models and temporary reviewer copies were
removed. The gallery and build caches remain ignored; nothing was staged or
committed.

Pidgey's corrected cream brow, closer narrow eyes, broad bill and continuous
throat are readable in the admitted battle view. Its field body is deliberately
small at 0.3048 metres; those wide views establish placement and silhouette,
not fine eye detail. Pidgeotto retains its red swept crest, deep breast and
folded cream feather layers. Pidgeot's brown shoulder cases cover the former
ribbed outline; its long gold/red crest, lowered folded wings and raised red
tail remain legible from both sides and rear quarters. Daylight separates the
brown/cream fields, with darker backs still readable. No reviewed bird is
clipped by the frame.

The classic field omits the added follower, so those pairs prove state and
world context rather than follower likeness. The battle move menu covers the
player's classic back picture; inspected pack backs and studio rear views
supply that likeness comparison. Flower heads partly overlap Pidgeot's talons
in its native battle; clear studio profiles and support checks cover the full
feet. Existing building fading appears in walking captures while the birds
remain visible. World travel intentionally uses flight clearance, separately
from the grounded battle rest.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SPECIES=PIDGEY \
CRYSTAL_RENDER_TEST_CAMERA='2 1 35 0' \
  target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map EcruteakCity --x 18 --y 23 --view both --hour 12 \
  --screenshot target/pidgey-quarter.png
```

The other field views use the same normal camera/input path: profile at camera
`2 6 50 0`, rear at `(20,27)` and camera `2 5 35 0`, and walking from `(18,23)`
with camera `2 0 35 0` and `--walk RRRR`. Receipts confirm each walking follower
faces `[1,0]`. Battle captures use `immersive_battle_3d` with
`--walking-encounter --walk` and forty repetitions of `RALA`, the Route 36
review map, synthetic production input, held move menu and an explicit second
classic screenshot. All captures are muted and use the owned image target.

These checks cover the inspected neutral orbits, sampled stored clips and
native views. They do not establish every source animation, shiny form,
browser rendering or full-roster visual acceptance. The broader model audit
remains active.
