# Scyther and Snorlax volume revision

Scyther's old scythes spread laterally and flattened in profile. Its wings
had the same plane error, the central crown was long and dark, the greaves
leaned forward, and the abdominal seams stopped at the front. Snorlax's head
was too small above a shallow, boxy seated body, with narrow feet and arms
extended down and outward.

## Reference and pose decisions

The authentic Battle Revolution
[male Scyther](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339440/)
and [Snorlax](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339458/)
assets were inspected through eight rendered angles. Labeled 4 × 2 sheets
share cameras across baseline, game reference and revision: orthographic
scale 1.8 for Scyther and 2.8 for Snorlax, both at 8-degree elevation. Snorlax's
larger camera fits the complete supine reference without clipping. Studio
height normalization does not change production physical size.

The supplied pack contributes all seven Scyther front frames, all six Snorlax
front frames and both back pictures. Four sampled frames each from the
separate Showdown front/back animations provide supplemental evidence
([Scyther front](https://play.pokemonshowdown.com/sprites/ani/scyther.gif),
[Scyther back](https://play.pokemonshowdown.com/sprites/ani-back/scyther.gif),
[Snorlax front](https://play.pokemonshowdown.com/sprites/ani/snorlax.gif),
[Snorlax back](https://play.pokemonshowdown.com/sprites/ani-back/snorlax.gif)).
Their generation is not established here.

Snorlax's extracted geometry is supine. Crystal's complete front animation
governs the seated posture; the extracted model supplies volume and pigment
evidence. Those poses are not presented as matching idle frames. Its authored
body now has a deeper rounded barrel, broader hood, larger soles and brown
pads, bent resting arms and a quiet fitted sleeping face. Full ear and arm
root caps stay buried in actual occupied head/body tissue through sampled
motion, rather than merely touching at their centers.

Scyther keeps a grounded mantis guard with forward-swept scythes, broad
sagittal wing surfaces, three short green cranial points, an angular cheek
and muzzle, a fuller abdomen and complete rear-wrapping shell seams. Lower
greaves are nearly upright; forward toes remain separate from shaft angle.
Its closed mouth seam does not claim an open jaw chamber. The blade hinge
now pitches the actual blade plane during its authored attack cue.

Full-color reference pigments supply green/cream and blue/cream/brown fields.
The small Crystal palettes alone do not establish those pigments. Foreign
meshes, textures and rigs are reference-only; neither original procedural
recipe consumes downloaded geometry or texture data.

## Stored anatomy and motion

| Species | Vertices / triangles | Logical / draw parts | Joints | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Scyther | 13,780 / 21,460 | 57 / 39 | 13 | 191 | 556,716 | 1.4986 m |
| Snorlax | 9,905 / 17,584 | 31 / 31 | 9 | 183 | 490,944 | 2.1082 m |

The verified Rust compiled-pack reader reports height digits 411 and 611.
Authoring heights remain 1.18 and 1.40. No loader, geometry budget, physical
scale, admission rule or numerical tolerance was relaxed. Scyther's packed
geometry pin differs from logical authoring order; lossless logical ranges
retain each source feature.

All 31 Python checks pass, covering stored normals, closed winding, source
parity, skin ownership, dimensions, joints/binds, clip endpoints, floor
clearance, positive posed triangles, strain, physical joins, blade cutouts,
both trim faces and fitted wing/face/body surfaces. Twelve independent
fidelity checks inspect decoded geometry without author pins. Ten fail on
the immutable old meshes; the old fitted-surface and arm-self-intersection
checks already passed and remain passing.

Thin-feature checks measure projected triangle area as well as bounds and
cross-section thickness. A diagonal edge-on wing can have a large bounds box
without a broad profile. Rotated trim probes run in the actual blade/wing
frame, selecting cap faces by normals rather than microscopic projected
side-wall residues. Fitted contour boundaries weld before extrusion to avoid
collapsed edge triangles. Adaptive blade trim preserves strict fit and budgets.

All eight final neutral views and nine decoded pose panels per species
(idle 25%, attack 18%, hit 31%, each at 45/90/180 degrees) were inspected.
These are sampled poses, not every possible animation phase or transition.
Background Blender Python and ordinary Python produce identical GLBs.
Scyther's rounded seam normals canonicalize signed zero for exact parity.
Only these two of 276 canonical assets changed in this pass; the other 274
retain their pre-review bytes.

Reproduce source checks with:

```sh
python3 tools/snorlax_glb.py
python3 tools/scyther_glb.py
PYTHONPATH=tools python3 -m unittest -v \
  test_snorlax_glb test_scyther_glb test_mantis_sleeper_fidelity
```

For source export inside background Blender, use its
`--python-use-system-env` switch with `PYTHONPATH="$PWD/tools"`. Scyther routes
through `build-battle-species.py`; Snorlax uses `snorlax_glb.export_snorlax`
directly. The battle builder does not register Snorlax; requesting it in
`--only` does not establish that an asset was produced.

Eighteen selected Rust checks pass: both revised rigs and actual-height GPU
paths, complete two-support walking with retained upper bindings across 64
phases, roster schema/binds, finite clips, dense culling bounds, malformed
container rejection, current battle cue timing, anatomy roles, centered gait
supports and roster-wide world rig coverage. The prior Sneasel/Stantler gait
regression also remains passing. Scyther retains Walk travel and Snorlax Heavy;
neither becomes a quadruped.

The wider near-loop check still fails on an unchanged Charizard sail at the
existing 0.00001 tolerance. The recorded neutral point is approximately
`(-0.7589795, 1.2250961, -0.1665185)` and the near-loop point
`(-0.7589881, 1.2250849, -0.1665186)`. Charizard's SHA-256 remains
`69b393e4069bba185f292d757cf427ce76d6d9ce31423d83808307e9b9b44d73`.
This open broader-roster failure is retained, without a threshold change.

The subsequent [Charizard/Dragonite revision](dragon-pair-fidelity.md)
supersedes that Charizard result and asset identity. The paragraph above
records the result at the time of this Scyther/Snorlax review.

## Native review

The rebuilt test executable and both native examples contain the exact revised
compressed GLBs. Each compressed payload decompresses to its canonical asset
and occurs inside the actual executable bytes. Executable SHA-256 identities
remain unchanged through capture.

Eleven paired 1000 × 700 captures were inspected: quarter, profile,
rear-quarter and ordinary four-step walking views of each species in
Ecruteak City, two admitted Route 29 battles and one additional Route 36
Snorlax battle. All receipts preserve authoritative state. Walk receipts
confirm that each follower completes its turn and faces east. Field cameras
use zoom 2, quarter orbit 1/pitch 35, profile orbit 6/pitch 50, rear-quarter
orbit 5/pitch 35 and walking orbit 0/pitch 35. Field time is noon.

Scyther's native blades retain depth, the rear wings remain visible and its
upright greaves support the guard. Snorlax's field views show its larger hood,
rounded belly, bent forearms and substantial feet. Blue rear tissue remains
readable in outdoor lighting. The walking camera includes a faded building
beside the player; it does not hide either follower.

The first Snorlax Route 29 battle uses a steep overhead camera and gives weak
face/depth evidence. It is retained and labeled as such. A second ordinary
walking encounter on Route 36 admits a clear low view against Growlithe at
the same 2.1082 m height. No camera override, model resize or admission waiver
was introduced. Scyther's Route 29 battle against Pidgey is also admitted at
1.4986 m. Both scenes use modeled participants and no fallback; the extra
Route 29 Snorlax/Pidgey scene is admitted as well.

The classic field omits the presentation-only follower, so those pairs prove
matching state and context, not source-follower likeness. Battle move menus
cover the player's back picture; the separately inspected supplied back
sprites and complete studio orbits provide that evidence. These normal native
samples do not establish shiny, browser, every-frame animation or full-roster
acceptance.

The ignored review gallery is
`target/pokemon-fidelity-review-20261021-mantis-sleeper/index.html`, containing
33 selected PNGs, eleven paired-state receipts, source/executable identities,
physical-size/admission excerpts and executed check logs. Reference archives,
draft exports and temporary capture logs are removed after the selected review
is saved. The reusable `game-model-fidelity-audit` skill includes the new
profile-area, full root-cap, local cap-normal and supine-reference lessons.
Nothing was staged or committed.

The executed native build was:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Captures use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1`,
`CRYSTAL_RENDER_TEST_SECOND_VIEW=toggle`, synthetic ordinary input and the
completed `content-packs/core-modular.browser.crystalpack`. Party species
selects SCYTHER or SNORLAX, with shiny set to 0. The battle command is
`immersive_battle_3d content-packs/core-modular.browser.crystalpack
--walking-encounter --walk` followed by the ordinary `RALA` route repeated
40 or more times, `--size 1000x700 --screenshot` and `--second` output paths.
Only battle captures set `CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`; the additional
encounter sets `CRYSTAL_RENDER_TEST_WALKING_MAP=Route36`.
