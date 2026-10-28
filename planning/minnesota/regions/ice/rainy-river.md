# Rainy River Icefields

This is one hand-authored32×24 winter region and a real5×4 warming hut west of
R20. The final whole and all12 player cameras were reviewed by sight. This is a
published683-map native review candidate; actual region gameplay and GPU
acceptance remain pending their separate controller/capture receipts.

The frozen river has a broad northern ice bend, a narrow stepped middle and a
long lower crossing. Snow banks stop sliding and provide an ordinary return.
Snow, icy encounter grass and ICE use separate native collision types. Snowy
pines fit the river banks; the hut uses the complete native forest facade.
This is separate from the Boundary Waters Safari and its open connected lakes.

## Frozen source and exact art

- `MnRainyRiverIcefields.layout.json`: SHA256
  `0e474ee47f0f3fdea0fcd3c32f0f5683d1f206222ce74457214f29f7017b1c78`.
- `MnRainyRiverIcefields.json`: SHA256
  `9f795c5da670c0db655906d7d046fd4c6862805b790209a40f3731e7d4f42019`.
- Compiled native blocks: SHA256
  `718fcae45a4ade9bef073c5577c6e40ba1aa28a4ba6e1441b3abac55fc1ec057`.
- Recipe `winter-art-v01.json`: SHA256
  `4efd8ef08fc06bfc870144014602aca5ca639b34015bade3dddb5ac769aedd74`.
  Use corrected `preview_winter_art_v02.py`: all224 original palette entries
  remain exact; new private glyphs occupy E0–F5. Geometry uses native blocks3c
  snow/FLOOR,3d icy/TALL_GRASS,3e sliding/ICE and3f snowy pines/WALL.

The final source-art epoch is
`renders/ice/region-v01/final-v02-native-slots/`:47 visual/anchor check groups,
29 physical groups and13 rejected physical/source counterexamples. All whole,
12 player-camera and hut images are byte-identical to the earlier visually
reviewed final-v01 despite the corrected private glyph allocation. Measured warm
rerender0.41seconds. All rejected earlier passes and originals remain retained.

`final-v03-native-music/` records the only subsequent source metadata correction:
`music` was `MUSIC_ROUTE_42`, which the actual pack does not contain; actual
native Route42 and R20 both use `MUSIC_LAKE_OF_RAGE`. Geometry, art, event text,
parties, rewards and all other source fields remain exact. The strict publisher
requires both that actual native map value and its installed audio manifest.

`rainy-river-native-preimage.contract.json` records the original681-map3d1c
baseline and exact R20 module preimage. The publisher receives the already
art-added pack and separately verifies that actual chosen parent's file SHA.
It does not mislabel an art-added pack as the earlier3d1c file.

## Concrete courses and services

`rainy-river-controller-plan.json` lists every adjacent ordinary input and each
complete ICE traversal, with actor bodies and the two optional trainers' sight
squares excluded from ordinary geometry courses.

- R20 core `(0,44)` and `(0,45)` join the new east edge `(63,44/45)`, reciprocal
  offset0. R20's blocks, actors, programs, flags and warps remain exact. Only its
  unused WEST connection and corresponding attribute mirror are added.
- Lower glide: snow `(54,36)` Left→snow `(37,36)`,17 actual walking tiles;
  reverse Right→`(54,36)`. Opposing inputs cannot redirect a live slide. The
  ordinary actor-present snow-bank return is139 steps.
- Short west bank to hut:36 ordinary steps with6 real icy encounter-grass
  steps. The long snow return provides a different encounter/travel choice.
- Upper cache: snow `(19,7)` Down→snow `(19,10)` across two ICE tiles,3 total
  steps. Reverse Up returns to the original bank. The cache island has no
  ordinary dry approach. `NEVERMELTICE`×1 is at `(21,11)`, approached `(20,11)`.
  Its fresh flag is `EVENT_MN_ICE_NEVERMELTICE`.
- Snow `(18,10)` Left slides6 steps to ICE `(12,10)`, stopped by actual forest
  `(11,10)`. Right returns6 steps to snow `(18,10)`. This gives a genuine
  stationary ICE stop for direction change and Save/Continue acceptance.
- Hut active door1 `(9,27)`→room passive2 `(5,6)`. Room active carpet1 `(5,7)`
  →outside passive2 `(9,28)`. Both passive arrivals are ordinary unoccupied FLOOR.
  The first Down from `(5,6)` lands on the directional carpet `(5,7)`;
  another Down triggers the native exit. The initial controller run reached
  that carpet correctly but its fixture expected the exit one input early.
  The keeper at `(5,5)` runs actual native HealParty at command index4. This
  restores HP/status/PP without granting a border quest or changing recovery.
