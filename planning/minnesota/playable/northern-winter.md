# Northern winter native review pack

Five Rust publication stages completed strict written readback. The current
file is `content-packs/minnesota-northern-winter-v02.crystalpack`: **683 maps,
99 connected Minnesota exteriors**. All eight controller cases pass on final
v02: three region cases and five route-section cases. The17 initial asset cases
individually pass, as do three roof asset cases and four final renderer cases.
Native static review retains29 exact baseline GPU comparisons plus two final
roof comparisons; five final GPU frames load actual reached controller Saves. These are
explicit local acceptance fixtures, not earned campaign progression or a
completed interactive-client playthrough. The following v1 acceptance remains
frozen; v02 identity-specific controller/capture receipts are recorded below.

## Exact published chain

All six files below are local, external/ignored packs. Their file hashes,
stored identities and counts were reread for this runbook. Nothing here exports
another audio/source catalogue or changes the original3d1c parent.

| Stage / local `content-packs/` file | File SHA256 | Content identity | Maps |
|---|---|---|---|
| Parent `minnesota-cohesion-harbor-water-v01.crystalpack` | `3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df` | `174f1942c42bd590e292e5a95340ffb9d93758bf0ab1ec31d83dadfd9fc9c722` |681|
| Art `minnesota-winter-art-v01.crystalpack` | `87660aad1dea2a811dbb070c1a592e3ab46246edd05852aa4033d4ec44c6c536` | `f05f841d747753e4093560b1d8a14b77b9746a3ebe4a2dd589299d9270e22815` |681|
| Region `minnesota-rainy-river-ice-v01.crystalpack` | `0d712d2311355534dc37c1d8ff83053217af14f0cac213cc971635a5745a5bea` | `d59e55e3414a452d0a0566909c5f271da437a88471a1cc467f081a47b8762956` |683|
| Mixed art `minnesota-winter-mixed-art-v01.crystalpack` | `2b2887fd056d507d85ac5d2b8861e92559d8679cf11902f63c42f270d617842d` | `bed021120fb15c2e2106b1ec8b4f030af252bc0f405572c6e7cf83f42f0275e4` |683|
| Final v1 baseline `minnesota-northern-winter-v01.crystalpack` | `aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a` | `ab1c60993ffc1bb0d4c3ef4c02f2455208cadc939faee3ffa5ce310ebbb61805` |683|
| Final v2 cold roofs `minnesota-northern-winter-v02.crystalpack` | `92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33` | `043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f` |683|

Executed success logs, in order:
`.local/winter-art-publication-v02.log`,
`.local/winter-region-publication-v02.log`,
`.local/winter-mixed-art-publication-v01.log`,
`.local/winter-sections-publication-v02.log`,
`.local/winter-roof-publication-v01.log`.
Each CLI writes a fresh output, reads it through the native verified reader and
requires full equality with the validated in-memory result before success.
Earlier failures remain retained: native palette-map224 versus256 allocation,
the nonexistent MUSIC_ROUTE_42 identifier, and absent-versus-null source metadata
were corrected without relaxing preservation. Actual Route42/R20 music is
MUSIC_LAKE_OF_RAGE.

## Reproduce from the exact parent

The initial four publisher executables currently exist under
`target/runtime-fps/wasm-dev/`. Their source is under
`crates/crystal-assets/src/bin/`; Root's successful build logs include
`.local/winter-publishers-build-v04.log`. Use the pinned Rust1.99.0 toolchain.
The designated publisher owns Cargo/export work; do not run a competing build.
These are existing Rust publishers, not a Python pack writer.

Inputs under `planning/minnesota/regions/ice/` are frozen. The region dossier's
`DETACHED_SOURCE_DRAFT_NOT_NATIVE` status is historical authoring-freeze metadata;
its exact9f79 hash is preserved. Actual publication and acceptance receipts
determine the current status. Do not relabel or overwrite frozen inputs.

Frozen input hashes:

| Input | SHA256 |
|---|---|
| `winter-art-v01.json` | `4efd8ef08fc06bfc870144014602aca5ca639b34015bade3dddb5ac769aedd74` |
| `MnRainyRiverIcefields.layout.json` | `0e474ee47f0f3fdea0fcd3c32f0f5683d1f206222ce74457214f29f7017b1c78` |
| `MnRainyRiverIcefields.json` | `9f795c5da670c0db655906d7d046fd4c6862805b790209a40f3731e7d4f42019` |
| `winter-mixed-edges-v01.json` | `a6d8f46a813555c5daff1deffe0d2cb0682570f244971c28fc19f985593df40c` |
| `sections/source-v03/integration.contract.json` | `c901add54e8ec698b72f563bcd2fa93ef0383e95bd399db8f6bb70f048032735` |

