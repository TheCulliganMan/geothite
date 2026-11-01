# Voyageurs glacial cleft

The native510-map composition is
`content-packs/minnesota-northstar-glacial-cleft-review.crystalpack`, SHA256
`1630ad063aa4a1cf252087bdf743ca48a2a8cb6c9c46ad95cb7aceb1c74c3b05`.
It retains the completed a85 healer-refuge composition and replaces only the
existing9×8 R20 cave terrain and its owned block payload, plus derived identity.
All509 other modules, original programs, art, audio and catalogs remain exact.

The dry granite cleft has a short leftward drop between different shelves,
two useful stairs and a lower ordinary connecting slot. Both original mouths,
Hyper Potion at5,5, quantity, pickup event and programs remain unchanged.
The east stair's initial side bypass was closed in v02 before publication.
The whole [native room and nine actual player views](../renders/caves/r20-native/README.md)
were inspected by Root and the author. The original tiny cave is not expanded
into a decorative lake or presented as a faster regional shortcut.

All six assets cases pass, including both native left-hop lanes, a separate
valid opposite-facing geometry probe, boundary/landing/shoulder/return negatives,
whole-data preservation, uncached assembly and actual written round trip.
The opposite-facing probe is a test variant and was never published.
Log: `.local/r20-cleft-native-assets-directional-fixture-fixed.log`.

All three production-controller cases pass (3.45s): both mouth pairings and
genuine passive returns; both useful stairs and wall-side rejection; both
leftward hop lanes, blocked reverse and actual15/13-step ordinary returns;
original fully printed pickup pages, input ownership, actual Hyper Potion use,
once-only event, reentry and Continue. These use explicit local initial fixtures
and do not earn Voyageurs access or the regional story. Evidence is in the
[controller handoff](../renders/caves/r20-candidate/controller-handoff.md).
Log/receipt: `.local/r20-cleft-controller-first.log` and matching `-receipt.json`.

The native report passes150 preservation checks and nine actual headless
captures, including raw GPU linkage and exact terrain/player pixels. All377
Rust audio PCM hash/frame/loop checks pass: `.local/r20-cleft-native-audio.log`.
Earlier failed asset fixtures are retained: parent-directory paths needed
canonicalization; native warp targets use their actual mixed-case constants;
reverse probes must turn before testing a wall; the opposite ledge metatile
was valid safe geometry and is now a positive directional test. These were
fixture corrections, not changes to the accepted candidate.

The original whole source SHA is
`fdf68fbac6dd8da99437cf9541607c59a7bbb3736246cc8694d8775764bd0707`;
frozen v02 is
`05617d4863db095ed7664ce6408f1d7f35e4ce3c6ed360d08bdfd36a35db2577`.
The guarded source promotion is applied. Independent readback proves all bytes
outside the one room and one interior contract unchanged:
`reviews/r20-cleft-promotion/1630ad06-05617d48/application-1793071059204947000/`.
The inactive eastern passive record's own target1 is preserved exactly.
Current source plan validation passes; user visual approval remains open.

Executed publication:

```sh
target/runtime-fps/wasm-dev/revise_authored_cave \
  .local/minnesota-city-revisions/refuges-a85fac64ab31c86722c486b50213053eb16ba3f6453f99e41bd81c962badb980.crystalpack \
  planning/minnesota/renders/caves/r20-candidate/v02/MnR20Cave1F.previous.json \
  planning/minnesota/renders/caves/r20-candidate/v02/MnR20Cave1F.candidate.json \
  content-packs/minnesota-northstar-glacial-cleft-review.crystalpack
```

Replay the ignored assets target `r20_cleft_revision` against the a85 baseline,
and controller filter `r20_cleft_controller_` with
`CRYSTAL_R20_CLEFT_CONTROLLER_PACK` set to the absolute1630 pack above.
Use the pinned toolchain/shared warm profile recorded in [R13's runbook](r13-refuge.md).
The earlier R12 five-case assets regression also passes after directional
validation changed: `.local/r20-directional-r12-regression.log`.
All original renders, rejected versions, source snapshots, crops and raw GPU
buffers remain retained and ignored, as do generated packs and binaries.