- Optional SKIER Elsa: PILOSWINE48/JYNX49; BOARDER Nils: SNEASEL49. Both use
  installed trainer classes, native learned moves and fresh defeat flags.
  New native class-local IDs/event constants are allocated without overwrites.
- Native grass species only: SWINUB/SNEASEL/NOCTOWL/PILOSWINE/JYNX/DELIBIRD/
  URSARING. Lutebrisk's pure ICE habitat is explicitly reserved for later F038
  integration; it is absent from this parent's251-species native catalogue.

International Falls remains an unimplemented town; no native connection or
border service is invented. R17's existing geography is77 metatiles away and is
not joined by an artificial filler strip. The region has no new Water table or
Surf shortcut through the frozen channel.

## Build and acceptance

Root owns Cargo and all publication. The focused API is
`build_winter_region_modpack(base, layout_json, dossier_json)`. Its new-only
lowering uses group84 after checking it is free, exact existing templates and
R20 art/program preimages, real native trainer/item/service tables, complete
uncached production assembly and full residual preservation. All681 original
modules remain exact except the R20 connection; original audio/art/programs,
HOME/recovery, encounters and story catalogue remain exact. New maps reuse the
existing R20 seven-bit landmark.

The CLI requires a fresh output and verifies the chosen input hash and complete
written readback. CLI argument shape used by the successful music-corrected publication (Root owns
the built executable; the final existing output must not be overwritten):

```sh
build_winter_region \
  content-packs/minnesota-winter-art-v01.crystalpack \
  planning/minnesota/regions/ice/MnRainyRiverIcefields.layout.json \
  planning/minnesota/regions/ice/MnRainyRiverIcefields.json \
  87660aad1dea2a811dbb070c1a592e3ab46246edd05852aa4033d4ec44c6c536 \
  content-packs/minnesota-rainy-river-ice-v01.crystalpack
```

Five ignored asset cases cover native structure, stale source/art/namespace
preimages, meaningful unsafe geometry, unrelated-data corruption and complete
uncached/written roundtrip. Root must run them with
`CRYSTAL_WINTER_REGION_BASE` set to the exact art-added parent.

Three ignored production cases are in
`crates/crystal-bevy/src/bevy_shell/tests/winter_region.rs`, child of
`minneapolis_falls.rs`, filter `winter_region_controller_`. They read
`CRYSTAL_WINTER_REGION_CONTROLLER_PACK` once per process and clone immutable
verified runtime data for fresh sessions. Explicit initial fixtures are legal
Mew80/SURF/Fog, local Floor position, native replay RNG and2 Max Repels; the
healing case also starts with HP−30/PARALYSIS/PP1. These are local acceptance
fixtures, not earned regional progress. All later movement, item consumption,
healing, battle, rewards and saving use production controller inputs. Successful
item proof retains the actual Found payload and fully prints both PocketText
pages; it does not pretend the ephemeral automatic Found page was observed
between turn-level press returns. Root runs actual PP/money/once-only battles,
both-lane seams, slides/stops/returns, shelter/ICE-stop Continue and native GPU
views before the new region can be called playable.

Optional actual reached-state GPU handoff: set `CRYSTAL_WINTER_REGION_SAVE_DIR`
when running the controller cases. Unique production Saves retain the lower
seventeen-step stop, healed hut, collected cache, stationary ICE wall-stop and
both trainer victories. Coordinate-only native views and purposes are under
`native-review/`; source/reference comparison is explicitly separate from GPU
execution.

## Actual written native region

Root's music-corrected publication succeeded in
`.local/winter-region-publication-v02.log`:
`content-packs/minnesota-rainy-river-ice-v01.crystalpack`,683maps,
SHA256 `0d712d2311355534dc37c1d8ff83053217af14f0cac213cc971635a5745a5bea`,
identity `d59e55e3414a452d0a0566909c5f271da437a88471a1cc467f081a47b8762956`.
Independent written proof is retained at
`renders/ice/region-native/v01-preservation/`:5,962checks/23corruptions PASS,
all680 unrelated old modules exact, R20 only declared connection, both complete
new modules independently source-constructed, all99exteriors connected without
overlap, exact old embedded audio/programs/art/HOME/recovery. Written source
render comparison has14images/1,083,392pixels exact; this is not GPU proof.
Production controller and headless GPU results remain separate pending receipts.
The frozen source JSON's historical detached status is intentionally unmodified.