Run from the repository root. The following reproduces the same four executed
CLI stages into a **new directory**, preserving all existing published files.
This fresh reproduction was not run again by the runbook author. `mkdir`
intentionally fails if its directory already exists; choose a fresh numbered
epoch for another reproduction. The expected output hashes are the chain above.

```sh
(
set -eu
python3 - <<'PY'
from pathlib import Path
from hashlib import sha256
p = Path('content-packs/minnesota-cohesion-harbor-water-v01.crystalpack')
assert sha256(p.read_bytes()).hexdigest() == \
    '3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df'
PY
mkdir .local/northern-winter-reproduction-v01

target/runtime-fps/wasm-dev/build_winter_outdoor_art \
  content-packs/minnesota-cohesion-harbor-water-v01.crystalpack \
  planning/minnesota/regions/ice/winter-art-v01.json \
  .local/northern-winter-reproduction-v01/art.crystalpack

target/runtime-fps/wasm-dev/build_winter_region \
  .local/northern-winter-reproduction-v01/art.crystalpack \
  planning/minnesota/regions/ice/MnRainyRiverIcefields.layout.json \
  planning/minnesota/regions/ice/MnRainyRiverIcefields.json \
  87660aad1dea2a811dbb070c1a592e3ab46246edd05852aa4033d4ec44c6c536 \
  .local/northern-winter-reproduction-v01/region.crystalpack

target/runtime-fps/wasm-dev/append_winter_mixed_edges_art \
  .local/northern-winter-reproduction-v01/region.crystalpack \
  planning/minnesota/regions/ice/winter-mixed-edges-v01.json \
  .local/northern-winter-reproduction-v01/mixed.crystalpack

target/runtime-fps/wasm-dev/build_winter_sections \
  .local/northern-winter-reproduction-v01/mixed.crystalpack \
  planning/minnesota/regions/ice/sections/source-v03/integration.contract.json \
  2b2887fd056d507d85ac5d2b8861e92559d8679cf11902f63c42f270d617842d \
  .local/northern-winter-reproduction-v01/northern-winter.crystalpack

python3 - <<'PY'
from pathlib import Path
from hashlib import sha256
expected = {
    'art': '87660aad1dea2a811dbb070c1a592e3ab46246edd05852aa4033d4ec44c6c536',
    'region': '0d712d2311355534dc37c1d8ff83053217af14f0cac213cc971635a5745a5bea',
    'mixed': '2b2887fd056d507d85ac5d2b8861e92559d8679cf11902f63c42f270d617842d',
    'northern-winter': 'aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a',
}
for name, wanted in expected.items():
    path = Path('.local/northern-winter-reproduction-v01') / (name + '.crystalpack')
    assert sha256(path.read_bytes()).hexdigest() == wanted, name
print('All four reproduced files match the recorded publication hashes.')
PY
)
```

The sections CLI reads every exact before/candidate source file named and hashed
by its contract. It does not reread mutable canonical route drafts. The main
region CLI pins its actual chosen art parent's file hash; it does not pretend
that the art-added parent is still the earlier3d1c file.

## Stage5 cold-roof fidelity — actual v02 publication

Root executed the existing Rust CLI below and verified strict written readback
in `.local/winter-roof-publication-v01.log`. Its source is
`crates/crystal-assets/src/bin/build_winter_roof_fidelity.rs`; the executed
binary SHA is `3b946cd0afabcb81c23c4ee4ee0a757ffc39d4bd5e5ebd1faad5f1a5f213c87f`.
The CLI requires the exact v1 parent file and a fresh output:

```sh
target/runtime-fps/wasm-dev/build_winter_roof_fidelity \
  content-packs/minnesota-northern-winter-v01.crystalpack \
  planning/minnesota/regions/ice/roof-fidelity-v01/roof-fidelity.contract.json \
  content-packs/minnesota-northern-winter-v02.crystalpack
```

That actual output already exists; do not rerun the command over it. For a fresh
reproduction after the four-stage block above, use its reproduced v1 file and
new output name instead:

