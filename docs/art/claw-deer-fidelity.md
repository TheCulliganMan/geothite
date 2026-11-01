# Sneasel and Stantler volume revision

Sneasel's previous hocks splayed outward and sloped forward. Its plume and
three tail feathers were built in the wrong plane: wide from the front and
almost invisible in profile. Stantler had smooth oval antlers, an oversized
blue nose, stern decal eyes, rounded hoof buttons and missing flank ribs.

## Reference and pose decisions

The authentic Battle Revolution
[Sneasel](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339525/)
and [Stantler](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339543/)
models were inspected through eight actual rendered angles. Baseline, game
reference and revised models share a 1.6 orthographic camera scale, 8-degree
elevation and labeled 4 × 2 contact sheets. Studio height normalization is for
comparison; production physical sizes are unchanged. Ordinary Johto male
Sneasel supplies the long plume reference; female and Hisuian forms are outside
this review.

All five Sneasel front frames, all four Stantler front frames and both back
pictures were inspected from the supplied pack. Four frames each from the
separate Showdown front/back animations provide supplemental volume and pose
evidence ([Sneasel front](https://play.pokemonshowdown.com/sprites/ani/sneasel.gif),
[Sneasel back](https://play.pokemonshowdown.com/sprites/ani-back/sneasel.gif),
[Stantler front](https://play.pokemonshowdown.com/sprites/ani/stantler.gif),
[Stantler back](https://play.pokemonshowdown.com/sprites/ani-back/stantler.gif)).
Their generation is not established here.

Crystal's complete Sneasel front animation confirms its raised left guard.
The extracted A-pose supplies volume, rather than neutral arm carriage. Its
feet are now under the hips, with nearly vertical shafts and separate forward
projecting toes. The skull is rounder, hips fuller, plain ear shorter and plume
root supported by its own auricle. Sagittal folded plume and tail blades are
broad from either profile and narrow from front/rear. A fitted narrow closed
smirk replaces the previous painted open mouth and tongue.

Stantler keeps a standing quadrupedal neutral pose from the complete 3D
reference; Crystal's front includes a charging pose and is not treated as an
identical idle animation. Its angular branching antlers retain open loops and
lower violet orbs. A tan cheek volume supports a smaller dark nose, with a
softer fitted gaze, drooping ears and a longer cream bib. Three fitted cream
ribs wrap each flank. Slim front shafts stay upright, rear hocks have a smaller
bend, and flat cloven soles lead into sloping pasterns.

Full-color game references govern pigments. Crystal's four-color palettes
reuse shadow indices across different surfaces; they do not establish blue
nose pigment or pure white antlers. No downloaded game mesh, texture or rig
is used as an authoring/export input or shipped asset. Both models remain
original recipes in this repository.

## Stored anatomy and motion checks

| Species | Vertices / triangles | Logical / draw parts | Joints | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Sneasel | 8,992 / 8,736 | 38 / 38 | 13 | 188 | 395,344 | 0.889 m |
| Stantler | 13,113 / 12,428 | 56 / 51 | 12 | 223 | 545,588 | 1.397 m |

The verified Rust compiled-pack reader reports height digits 211 and 407,
respectively. Authoring heights remain 1.11 and 1.28. No loader, scene admission,
physical scale or numerical tolerance was relaxed. Six Stantler rib layers
share one draw primitive with lossless logical ranges; runtime geometry pins
use the packed buffers rather than the logical authoring order.

All 38 Python checks pass: stored closed solids, actual normals, full source
parity, dimensions and budgets, binds, owners, planted soles, clip endpoints,
posed physical joins, fitted features, positive winding and strict edge strain.
Ten independent fidelity checks inspect actual decoded geometry without using
an author digest. All ten reject the old models. They measure shin axes,
feather plane/thickness/sweep, skull and hip depth, full buried feather root
caps through motion, muzzle/nose separation, angular antlers, long bib, six
wrapping ribs, sloping cloven hooves and actual posed surface clearance.

A collinear feather-cap corner exposed a quantized sliver; the revised contour
has real turns and derives face normals from final stored f32 positions.
Sneasel's secondary arm deformation also needed a broader blend and smaller
wrist turn to preserve the existing 0.80–1.20 strain limits. Its main raised
palm and both talons retain a forward attack excursion above 0.03 authoring
units. Stantler's shin endpoints now bury into the actual sloping hoof volume.

The production gait regression exposed a separate classification defect:
`Talon / hooked ivory hand claw` had been interpreted as a foot and rebound to
walking IK. Explicit hand talons now classify as arms, preserving their wrist
skin; ordinary foot talons retain their leg role. The new two-/four-support
regression checks the actual complete walking rig, upper bindings, ground
contact and bounds across 64 gait phases.

The background Blender builder and ordinary Python exporters produce identical
GLBs. Girafarig's shared authoring path reproduces its unchanged canonical
bytes. Only these two of 276 canonical battle assets changed in this pass;
the other 274 retain their pre-review bytes.

Eighteen selected Rust checks pass, covering the revised rigs and pack-size
GPU path, the unchanged Girafarig path, complete two-/four-support gait,
canonical schema/binds, finite clips, dense culling bounds, malformed container
rejection, actual battle cue timing and roster-wide world rig coverage.
At the time of this review, one wider near-loop check fails on unchanged
Charizard: a sail vertex near
`(-0.7589795, 1.2250961, -0.1665185)` moves beyond the existing 0.00001
near-loop tolerance. Its pre-review/current GLB SHA-256 is unchanged
(`69b393e4069bba185f292d757cf427ce76d6d9ce31423d83808307e9b9b44d73`).
The test and tolerance remain intact; this is an open broader-roster issue.

The later [Charizard/Dragonite revision](dragon-pair-fidelity.md) supersedes
that Charizard result and asset identity. The failure above remains the
historical result of this Sneasel/Stantler pass.

## Production review

The rebuilt test executable and both native preview executables contain the
exact current compressed GLBs. Decompressing the embedded payloads reproduces
the canonical bytes; executable hashes stay fixed across the captures.

Eight field pairs were inspected at noon in Ecruteak City: front quarter,
profile, rear/rear quarter and ordinary walking for each species. Sneasel uses
camera zoom 3, Stantler zoom 2. The ordinary `RR` / `RRRR` routes finish with
both player and follower facing east, verified from the actual receipt rather
than inferred from camera direction. Stantler's rear camera changed from orbit
4 to 5 because the player obscured its head; the final rear quarter exposes
the antlers and neck. Both retain their actual pack physical heights.

Two admitted Route 29 walking-encounter battles were inspected at the fixture's
16:00 time: Sneasel versus Pidgey and Stantler versus Sentret. Each pair has
`state_preserved=true`, the intended modeled identities, `source=[]`,
`fallback=None`, and a final `accepted=true reason=None` terrain trace. The
recorded uniform model scale reproduces 0.889 m / 1.397 m respectively.
Sneasel's upright shafts, separated toes, guard, side-facing feathers and round
skull remain readable. Stantler's padded muzzle, short dark nose, side ribs,
cloven wedges and attached tail remain readable under the native lighting.

The classic field omits the added follower, so its pair establishes matching
state and composition rather than sprite likeness. The battle move menu hides
the player's back picture; the actual pack backs and complete neutral studio
orbits supply that evidence. Profile battle cameras show Stantler's antlers
nearly edge-on, while the studio front/rear and native front/rear quarters
establish their full branching silhouettes.

Final local review:
`target/pokemon-fidelity-review-20261021-claw-deer/index.html`, with 32 PNGs,
ten paired-state receipts, exact asset/executable proofs and test logs.
Every final neutral angle and all eighteen decoded idle/attack/hit views were
visually inspected. The ten native/classic pairs were inspected after checking
state receipts. These sampled native views do not establish exact sprite-frame
animation, female forms, shiny palettes, browser visuals or full-roster visual
acceptance. The broader roster audit remains open.

## Executed commands

```sh
python3 -m unittest discover -s tools -p 'test_sneasel*.py'
python3 -m unittest discover -s tools -p 'test_stantler_glb.py'
python3 -m unittest discover -s tools -p 'test_hoofed_sculpt.py'
python3 -m unittest discover -s tools -p 'test_claw_deer_fidelity.py'

/Applications/Blender.app/Contents/MacOS/Blender --background \
  --factory-startup --threads 2 --python-exit-code 1 \
  --python tools/build-battle-species.py -- \
  target/pokemon-native-review/claw-deer-fidelity-20261021/blender-source \
  --only sneasel,stantler,girafarig

CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run --message-format=json
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1 --message-format=json
```

The Python suites were executed together against the exact candidates before
publication; canonical copies are byte-identical. The listed test build is
followed by nineteen individually selected test-executable invocations, with
eighteen passes and the retained Charizard failure above. Native captures use
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_PREVIEW_MUTE=1` and normal party fixtures, with movement driven by the
production controller. `CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1` is scoped to the
battle fixture and `0` for field movement.

Reference-only downloads, draft exports, raw orbit frames and temporary
reviewer helpers are removed after publishing the selected ignored gallery.
The completed shared packs and task Cargo cache are preserved. No staging or
commit is part of this work.
