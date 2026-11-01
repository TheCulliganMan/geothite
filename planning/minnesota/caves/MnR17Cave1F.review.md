# R17 fitted tor den source review

**Frozen candidate published as native review pack; native preservation/capture PASS.**
Root published actual4891;152 preservation/capture checks and six fullLCD
comparisons pass. The real production-controller case PASS2.32s; Root reviewed and authorized
source promotion, now applied with independent outside-byte proof. No earned Border Pines progression or user
final visual approval is claimed.

Candidate `MnR17Cave1F.candidate.json` SHA256 is
`f590d69e806a4a9e8aae29d03ca2c3b40a386b45ab366755965730ecc67c5437`.
Previous whole source `MnR17Cave1F.previous.json` SHA256 is
`37ba34f06fe3abf8948643510eaeb2b7a7d65994997c61aed09e0e49ce204967`.
The immutable pass is
`../renders/caves/r17-candidate/f590d69e806a4a9e/`: whole224×256 native-art original,
4× whole, six player-size source/CHRIS views, preservation/source proofs, all
12 rejected negative sources, exact source/native contract/reviewer snapshots.
Use `whole-4x.png`, `decision-cameras.png`, `fitted-low-bend-4x.png` and
`worst-overhang-4x.png` for review. The full room is rendered at once.

## Identity and concrete change

This is a short single-floor optional den beneath the border pines. Its granite
tongue joins the east cave wall. The mouth turns west along the fitted low bend,
then the skirt reveals the original Revive tucked beneath a northern overhang.
The useful approach is ten ordinary steps; the short return retraces it. There
is no empty circuit, tiered refuge, duplicate gallery, stair, ledge, field-move
gate, new actor or extra warp.

The old room's central loose boulder sat in a broad56-Floor hollow. The accepted
pass has27 free Floor cells, all reachable with the original item occupied.
Its widest horizontal free run is four gameplay tiles at `(5,10)..(8,10)`, used
by the actual bend, not an empty shoulder. The upper reward pocket is fitted
around the unchanged item. Whole and all decision/widest/worst source views were
personally opened and reviewed; Root separately viewed v04 whole and accepted
the same final geometry for implementation review.

Retained rejected passes:

- v01: a stair-separated nook read as a small refuge; rejected.
- v02: a floating bent tor created two empty equivalent skirts; rejected.
- v03: attached fill used native31 repeatedly and produced obvious false
  ceiling/platform squares; rejected by sight.
- v04: replaced those false squares with closed void and a fitted native2d/2e
  edge; Root viewed this geometry. v05 added honest source CHRIS scale views.
- Frozen f590 adds explicit implementation probes; its geometry is unchanged.

Every original/pre-conversion/crop/resize/RGBA stage and source pass remains
retained in its own folder plus the render archive. Earlier passes were not
rewritten to hide their defects.

## Original native preimage and preservation

The actual installed baseline is
`content-packs/minnesota-northstar-limestone-review.crystalpack`, SHA256
`e549c178f37a3137e8ca7f119aeee79460164f1dbd3ea6d205be8928415fd04c`.
Original whole route/interiors and complete room/native contract were frozen
first under `../renders/caves/r17-candidate/before-e549c178f37a3137/`.
`MnR17Cave1F.native-contract.json` records native attributes, map57/4, preimage
module hash and the original source/native interaction contract.

The cave has no resident NPC. It retains one original `MN_R17_ITEM_04` Poké Ball
at `(5,5)`, `REVIVE` quantity1 and `EVENT_MN_R17_ITEM_04`. Its actual native
`itemball` program remains index0. The separate ranger-hut keeper is untouched.
All non-row/noneditorial whole-source fields compare equal, including the
original item access text, id, quantity, flag, tile, mouth and exit_doors.

Retain native7×8 cave extent/border1, CAVE environment, original
`nite`/`PALETTE_NITE`, MUSIC_UNION_CAVE, LANDMARK_MN_BORDER_PINES, phone18,
FISHGROUP_LAKE, no connections. Inside warp1 `(9,13)` LADDER returns to exterior
`MnR17C00R00` warp2 `(10,44)` FLOOR. Clear indoor Floor is `(9,12)`; exterior
active CAVE warp1 `(10,43)` deliberately enters the room. This is a real original
passive return. No route/hut/port/actor/program/party/story change is proposed.

## Directed collision proof and fast loop

The reviewer reuses SourceArt, native palette/frame helpers, room_sheet and the
render archive. Whole/camera sprites use the actual tile-top-left OAM anchor,
forced NITE and source standing south frames. Centered160×144 source camera
frames are labelled as source previews; native edge-clamping has not been
claimed from these images.

The graph includes reachable native `UP_WALL` at `(3,4)`, `(5,2)` and `(6,2)`.
Native entry masks block entering such a tile Down; native departure masks block
leaving it Up. Horizontal movement and entry Up from the Floor below remain
legal. The reviewer applies both halves, rather than treating those cells as
solid or ordinary unrestricted Floor. All Floor and directional roof cells can
return to the clear arrival; there are no directional traps or unpaired ladders,
and every outer core boundary is WALL.

Twelve meaningful negatives are rejected: original reward identity, event,
quantity and position changes; changed mouth; palette mismatch; severed neck or
low bend; obstructed actor Floor; open outer Floor; extra ladder; and a Floor
above the directional ceiling that is reachable only if UP_WALL departure masks
are incorrectly ignored. Their exact input sources/reasons are retained.