```sh
target/runtime-fps/wasm-dev/build_winter_roof_fidelity \
  .local/northern-winter-reproduction-v01/northern-winter.crystalpack \
  planning/minnesota/regions/ice/roof-fidelity-v01/roof-fidelity.contract.json \
  .local/northern-winter-reproduction-v01/northern-winter-v02.crystalpack
```

The expected v02 file hash is `92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33`;
its stored identity is `043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f`.
This fresh reproduction has not been repeated by the runbook author.

The frozen roof contract SHA is
`db9a077f82b7a307c5df41a4c2751bf64d5f88064d405ee40dd7306d214d96eb`.
It appends bounded snowy native canopy recipes137/138 and four private glyphs,
then replaces only R11 metatiles `(11,9)`44→137 and `(12,9)`45→138. Original
collision arrays remain HEADBUTT_TREE/HEADBUTT_TREE/WALL/WALL. It neither changes
region geometry nor introduces another gameplay gate or activity.
`sections/source-v04/final-source.contract.json` records the matching final
authoring bundle; the original c901 source-v03 publisher input remains frozen.

Roof-fidelity asset regressions **3/3 PASS** in
`.local/winter-roof-tests-v01.log` (62.66s), including exact two-cell/art residual
preservation and stale-source/private-glyph/unrelated-data rejection.

Final v02 main-region controller **3/3 PASS** in
`.local/winter-region-controller-v03.log` (102.90s). The actual existing test
binary was `.local/winter-controller-tests-v02`, SHA
`dfa5cd0eda92cd804727f23ce04a1d187c19b64f35fbb3f2d3871fac60f90950`, invoked
with `winter_region_controller_ --ignored --test-threads=1`,
`RUST_MIN_STACK=33554432`, `CRYSTAL_WINTER_REGION_CONTROLLER_PACK` bound to the
absolute final v02 pack and `CRYSTAL_WINTER_REGION_SAVE_DIR` bound to fresh
`renders/ice/native-controller-v03/region-saves`. The adjacent content-addressed
controller receipt retains the exact command, environment and five Save hashes.
No fixture state writes occur after initial setup; actual production input
changes are required and asserted. This reruns all main-region traversal,
cache/healing/trainer/resource/Continue cases against the final identity.

Final v02 route-section controller **5/5 PASS** in
`.local/winter-sections-controller-v04.log` (518.25s). Its independent receipt
`.local/winter-sections-controller-v04-receipt.json` binds final file92bb6455…,
identity043b57…, exact unchanged test binary and mandatory32MiB thread stack.
All82 ice-entry/return courses,16 wall-stop turns, original Cut denial/opening/
reload, ULTRA BALL once/Continue, and qualified Waterfall/current/bridges pass.
The preceding v03 attempt aborted from a missing thread-stack environment
setting; it remains retained. This rerun changed no map or production behavior.

The two final roof GPU comparisons in
`renders/ice/roof-fidelity-v01/native-gpu-v02/capture-progress.json` each compare
all368640 pixels with zero mismatches. The whole-area board in
`renders/ice/whole-area-native-v02/` matches all three reviewed source scenes
and all three terrain layers exactly; it remains a static native-art composite.
The refreshed connected atlas is `renders/ice/native-geography-v03/atlas/`.
Its connection graph is evidence for placement, not visual acceptance of river
geography: city/river continuity has a dedicated correction task under
`connections/river-continuity-v01/`.

Five fresh v02 reached-Save GPU captures **5/5 PASS** under
`renders/ice/reached-gpu-v02/`, using renderer `51d7855c…` and no map/tile/pose/
hour/party overrides. The capture receipt is
`receipt-09504eeae3ee2f047a79b44e1c48ec091284c25b2cc01570f135097c38165e88.json`;
`personal-review-ff51e1431b8b64b5cc2b74a1c8ba51abbfcba88e65eb26bbc2fcb71a358d043d.json`
records inspection of every original and contact. All five raw GPU buffers,
original PNGs, logs, conversions, exact opaque Save copies and history remain.
Input hashes stayed unchanged and the unique private executable was removed.
All five final v02 images also match their corresponding actual v1 GPU frames
pixel-for-pixel. That is an actual-frame preservation check, not a source-pixel
claim. These five region scenes do not show the changed R11 roof cells; the
separately owned roof-camera review is additional evidence.

