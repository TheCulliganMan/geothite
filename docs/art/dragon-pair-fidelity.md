# Charizard and Dragonite volume revision

Charizard's previous wings had wide rectangular outer corners and teal backs.
Its neck was upright, the cream bib reached too high, the tail was short and
curled sharply upward, and its arms hung down. Dragonite had a shallow narrow
pear, round shallow skull, large frontal bat wings, oversized feelers, hanging
arms, forward-sloping ankles and a short curled tail.

## Reference and pose decisions

Authentic Battle Revolution
[Charizard](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339346/)
and [Dragonite](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339463/)
were inspected through eight actual rendered angles. Game reference, immutable
baseline and revision share 2.4 orthographic scale, 8-degree elevation and
labeled 4 × 2 sheets. Studio normalization is for comparison; pack dimensions
and authoring heights remain unchanged.

All four Charizard front frames, all six Dragonite front frames and both back
pictures were inspected from the supplied pack. Four frames each from separate
Showdown front/back animations provide supplemental posture evidence
([Charizard front](https://play.pokemonshowdown.com/sprites/ani/charizard.gif),
[Charizard back](https://play.pokemonshowdown.com/sprites/ani-back/charizard.gif),
[Dragonite front](https://play.pokemonshowdown.com/sprites/ani/dragonite.gif),
[Dragonite back](https://play.pokemonshowdown.com/sprites/ani-back/dragonite.gif)).
Their generation is not established here. Every sampled frame fits its panel.

Dragonite's extracted model has horizontal arms; Crystal's complete front
animation supplies the bent forearm guard. Grounded neutral mesh supports
remain distinct from its existing Fly world travel. Charizard's raised hands
and forward neck follow Crystal and the game model. Its extracted orbit has
no visible tail flame; the separate animations supply that posture evidence.
Dark triangular sail shading does not establish holes in the membrane.

Both retain original procedural recipes. No downloaded mesh, texture, rig or
animation is an authoring/export input or shipped asset. Full-color game
references guide the orange/yellow, cream and teal pigments; low-index Crystal
shadows alone do not identify those colors.

## Revised anatomy

Charizard has a fuller barrel, forward neck, shorter projecting muzzle,
back-swept horns, raised complete three-digit palms and explicit short upright
ankles above planted pads. The cream field ends below the neck. Its long low
tail carries an actual supported underside and a narrower, deeper original
five-layer flame. Peaked scalloped wings have orange closed backs, fitted teal
fronts and complete spars/ribs on coherent wing controls.

A quarter view exposed a far-wing triangle below the jaw. Fixed-camera part
isolation identified the wing; additional back sweep clears the silhouette.
The shoulder web remains orange. This is separate from the mouth correction:
a closed recessed dark throat and floor fitted to the actual moving jaw now
back the empty gape. The throat overlaps both roof and jaw through sampled
motion, and floor clearance is measured in the jaw's moving frame.

Dragonite has a deeper pear, taller deeper skull, larger muzzle and haunches,
compact rear-swept wings, moderately shorter feelers, bent forward forearms
and a long low tapered tail with its continuous ventral ribbon. Actual ankle
shafts remain nearly upright, separately from their projecting toes. The arm
centerline bends gradually to avoid crossed interior faces. Wrist weights
follow its actual arc distance across a broader transition; this fixes a
20.37% stretched edge while retaining the existing 20% limit and attack pose.

| Species | Vertices / triangles | Logical / draw parts | Joints | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Charizard | 17,482 / 26,850 | 72 / 45 | 15 | 234 | 752,436 | 1.7018 m |
| Dragonite | 18,499 / 28,652 | 60 / 34 | 15 | 184 | 772,872 | 2.2098 m |

The verified Rust compiled-pack reader reports height digits 507 and 703.
Both authoring heights remain 1.40. Logical ranges preserve complete source
anatomy while the runtime reads packed buffers. All existing geometry, part,
accessor, byte, floor, strain and numerical limits remain intact.

Current canonical GLB SHA-256 identities are
`15e66eb92d1f8dc8f4cc298ef06863ba96d2e95f5c5931a94a27b73e771d672d`
for Charizard and
`544055acf9a24df8a4e875a8b653f7c1932e942d474432500ecf7e5cee8243c1`
for Dragonite.

## Stored and source checks

Fifty-five unique Python checks pass: forty-one source/skin checks across both
complete families and fourteen independent decoded fidelity checks. They cover
closed winding, normals, source parity, budgets, hierarchies/binds, every
independent control, fixed soles, positive posed triangles, strict edge strain,
occupied joins/caps, supported eyes/bib/floor, real empty mouths, wing surfaces,
tail ribbons through motion, un-crossed limbs and mutation rejection. Both
mouth-root methods were repeated after adding the explicit throat/floor join
checks. All fourteen fidelity checks reject the immutable old pair.

Contour boundaries weld before extrusion where narrow stripes or eye sockets
would create quantized slivers. Fitted color remains on its actual parent;
concave wing scallops use perimeter offsets rather than radial shrinking.
The stored near-loop check samples the same absolute 10-microsecond offset
as Rust's production test. Charizard's idle stroke retains four beats and its
nominal amplitude with reduced velocity at each join. Clip durations and key
counts remain unchanged.

Background Blender and ordinary Python produce identical GLBs for all five
family members. Charmeleon, Dratini and Dragonair reproduce their unchanged
canonical bytes. Only Charizard and Dragonite changed among the 276 canonical
assets in this pass; the other 274 retain their pre-review bytes.

All eight final neutral views and nine decoded pose panels per species were
inspected: idle 25%, attack 34% and hit 46%, each at 45/90/180 degrees with
fixed neutral fit. These are sampled poses, not every animation or transition.

```sh
PYTHONPATH=tools python3 -m unittest -v \
  test_fire_dragon_sculpt test_fire_dragon_glb \
  test_dragon_family_sculpt test_dragon_family_glb test_dragon_fidelity
```

The actual source builder is `tools/build-battle-species.py --only
charmeleon,charizard,dratini,dragonair,dragonite` in background Blender, using
`--python-use-system-env` and `PYTHONPATH="$PWD/tools"`. Review output stays
ignored under `target/`; game content packs are not regenerated.

Twenty-one selected Rust checks pass against the revised canonical assets:
schema/physical size, binds and endpoints, finite clips, dense off-grid bounds,
malformed-container rejection, both complete family rigs, all five actual-height
GPU paths, world support/flight preservation, current battle cue timing,
anatomical ownership and full-roster travel coverage. The focused
`dragon_pair_stored_idle_closes_at_production_time_offsets` check uses the
production f32 sampler and the same helper/tolerance as the shared loop check;
both revised dragons pass.

The later [fossil-pair revision](fossil-pair-fidelity.md) resolves the Aerodactyl
failure recorded below; the broader loop check now reaches unchanged Pidgeotto.
The following diagnostic and 21/22 result record this dragon-pair checkpoint.

One wider check remained failing on unchanged Aerodactyl, identified by
species/time/vertex in its diagnostic: idle time 3.19999, vertex 11793, neutral
`(-0.7196787, 1.0942374, -0.1210889)` and sampled
`(-0.719685, 1.0942271, -0.12108911)`. Its Y difference exceeded the existing
0.00001 maximum-axis tolerance. Charizard's former failure was resolved; this
later roster failure was left open at that checkpoint, with Aerodactyl and
the limit unchanged. Those results were 21 passes and one failure.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib \
  dragon_pair_stored_idle_closes_at_production_time_offsets
```

## Native review

The rebuilt test executable and both native examples contain the exact revised
compressed GLBs. Each compressed payload decompresses to its canonical asset
and occurs inside the executable bytes. Both running preview identities stay
unchanged throughout capture and recapture.

Ten paired 1000 × 700 captures were inspected: quarter, profile, rear-quarter
and ordinary four-step travel for each species in Ecruteak City at noon, plus
two admitted walking encounters on Route 36. Every receipt preserves the
authoritative state. Travel receipts confirm east-facing followers after the
turn; their original Fly classification remains active.

Field cameras use zoom 2, quarter orbit 1/pitch 35, profile orbit 6/pitch 50,
rear orbit 5/pitch 35 and travel orbit 0/pitch 35. Dragonite's rear tail clipped
at zoom 2; recapturing the same placement at normal zoom 1 includes the full
tail. Body size and bounds remain unchanged. A faded building beside the
traveling player does not hide either follower.

Charizard's native views show the forward skull/neck, open backed mouth,
raised palms, short upright ankles, long burning tail and distinct orange/teal
wing faces. Dragonite's deeper skull/pear, friendly muzzle, bent forward arms,
compact wings, upright ankles and long ventral tail remain readable. Far
limbs and wings can overlap in profiles; the separate orbit supplies those
views. Small claws, belly folds and wing ribs remain distance-dependent.

Both battles use complete modeled bodies with no source fallback and zero
support offset. Admission traces retain uniform scale and the actual pack
heights: Charizard 1.7018 m against Bellsprout, Dragonite 2.2098 m against
Vulpix. These ordinary witnessed encounters give clear low side views with
the full tails in frame. No physical size, support, geometry-omission, bounds,
camera-admission or authentic-ground threshold was relaxed.

The classic field omits the added follower, so those pairs prove matching
state and context. Battle move menus cover the player's back picture; the
separate supplied back sprites and complete studio orbits provide that
likeness evidence. Normal native samples do not establish shiny, browser,
every-frame animation or full-roster acceptance.

The native examples were built with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Captures set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1`,
`CRYSTAL_RENDER_TEST_PARTY_SHINY=0`,
`CRYSTAL_RENDER_TEST_SECOND_VIEW=toggle`,
`CRYSTAL_RENDER_TEST_CAPTURE_SOURCE_FALLBACK=0` and synthetic ordinary input.
Party species selects CHARIZARD or DRAGONITE. `render_at_location` uses the
completed `content-packs/core-modular.browser.crystalpack`, Ecruteak City
position (18,23), or (20,27) for rear views, `--view both --hour 12`; travel adds
`--walk RRRR`. Only battles set `CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`,
`CRYSTAL_RENDER_TEST_WALKING_MAP=Route36` and `CRYSTAL_ENCOUNTER_TRACE=1`.
Their `immersive_battle_3d` command uses the same completed pack,
`--walking-encounter --walk` followed by `RALA` repeated 40 times,
`--size 1000x700 --screenshot` and `--second` output paths.

The ignored gallery is
`target/pokemon-fidelity-review-20261021-dragon-pair/index.html`, with 31
selected PNGs, ten final paired-state receipts, source/executable identities,
pack-height/admission excerpts and executed verification logs. Reference
archives, draft exports and temporary captures are removed after saving it.
The reusable `game-model-fidelity-audit` skill now includes component isolation,
moving mouth floors, absolute-time loop sampling and precise roster failure
attribution. Nothing was staged or committed; broader roster work remains open.
