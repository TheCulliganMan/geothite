# Warm North Shore production controller handoff

The focused unregistered child leaf is
`crates/crystal-bevy/src/bevy_shell/tests/north_shore_warm.rs`, intended under
`minneapolis_falls::north_shore_warm`. Root owns registration and Cargo. The
actual native source baseline is the published warm pack
`content-packs/minnesota-superior-warm-coast-v01.crystalpack`, file SHA256
`ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8`,
identity `5009f2898563ecf699d7036b19d988ef5f4d43a8b39cbd48f8616cc9c45eca32`.
The tests load the caller's explicit verified pack once without mutating
process-wide environment variables, and permit a later composed identity.
They require all eight R11 chunks to use warm art with no active ICE. They do
not require the old Cedar/Grand Marais connection or a particular global map
count, so the independent new coast geography can compose afterward.

Five ignored cases exercise real `VisibleShellController::press` input:

1. Three legal Floor fixtures walk15,31 and23 ordinary steps respectively,
   entering all52 formerly sliding quadrants from the frozen828-cell contract.
   After every adjacent move,96 released frames must leave the player exactly
   on that tile in Normal mode. All52 former ICE quadrants are covered; a slide
   to the old end cannot satisfy this test. Each course saves and continues.
2. The old native grass03 is approached from warm Floor44,30, reached at46,29,
   continued while standing in grass, crossed through47,29 and returned to
   Floor44,30. The real native Max Repel is used through Pack, and its remaining
   steps are consumed naturally. Party, bag and flags stay unchanged afterward.
3. All three native cave-stair lanes are walked both ways with their real
   blocked sides. Palisade's actual HOP_DOWN lip is entered, its landing reached,
   the reverse blocked, and the preserved29-step ordinary return is walked
   through the ridge/stair lanes. No defeated trainer flags are required.
4. Separate initial qualification fixtures deny the actual Baptism waterfall
   for missing Waterfall or missing Rising Badge; the real refusal notice owns
   movement. A qualified fixture mounts, climbs, continues at the native upper
   WATER tile, descends through the real downward current and dismounts on the
   original bank. These fixtures do not earn campaign badges or HM access.
5. Southern Superior starts on legal Floor49,188, mounts north at49,187, Surfs
   east to58,187 and south to58,189, then attempts Down into visible BUOY58,190.
   Surf remains blocked, Continue retains that state, and real inputs return
   to the same dry bank. This uses the current expanded56×96 coast; the older
   three-chunk test's launch49,36 is now Floor and is deliberately not reused.

All source-planned Foot and water courses avoid actual actor bodies and every
conservative standing trainer sight tile. No position, flag, RNG, inventory or
party is rewritten after the initial parent fixture. Native Max Repel prevents
incidental wild encounters. The tests do not grant trainer defeat flags.

After registration, the publication owner can run this focused group using
its serialized Cargo target:

```sh
CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK="$PWD/content-packs/minnesota-superior-warm-coast-v01.crystalpack" \
CRYSTAL_NORTH_SHORE_WARM_SAVE_DIR="$PWD/planning/minnesota/renders/north-shore-ely-geography-v01/r11-warm/NATIVE_EPOCH/saves" \
cargo test --locked -p crystal-bevy north_shore_warm_controller -- --ignored --test-threads=1
```

This command is handed off, not reported as executed by this author. Unique
pid/nanosecond save names retain every run. Continue compares map, tile, facing,
movement mode, party, bag and flags. The saved game's actual native identity
is retained rather than rewritten to fit an image or scenario.

## Native GPU positions

The static checker inventories nine unoccupied legal initial Floor cameras,
including their actual chunk/local coordinates. These are safe declared local
fixtures for an independent warm-art capture; none proves reached gameplay:

| View | Global core | Native map | Local core |
|---|---|---|---|
| Northern warm turn | 40,8 | MnR11C00R00 | 40,8 |
| Cabin and Baptism falls | 23,22 | MnR11C00R00 | 23,22 |
| Ordinary upper bank | 36,24 | MnR11C00R00 | 36,24 |
| Superior cove | 48,33 | MnR11C00R00 | 48,33 |
| Native rocky terrace | 25,64 | MnR11C00R01 | 25,16 |
| Second stream | 33,78 | MnR11C00R01 | 33,30 |
| Cave stair | 37,91 | MnR11C00R01 | 37,43 |
| Stoney falls | 25,152 | MnR11C00R03 | 25,8 |
| Duluth approach | 47,189 | MnR11C00R03 | 47,45 |

Prefer the actual retained reached Saves when reviewing input results. Useful
labels and their reached positions are:

| Save label | Global core | Mode |
|---|---|---|
| warm-northern-lane | 41,8 | Normal |
| warm-upper-bank | 37,24 | Normal |
| warm-shore-cove | 44,34 | Normal |
| warm-restored-green-grass | 46,29 | Normal, grass |
| warm-palisade-stair | 23,49 | Normal |
| warm-birch-stair | 37,92 | Normal |
| warm-lester-stair | 37,170 | Normal |
| warm-palisade-ledge-landing | 18,54 | Normal |
| warm-palisade-return-upper | 18,51 | Normal, grass |
| warm-falls-move-denial / badge-denial | 18,26 | Surf |
| warm-baptism-falls-summit | 18,19 | Surf |
| warm-baptism-falls-bank-return | 22,27 | Normal |
| warm-superior-southern-visible-buoy | 58,189 | Surf |
| warm-superior-ordinary-bank-return | 49,188 | Normal |

Grass and water Save positions above must not be presented as new initial
Floor fixtures. Render the original Save with its own map, position, mode,
facing and clock; no post-load position/hour/flag rewriting. Native static and
reached GPU captures must remain headless and retain original images plus
exact source comparisons. Source images and collision checks are separate
from production GPU acceptance.

## Read-only static course proof

```sh
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/revisions/north-shore-ely-geography-v01/r11-warm/controller-v01/check_static.py \
  --pack content-packs/minnesota-superior-warm-coast-v01.crystalpack \
  --expected-sha256 ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8 \
  --output planning/minnesota/renders/north-shore-ely-geography-v01/r11-warm/NEW_STATIC_EPOCH
```

The checker writes only a fresh ignored report/snapshot folder. It checks all
52 former ICE quadrants, source-authored Foot/grass/stair/return courses,
standing actor/sight avoidance, waterfall and southern buoy tokens, all eight
warm aliases and the nine Floor cameras. It does not execute inputs, create
a pack, certify gates or replace native visual review.
