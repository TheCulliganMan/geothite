# Aerodactyl and Kabutops volume revision

The subsequent [Pidgey family review](pidgey-family-fidelity.md) resolves the
Pidgeotto loop failure recorded below. Its focused family and complete-roster
checks pass at the original tolerance; the failure here remains a record of
what this earlier batch actually observed.

The October 21 revision replaces Aerodactyl's frontal wing fans, oversized
gape, forward-sloping ankles and raised tail with broad profile wings, a
moderate open jaw, upright ankles and a low centered tail. Kabutops replaces
separate crown horns, striped ivory ribs, three central back knobs, thick ivory
scythes and diagonal shins with a continuous helmet, gray chest armor, five
paired lateral ridges, flat gray blades and backward-bending hocks.

Reference-only [Aerodactyl](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339457/)
and [Kabutops](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339456/)
Battle Revolution volumes were inspected at all eight 45-degree angles.
The actual local pack's four front frames and back picture were also inspected
for each species. Four sampled frames from each separate Showdown
[front](https://play.pokemonshowdown.com/sprites/ani/) and
[back](https://play.pokemonshowdown.com/sprites/ani-back/) animation provide
additional pose evidence; their exact originating game was not established.
Aerodactyl's extracted raised-wing pose and animated wing beats differ from
the Crystal front pose. The orbit supplies volume and full-color pigments,
while the sprite frames supply separate posture checks.

All runtime meshes remain original procedural work. Downloaded OBJ files,
materials and textures stay in an external disposable reference directory and
are never inputs to the authored geometry or canonical GLBs.

## Geometry and articulation

Aerodactyl's gray wing cases and fitted purple undersides are closed volumes
whose broad planes face the profiles. The shoulder and leading arm are rooted
in the hunched torso; fingers, wrist claws and both surfaces share their wing
hinges. The final warped contours are triangulated in their actual planes.
Warping the old long cap triangles first had folded some faces even though
their overall solids remained closed.

The hooked skull has shorter swept horns, smaller eyes fitted onto its sides,
red-violet irises and gray orbital ridges. A supported marking alone had not
prevented the old eyes from stretching down the snout. The jaw retains real
empty volume, recessed backing joined to both jaws, and upper/lower lining
fitted to the actual moving surfaces. The low tail ends in a thin profile
spade. Gray toes project forward from upright ankles. Idle retains four wing
beats and its nominal amplitude, with flat join velocity at the loop boundary.

The first native battle exposed an additional posture error: the world follower
flew correctly, but the battle path planted the same model on the ground.
Aerodactyl now uses the existing battle hover clearance, preserving the authored
rotations and source cue timing. Camera fitting and GPU culling include the
actual hover offset as well as the initial pose; neutral dimensions and physical
scale remain unchanged. This also corrects the bounds for existing hovering
battlers without changing their geometry or hover motion.

Kabutops has one broad angular helmet with a low central keel. Continuous gray
breast armor supports three fitted lobes. Ten closed lateral back ridges join
the torso. Long curved scythes have broad front surfaces and real thin side
walls; their original shoulder, elbow and wrist controls follow the new arms.
Outward thigh mass sits over hocks that bend backward above the forward toes.
The short tail points downward and retains weighted base/tip controls.

| Model | Vertices | Triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Aerodactyl | 16,822 | 25,888 | 63 / 32 | 9 | 173 | 712,220 |
| Kabutops | 8,050 | 12,326 | 46 / 27 | 13 | 146 | 357,264 |

Authoring heights remain 1.30 and 1.20, with the raised Aerodactyl wings included
in its full-height bound. A fresh Rust read of the completed browser pack gives
height digits 511 / 403, or 1.8034 / 1.2954 m. Production continues to use those
pack dimensions and uniform scale. Kabuto's existing 0.5080 m asset is exactly
unchanged. No pack rebuild or audio changes are needed.

The normal Blender builder and direct Python exporters produce identical
canonical bytes for all three fossils, including unchanged Kabuto. All 274
other canonical variants remain byte-identical to the start of this revision.

## Verification

All 53 focused Python checks pass against the published canonical GLBs. They
cover source parity, closed outward anatomy, actual corner normals, original
loader budgets, fixed supports, occupied roots, fitted parent surfaces,
empty jaw volume, moving carriers, winding, strain and source/byte mutation
rejection. Sixteen independent reference-shape and stored-motion checks all
reject the old models without relying on the author digest pins.

The former minimum jaw-gap assertion required an oversized gape, and the former
front-area assertion required the wrong arrowhead plane. Those anatomy-specific
assumptions were replaced using the inspected references. Numerical tolerances,
strain, occupied-support checks and loader limits remain unchanged.

The focused Rust loop check uses the production f32 sampler and the existing
absolute 10-microsecond offsets and 0.00001 positional tolerance. Aerodactyl's
former seam failure and Kabutops both pass. Rig ownership, fixed contacts,
independent controls, original-size GPU activation and world flight/walking
checks also pass. The test executable was independently verified to contain
the exact current compressed GLBs.

Twenty-eight selected Rust checks pass, including the complete schema, binds,
finite clips, dense off-grid/blended bounds, malformed input rejection,
source-cue timing, anatomical roles and roster travel coverage. One broader
loop check now fails on unchanged Pidgeotto: idle time 0.79999, vertex 2841,
neutral `(-0.552248, 0.721109, 0.143599)` and sampled
`(-0.55224514, 0.7211221, 0.143599)`. Its Y difference exceeds the existing
0.00001 maximum-axis tolerance. Aerodactyl's earlier failure is resolved;
this later failure remains open. The result is 28 passes and one failure,
rather than full-suite acceptance.

The hover regression samples 33 positions in each stored clip for every
hovering species. Actual skinned vertices remain inside the battle bounds;
Aerodactyl retains its air gap throughout. Mew's layout envelope and Mew/Porygon
GPU checks pass, while seated Lapras and grounded tentacle tips retain their
floor contacts. The ordinary current-visibility regression also passes.

The source checks were executed with:

```sh
PYTHONPATH=tools python3 -m unittest -v \
  test_aerodactyl_sculpt test_aerodactyl_glb \
  test_fossil_scythe_sculpt test_fossil_scythe_glb test_fossil_fidelity
```

## Native review

Both production examples were rebuilt after the hover correction:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The test executable and both examples were independently checked for the exact
current compressed Aerodactyl, Kabutops, Kabuto and Mew GLBs, including
decompression equality. Their executable hashes remained unchanged throughout
the final captures. The completed ignored browser pack supplied the actual
dimensions; it was not regenerated.

Ten final native/classic pairs cover the two revised fossils. Each has an
Ecruteak quarter, profile and rear quarter, ordinary `RRRR` travel, and a Route
36 walking encounter held at the real move menu. A further Mew battle pair
checks the shared hover-envelope change with its unchanged 0.4064 m geometry.
All eleven receipts preserve position, facing, checksum, animation frame and
virtual time within their corresponding pair. Every field receipt identifies
the intended follower; all three battles identify the requested models and
report no source fallback. Original full-envelope and scenery admission gates
remain active, with accepted terrain and no rejection reason.

All 22 final native/classic PNGs were inspected. Aerodactyl retains a visible
air gap during battle, coherent gray wing backs in the field, purple
undersides, an open jaw and a low tail. Kabutops retains its broad helmet,
continuous chest, flat blades, paired ridges and bent hocks. Its final battle
shows the feet without the flower overlap present in an earlier discarded
capture. Mew's complete body and tail remain in view above its shadow.

Field captures start at Ecruteak `(18,23)`, hour 12, except the rear quarter at
`(20,27)`. Cameras are `2 1 35 0`, `2 6 50 0`, `2 5 35 0` and `2 0 35 0` for
quarter, profile, rear quarter and walking respectively. Battles use
`--walking-encounter --walk` with `RALA` repeated forty times and
`CRYSTAL_RENDER_TEST_WALKING_MAP=Route36`. The move-menu hold is enabled only
for battles. Captures use headless image targets, a 1000×700 viewport and muted
preview audio.

The classic field omits the added follower, and the move menu hides the
player's back picture. These pairs prove state correspondence; separately
inspected sprites and isolated orbits provide the back-view likeness review.
High cameras naturally hide some underside tissue, and a side battle angle
partly overlaps Aerodactyl's eye with its wing arm. The isolated head and
complete orbit supply the unobstructed face review.

The retained review is
`target/pokemon-fidelity-review-20261021-fossil-pair/index.html`.
It includes before/reference/revised full orbits, sampled clips, head close-ups,
the eleven native/classic pairs and the exact asset/state/test evidence.
The reusable fidelity-audit skill records the demonstrated triangulation,
surface-fit, anatomy-guard and battle-hover lessons and passes its validator.

Coverage remains scoped to the inspected views, sampled motion and native
paths. Every animation, shiny appearance, browser view and the remaining roster
are not established by this batch. Pidgeotto's recorded loop issue remains open.
Nothing was staged or committed. Task-owned downloads, draft exports, temporary
helpers and raw capture output were removed after retaining the review; shared
packs and Cargo caches remain.