Executed source loop:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/caves/MnR17Cave1F.review.py \
  --pack content-packs/minnesota-northstar-limestone-review.crystalpack \
  --room planning/minnesota/caves/MnR17Cave1F.candidate.json \
  --output planning/minnesota/renders/caves/r17-candidate/f590d69e806a4a9e \
  --self-test
```

The measured positive review is0.112s; full CLI including pack read and12
negative checks is1.732s. No Cargo/export or native pack/source promotion was
performed. Route MN_R17 and shared interiors remained untouched.

## Focused native controller handoff

`crates/crystal-bevy/src/bevy_shell/tests/r17_tor_cave.rs` is authored/formatted,
compiled/run by Root; the first run exposed an observation timing assumption.
The corrected actual case PASS2.32s in `.local/r17-tor-controller-pocket-fixed.log`,
with its receipt bound to exact4891. Test env is
`CRYSTAL_R17_TOR_CONTROLLER_PACK`; optional source env is
`CRYSTAL_R17_TOR_SOURCE`, default exact frozen f590 candidate.

One ignored case,
`r17_tor_cave_controller_bent_route_directional_roof_revive_once_and_continue`,
uses an explicit local passive-mouth/empty-native-party/RNG fixture. All later
steps use the production controller. It proves the actual mouth/clear/passive
return, adjacent ten-step fitted route, stepping onto all three directional roof
cells and returning, actual original Revive grant and retained native found-item
payload, both actually printed PocketText pages/input ownership, once/reentry and
walking onto the collected tile, complete semantic flags/resources and Continue.
No money/badge/report/party writes occur after setup. Revive use is deliberately
left to the already tested native R10 item-use coverage, as Root requested.
Root executed this case successfully on exact4891; it does not claim earned
regional progress or user final visual approval.


## Written native review4891

Root published `minnesota-northstar-tor-review.crystalpack`, SHA256
`4891e6a084e21ea05866740fb04cc8c9891fca2433dbb5a93b58269aed78b72a`,
internal identity `7c67f6ad0770ae69deeb8e547f10efdeb03b2fcca35cc180239a064034ba6baa`.
The read-only comparison against immediate tight-gallery fe57 passes152 checks:
all509 unrelated modules and all other data exact after only owned56blocks and
owned encoded map-block payload normalization. Art/audio/HOME/recovery/programs,
original Revive/object/flag/warp pairs and forcedNITE remain exact.

Six headless production views compare all138,240 LCD pixels exactly, including
terrain, native south-facing CHRIS/item frames and foreground. Whole/source,
actual native contact and actual160 widest/worst were personally viewed. The
private capture executable was removed after capture. Complete evidence and
bounded visual-review receipt are in
`../renders/caves/r17-native/4891e6a084e21ea0/review-01/`.

The first controller leaf/log remain retained. Production controller settling
internally finishes automatic FoundText plus native60-frame fanfare before
returning at the first PocketText input wait. The corrected fixture honestly
asserts its retained native found-item payload/ItemBall phase and both actually
printed Pocket pages/input ownership. It does not claim an intermediate native
Found/Fanfare snapshot. The corrected case PASS2.32s; no production clock changed for a test.
Source route/interiors promotion is now applied as recorded below.


## Guarded promotion preflight and authorized application

After Duluth released the completed R10 source application, exact current full
R17 route and interiors were frozen under
`../renders/caves/r17-promotion/preflight-4891-cf44073e/`. The source-only script
`promote_r17_tor.py` prepares only the R17 room and its interior contract. It
requires exactf590/37ba/4891, native152/six/138240 evidence, real controller PASS
and receipt, and all377 PCM/frame/loop checks. Both full preimages and full
proposals, the owned objects, script/source snapshots and execution evidence are
retained. Independent prefix/suffix byte and complete JSON normalization proof
shows every outside-owned record/byte preserved. The apply path requires the
same full-file hashes, rejects concurrent drift, stages replacements and checks
independent applied readback. Root subsequently authorized the exact reviewed
proposal; apply completed without source drift.

Current reviewed preimages are route
`6347b08de0a54ed5d3a6bb633a545dfd5d6276ebe2eb127dddce60bdfe14cff1`
and interiors
`cf44073ef21dd80315bcb50c3833a93c46073e1170c24e22417f617493d72972`.
The snapshot's `preflight.json` records exact proposed hashes; Root reviews that
concrete proposal before separate apply authorization.


Applied after explicit Root review of freeze90a3d993. Application receipt and
full snapshots are under
`../renders/caves/r17-promotion/preflight-4891-cf44073e/application-1793073681402488000/`.
Only the cave-room object in MN_R17 and its one interiors contract changed.
Independent full readback, prefix/suffix bytes and all-other-record JSON proof
PASS. Actual route after SHA256 is
`1dad0d285e804a2c17ea8b4eac9ea6ce56779426a0931792e254330d5af790e2`;
interiors application-epoch SHA256 is
`92a0862487ae6483adda1ba90d9d7fff15adba6bc2fba0f1b4b5bb5ce2d970b8`.
The frozen candidatef590/previous37ba and all render/failure history remain
unchanged. No Rust/Cargo/publication/STATUS/atlas work was performed here.