Final v02 renderer regressions **4/4 PASS** in
`.local/winter-renderer-v02.log` (15.93s), explicitly bound to final `92bb6455…`.
The fresh24 production loader RGBA rustle assets in
`renders/ice/native-renderer-v02/rustle-frames/` were independently checked and
personally viewed. Every18×9 RGBA frame and PNG matches preserved v1 exactly;
each retains76 opaque/86 transparent pixels and native OAM opacity. The two
animation phases remain distinct, both winter aliases match one another, and
native green/night/dark controls remain unchanged. Exact proof and visual limits
are in `native-renderer-v02/rustle-review-v01/proof-d436de99adc9726f1a5caf1172a87b409aa895241193adb9eff64bc09c01f26f.json`.
These are production loader assets, not another GPU framebuffer or live
movement-effect timing claim. No duplicate frame renders were written for this
review; existing fresh assets and an in-memory comparison view were used.

The finite hut/adjacent keeper/carpet, snow landing, ICE wall-stop, cleared cache
and final Boarder scene remain readable. The broad frozen reach is still
visually repetitive. Actual local Mew80/SURF/Fog/RNG/inventory fixtures are
explicit; none of these results claim earned campaign access or a completed
interactive-client playthrough. All v1 inputs, failed attempts, passing logs,
Saves and captures stay retained under their own epoch.

The concise current visual evidence index is
`regions/ice/native-review/current-evidence-v02.md`: existing whole winter board,
full new-region original and five actual reached-Save GPU views, with their
different evidence types labeled. It creates no new maps or duplicate renders.

## What is installed

Rainy River Icefields is one32×24 hand-authored outdoor map plus a5×4 warming
hut in new checked-free group84. Its only regional join is reciprocal offset0
between R20 core `(0,44/45)` and the region's east edge `(63,44/45)`. Original
R20 changes only its unused WEST connection and attribute mirror; its terrain,
actors, programs, warps and story conditions remain exact. The region is a leaf
branch at world origin `[-30,-433]`; it does not connect around an old checkpoint.
International Falls town and the separate77-metatile R17 gap remain explicitly
unimplemented;99 native connected exteriors do not mean every planned town is
implemented.

The lower frozen crossing slides17 walking tiles in either direction. Snow
banks stop the slide. A139-step ordinary snow route and a36-step hut-bank
route with6 icy encounter-grass steps provide different travel choices. The
upper cache requires a three-step ICE approach. A separate real forest wall
stops the player on ICE `(12,10)`, permitting an ordinary direction change.
There is no new Surf/Water shortcut through this frozen backwater.

The hut has real door1→interior passive2 `(5,6)`, carpet1→outdoor passive2
`(9,28)`, a reachable keeper and actual native HealParty at command index4.
One Down lands on carpet `(5,7)`; the next Down triggers its directional exit.
It restores HP/status/PP without adding blackout recovery or completing a border
report. The keeper's “No border report / is handled here” distinguishes this rest
service from the existing R20 SQ03 quest desk; it is not an unimplemented-service
placeholder. HOME/recovery remain original. NEVERMELTICE×1 uses a fresh one-time
item flag. Optional native SKIER Elsa has PILOSWINE48/JYNX49; BOARDER Nils has
SNEASEL49, with native learned moves, class-local IDs and defeat flags.
Encounters use only installed native species. The parent and final pack contain
**251 species**; Lutebrisk's requested pure ICE habitat is a labeled future F038
reservation, not an installed species or encounter claim.

R17 now has a continuous northern snowy cedar/marsh section and deliberate
southern snowline. Its original lower water, timber bridge, Cut gate, habitat
necks, actors, rewards, huts and story remain. R11 has a substantial upper
frosted gorge with icy bank shelves/grass; Lake Superior, tributaries, animated
Waterfalls, bridge rails, lighthouse and existing programs remain exact. Only
`MnR17C00R00`, `MnR17C00R01`, `MnR11C00R00` and `MnR11C00R01` receive the declared
2078 metatile changes and tileset/attribute/block mirrors. This seasonal pass is
not a whole-route geometry completion claim. The user-rejected tiny15-cell
proposal stays archived and is not installed.

## Retained v1 evidence and remaining acceptance

- Art publication's3 native asset checks pass in
  `.local/winter-art-assets-v02.log`, including original-prefix and icy-grass
  priority invariants. Corrected private glyph allocation preserves all224
  original palette entries and uses E0–F5 for new art. The mixed append preserves
  original art and adds fitting sign/Headbutt/snow compositions; it does not
  rewrite Cut mechanics. Snow FLOOR, icy TALL_GRASS and sliding ICE are distinct.
