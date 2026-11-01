# Ampharos and Granbull head-volume revision

Ampharos's short round head has been replaced by a longer hanging muzzle with
smaller lateral eyes, a higher crown and more horizontal yellow ears with black bands.
Granbull now has a broad beveled jaw, fuller cheek points, a compact nose and
small upper fangs. Its mouth is a real recess rather than a black patch over
an uncut skull. Beveled fists and broad flat paws replace the round pads.

This follows the [leg/posture revision](upright-trio-fidelity.md). The stored
leg/haunch geometry is exactly unchanged in both models, and authoring heights
remain 1.35 and 1.06. Ampharos's crown retains the previous ear-tip datum so
reorienting the ears does not rescale the body or supports. Granbull's paws
retain the previous exact floor datum for the same reason.

The [Ampharos](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339492/)
and [Granbull](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339520/)
Battle Revolution volumes were inspected at all eight 45-degree angles, with
an additional eight-angle underside orbit for Granbull. Captured game idle
front/back frames and the actual local pack pictures from the earlier review
were also rechecked. Extracted neutral arm poses differ from the desired idle;
original relaxed flippers and the bent guard remain. Reference meshes and
textures stay external and are never inputs to authored geometry or exports.

`tools/ampharos_sculpt.py` lowers the front of the continuous head while
retaining the rear neck connection. Nose and smile layers follow the actual
stored triangles. The lower head extends forward and remains thick at the
facial landmarks. The neck bands follow interpolated sections of the neck
carrier before the muzzle hangs in front of it; guessed cylindrical bands
left uneven exposed edges. `tools/electric_sheep_glb.py` assigns the lowered
anterior tissue to the head without pulling the neck rings onto that carrier.

`tools/bulldog_sculpt.py` uses rounded rectangular sections for the cheeks,
jaw, fists and paws. The head's anterior surface is recessed behind the
mouth opening. Its dark lining is clipped to those real triangles and cannot
bridge across the chamber. A separate solid tongue overlaps the lower jaw
and shares its carrier with the body-colored cheek points. The jaw pivot
in `tools/bulldog_glb.py` follows the revised hinge position. Small upper
fangs remain seated in the upper muzzle through motion.

Both final neutral models were inspected through all eight angles. Additional
stored-skin views sample idle 0.25, attack 0.34 and hit 0.18 at quarter/profile
angles. Granbull's underside orbit checks the jaw, cuffs and complete soles.
The retained before assets are the immediately preceding leg-corrected GLBs,
and studio scale and lighting are shared with the revised assets.

| Model | Vertices | Triangles | Parts | Joints | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ampharos | 3,806 | 5,804 | 30 | 9 | 204,788 |
| Granbull | 10,841 | 14,536 | 42 | 13 | 487,372 |

Thirty-five focused Python checks pass against the final stored assets.
The new guards reject the prior Ampharos head, Granbull jaw/sole proportions
and filled-in mouth. Granbull's aperture probes are derived from the actual
posed upper lip, jaw and tongue in the head carrier's frame. They check empty
volume at three lateral positions and several depths through idle, attack and
recoil, while also requiring the lining behind the gap. A former fixed mouth
coordinate in empty air would not pass this check.

Independent geometry/storage validation, closed-volume/shading tests, occupied
roots, fitted layers, strain and self-intersection checks also pass. Existing
budgets remain unchanged, including Ampharos's 6,000-triangle ceiling. The
strict family exporters preserve Mareep, Flaaffy and Snubbull. The audit skill
now explicitly distinguishes an actual open chamber from dark paint over
solid geometry; that rule was applied and its skill validator passes.

Seven focused Rust checks pass: electric-sheep and bulldog GPU activation at
verified pack heights, their runtime anatomical owners, the complete species
schema, the three upright species' complete walking supports, and hip-centered
biped steps. The test executable was checked for the exact compressed current
Ampharos and Granbull payloads before running those checks.

The production examples were rebuilt with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Both resulting executables were independently checked for the exact compressed
GLBs, including decompression equality with the canonical assets. They consume
the completed ignored `content-packs/core-modular.browser.crystalpack` without
rebuilding it. Captures used `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, and `CRYSTAL_PREVIEW_MUTE=1`.
Ordinary `RALA` walking inputs admitted Route 29 battles; the move-menu reviewer
held the visible menu after production input. New Bark walking used the
production controller at `(7, 12)`, hour 12, with `RR` and the supported
`2 1 35 0` and `5 0 20 0` cameras. The battle-only hold flags were off for field
captures. Original physical sizes and scene-admission checks remained active.

All twelve PNGs were individually inspected. Six paired-state receipts preserve
checksum, LCD animation frame and virtual time across the corresponding views:

| Species / view | Frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Ampharos / battle | 866 | 654370726 | 1761 | 31.242500583 |
| Ampharos / wider walking | 585 | 1573653256 | 631 | 11.635124999 |
| Ampharos / closer walking | 657 | 69064354 | 671 | 11.962136958 |
| Granbull / battle | 811 | 714508003 | 1657 | 29.209539790 |
| Granbull / wider walking | 708 | 547397221 | 761 | 14.012327624 |
| Granbull / closer walking | 681 | 903971925 | 708 | 14.121005166 |

Battle receipts identify the requested modeled species and no fallback. Field
receipts identify the requested species as `PartyFollower`. The head silhouettes,
neck bands, eyes, jaws and flat paws remain readable at gameplay size. The
classic battle move menu covers the player's back picture, so that image proves
matching state rather than back-view likeness. The classic field omits the added
follower. A sign partly covers the near foot in the closer field camera; the
wider view and isolated orbit supply the support review. Camera-relative walking
can end at different positions between camera configurations; each pair preserves
its own state.

Only Ampharos and Granbull changed among the 276 canonical variants in this
revision; the other 274 GLBs are byte-identical. Nothing was staged or committed.
Reference downloads, draft exports and temporary verification files were removed
after the selected review images and receipts were retained in ignored output.

The retained review is
`target/pokemon-fidelity-review-20261020-head/index.html`.
Coverage is limited to these revised assets, the inspected neutral orbit and
sampled poses. It is not blanket acceptance of the 276-variant roster.
