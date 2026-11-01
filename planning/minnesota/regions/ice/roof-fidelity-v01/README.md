# Two cold roof canopies

The actual waterfall GPU frame revealed a bright green canopy strip embedded
above the brown house roof. It comes from native North Shore blocks2c/2d at
MnR11C00R00 metatiles(11,9)/(12,9). Their top two8px glyph rows use native
canopy1e/1f/3e/3f; the lower two rows use brown timber glyphs. Whole-map source
agreement had preserved this visible defect, so this separate append corrects
its appearance without changing the accepted geography or gameplay.

`roof-fidelity.contract.json` is frozen SHA256
`db9a077f82b7a307c5df41a4c2751bf64d5f88064d405ee40dd7306d214d96eb`.
Its exact parent is `content-packs/minnesota-northern-winter-v01.crystalpack`,
file SHA256 `aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a`,
content identity `ab1c60993ffc1bb0d4c3ef4c02f2455208cadc939faee3ffa5ce310ebbb61805`.
The proposed fifth stage follows the four already published stages documented
in `../INTEGRATION.md`: winter art → Rainy River region → mixed winter edges →
substantial existing-route winter sections → this roof fidelity append.

The append adds only mn_winter_north_shore blocks89/8a and changes only those
two cold map cells. New glyphsFA..FD keep these precise native canopy shapes,
using the already reviewed winter-canopy index mapping and foreground colors.
Existing snow glyphs were checked and are not exact shape matches. FE/FF remain
free. Every old137-block recipe and every old used glyph/palette remains exact.
All four collision quadrants stay `[HEADBUTT_TREE, HEADBUTT_TREE, WALL, WALL]`.
Brown roof pixels, facade walls and the door2e remain untouched. No Cut,
Headbutt availability, encounters, actors, events, programs, room, port, audio
or report metadata is changed.

The final source preview is
`renders/ice/roof-fidelity-v01/source-review-v02/roof-canopy-before-candidate.png`
relative to the Minnesota planning directory. It was inspected by sight. Its
source camera at core(23,22) is ordinary unoccupied Floor and includes the
complete canopy in a native160×144 view, enlarged4×. The first preview at
(23,24) cropped the canopy out of the smaller source LCD window; that superseded
pass remains retained. Both epochs keep complete before/candidate maps. Exactly
1,024 terrain pixels change within source rectangle[352,288,416,304); every
changed pixel lies in the canopy half of the two explicit cells. Original
recipes and both roof-timber halves match in all four time banks. This is a
source preview, not an emitted pack or a GPU capture.

Reproduce the fast source preview in a fresh output directory:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/regions/ice/roof-fidelity-v01/preview_roof_fidelity.py --pack content-packs/minnesota-northern-winter-v01.crystalpack --contract planning/minnesota/regions/ice/roof-fidelity-v01/roof-fidelity.contract.json --output planning/minnesota/renders/ice/roof-fidelity-v01/NEW_SOURCE_REVIEW
```

The source-only Rust implementation is
`crates/crystal-assets/src/winter_roof_fidelity.rs`, with its focused validation
and test leaves. Public API:
`build_winter_roof_fidelity_pack(&CompiledGamePack, &str) -> Result<CompiledGamePack>`.
It validates the exact source preimages, free glyph slots, reused canopy
profile, inherited collisions and two map cells. Complete output residual
normalization restores only the individually validated owned differences before
requiring whole CompiledGamePack equality. Cached and uncached block geometry
and all unrelated native parser state are checked separately. No map attributes
or report/manifests change; identity is recomputed normally.

After registration/build by the publication owner, the fifth-stage CLI is:

```sh
build_winter_roof_fidelity content-packs/minnesota-northern-winter-v01.crystalpack planning/minnesota/regions/ice/roof-fidelity-v01/roof-fidelity.contract.json content-packs/minnesota-northern-winter-v02.crystalpack
```

The publication owner executed this fifth stage successfully; strict written
readback is recorded in `.local/winter-roof-publication-v01.log`. The actual v02
file SHA256 is `92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33`,
content identity `043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f`.
The CLI requires a fresh output and verifies exact written/read-back pack equality. Three external
asset tests use `CRYSTAL_WINTER_ROOF_TEST_PACK` set to the exact v01 parent. They
cover expected native art hashes, cached/uncached residual preservation,
idempotency/stale source/occupied glyph rejection, and unrelated art, geometry,
program, palette and report mutations. All three passed in the publication
owner's `.local/winter-roof-tests-v01.log`. This author ran no Cargo or native
publication. The accepted native v01 packs, source contracts, renders and failed
review epochs remain immutable.

Independent actual v02 preservation and four negative checks pass under
`renders/ice/roof-fidelity-v01/written-native-v02/proof.json`. Both actual headless
GPU views also pass with zero full-frame mismatches using the retained production
binary51d7855c: core(23,22) shows the full canopy context, and core(23,24) retains
the original waterfall view. Every original was personally inspected; the green
strip is gone and timber/door pixels remain native. See
`renders/ice/roof-fidelity-v01/native-gpu-v02/capture-progress.json` and
`visual-review-final.json`. Actual before/after images were captured at different
Water phases(0 and1); their derived comparison sheet explicitly labels this,
and neither original was phase-normalized. Each complete capture independently
matches a coherent native reference. These explicit local fixtures do not prove
earned campaign access.