- Main region independent written audit:
  `renders/ice/region-native/v01-preservation/`,
  `proof-a57ab730be05d9283f30993e6b8621ad3d2b2fb6e332ab1d974a66d2a915703e.json`:
  **5962 checks/23 corruptions PASS**. All681 old modules retained,680 wholly
  exact, R20 only declared connection; both new complete modules independently
  constructed from source/templates including every raw/typed program index,
  19 new roots,6 new constants, legal native parties and exact encounters.
  Entire residual data/art/audio/HOME/recovery are exact; both raw CBOR
  identities independently rehash. All99 actual exteriors form one consistent
  reciprocal component without rectangle overlaps.
- Sections independent written audit:
  `renders/ice/sections-native/written-aa1aa251-v02/`,
  `proof-8fe3ec8efbac7c371c8f50607fcf103d31b99a79746e570f2e0a1cd93d46351d.json`:
  **821 checks/10 corruptions PASS**. All679 unrelated whole modules—including
  the new region/hut—remain exact. Original raw/typed gameplay, encounter tables,
  rooms, actors, recovery, art/runtime files and377 audio descriptors/programs
  remain unchanged. Inherited old uncached representations are compared
  before/after instead of silently repaired to match a cache.
- Main region written-source render comparison:
  `renders/ice/region-native/v01-written-source-renders/`:14 comparisons,
  1,083,392 pixels exact. Both full RGBA maps and12 RGB player views were retained
  and personally inspected. This is source-art agreement, **not GPU proof**.
  Authentic IcePath same-scale comparison and remaining plain-bank/pacing
  weaknesses are recorded under `regions/ice/native-review/`.

The17 Rust asset cases **individually PASS**. The original batch in
`.local/winter-all-assets-v02.log` has16 passes and one failed negative-fixture
construction: it incorrectly expected embedded compiled_audio in a modular
`midi_synth_v1` pack. The corrected fixture corrupts an actual audio descriptor;
its targeted rerun passes in `.local/winter-assets-negative-v03.log` (7.52s).
The failed batch remains retained; this is not represented as an unexecuted
clean17-case batch rerun.

Main region **3/3 PASS** in `.local/winter-region-controller-v02.log` (23.55s):
both R20 seam lanes/directions; long slides with opposing input/empty frames;
six-grass and ordinary-snow returns; actual cache/PocketText/once; real hut
pairing and HP/status/PP HealParty; both native trainer battles with PP, money
and defeat flags; Save/Continue inside the hut and at a stationary ICE stop.
The first failed run is retained: its fixture expected the directional carpet
warp on the landing input instead of the next Down. No production warp changed.

Sections have **four PASS cases** in `.local/winter-sections-controller-v01.log`
(the same retained batch contains one separate Cut-fixture failure). Those four
cover82 real entry/return courses,16 wall-stop turns, original ULTRA BALL
PocketText/once and qualified Surf/Waterfall/bridge traversal. The corrected Cut
case **PASS** is `.local/winter-cedar-cut-controller-v02.log` (7.17s): native
denial,66-step ordinary return,6-step opened shortcut and reload. These are
individual passing receipts, not a claim that the failed first batch passed.

Renderer regressions **4/4 PASS** in `.local/winter-renderer-v01.log` (9.03s),
covering actual glyph metadata, icy foreground selection/native-prefix grass,
unchanged animation schedules/pixels/registration and native OAM/cache behavior.
`renders/ice/native-renderer-v01/rustle-review-v01/` retains all24 production
loader RGBA rustle frames and pixel/sight reports. Those are actual loader
assets, not GPU framebuffers; live in-scene timing is a separate claim.

Static native GPU review is **29 views, zero mismatched pixels**:19 main-region
views in `renders/ice/native-gpu-v02/region/recomparison.json`,9 sections views
under `renders/ice/native-gpu-v01/sections/`, and one hut view under
`renders/ice/native-gpu-v01/hut/`. Original production PNGs/raw GPU readbacks
remain in v01. Three region comparisons initially differed because the source
reviewer omitted real two-hop neighbor terrain. The v02 reference follows
native active/depth2/depth1 rectangle precedence and immediate-neighbor camera
clamping, with separate source-resolution checks. It rechecks the unchanged
original GPU images; no production image or game terrain was changed to fit.

Actual reached-state GPU capture is **5/5 PASS** under
`renders/ice/reached-gpu-v01/`, using exact v1 pack `aa1aa251…`, rebuilt renderer
`51d7855c…` and the passing three-case log. The five opaque Saves retain the
17-step landing, hut after real healing, collected cache, stationary ICE wall
stop and both trainer victories. The capture receipt is
`receipt-2eb06e8ef0ed3fc834f6bc18105b912bd3c71ae7d977af1c818a282e47a9dcae.json`;
the separate `personal-review-11fcf8282b35fc701fdb437ddeb685619d661f8881568b86e67d8b9bbc22778c.json`
records inspection of every original and the contact sheet. All five raw GPU
buffers/descriptors (7,372,800bytes), originals, conversions, logs and exact Save
copies remain. Input hashes stayed unchanged; the private executable was removed.

The reached scenes use no map/tile/pose/hour/party override and ordinary
production Save/Continue RTC, which may advance naturally. No source-pixel
equality is claimed for those five scenes. The local legal Mew80/SURF/Fog badge,
replay RNG and initial inventory fixtures remain explicit, including initially
injured/paralyzed/lowPP Mew in the shelter case. No later gameplay state writes
occur. Screenshots supplement the actual controller resource/ownership proofs;
they do not independently prove prior slides, healing or battle outcomes.

Personally viewed the actual hut and all five reached frames. Snow versus
striped ICE, the stop against snowy pines, the cleared cache and the narrow
trainer-bank path read clearly. The5×4 hut correctly has native finite-map dark
fill, warm wood/tatami, adjacent keeper/player tiles and a real south carpet.
The broad frozen reach remains visually repetitive and the139-step snow return
is long; neither test counts nor a small camera establishes blanket Johto
quality. Fresh final audio verification and interactive production-client
acceptance remain separate receipts.

Controller environment bindings for the publisher's test run are
`CRYSTAL_WINTER_REGION_CONTROLLER_PACK` and `CRYSTAL_WINTER_TEST_PACK`, both set
to the absolute final pack path. `CRYSTAL_WINTER_REGION_SAVE_DIR` optionally
retains unique production Saves at actual reached ice/cache/hut/trainer states.
No later gameplay state is written by fixtures.

## Fast source review

Python only reads packs and renders/checks explicit source; all pack publication
above remains Rust. Use a new ignored output epoch for every rerender/proof:

```sh
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/freeze_rainy_river.py \
  --pack content-packs/minnesota-cohesion-harbor-water-v01.crystalpack \
  --output planning/minnesota/renders/ice/region-v01/recheck-v01
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/sections/review_sections.py \
  planning/minnesota/regions/ice/sections/source-v03 \
  planning/minnesota/renders/ice/sections-v01/recheck-v01
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/sections/check_sections.py \
  planning/minnesota/regions/ice/sections/source-v03 \
  planning/minnesota/renders/ice/sections-v01/recheck-proof-v01
```

Observed warm main-region rerender0.41s; complete sections review16.469s after
art load, including34 archived cameras. All original/full-resolution images,
RGBA/RGB conversions, crops, stages, rejected passes and failed publication/test
logs remain retained. Neither source numbers nor rendered pictures establish
that gameplay passes; the concrete controller receipts above establish their
bounded cases. Future v02 acceptance must name its own final identity and runs.

## Accepted source handoff

Canonical R17/R11 layouts and dossiers now contain the reviewed source-v04
winter terrain and explicit scoped native-evidence metadata. Root's immutable
acceptance is `renders/ice/root-native-acceptance-v02/acceptance.json`; the
atomic guarded apply and full outside-byte/readback proof is
`renders/ice/sections-native/canonical-promotion-v04/root-applied-01/receipt-7a40ade707e4d7d057272df0ba06bc00200a2bb6a69a24630089117ed5db05d2.json`.
World, interiors, old story/dossier fields and every unrelated byte span remain
exact. All frozen publisher inputs and previous images/packs remain retained.

## Rust verification limits

Changed leaf files pass pinned rustfmt. The focused region-builder regression
passes after replacing the visited-map insertion with `Entry::Vacant`; native
terrain/program output is unchanged. Final scoped Clippy for both affected
crates, libraries and tests passes in `.local/winter-clippy-scoped-v03.log`
(3m31s), with four explicit pre-existing lint allows and unrelated warnings.
No final diagnostic references the new winter files. The ordinary all-target
Clippy run remains blocked by existing `crystal-core` multiplayer `bad_bit_mask`
at line1332; workspace-wide formatting also retains widespread unrelated drift.
No unrelated reformat or lint fix was applied. Exact evidence hashes and allows
are recorded in `renders/ice/root-native-acceptance-v02/rust-verification.json`.
