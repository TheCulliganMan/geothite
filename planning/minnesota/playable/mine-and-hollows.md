# Mine and introductory hollows

The completed510-map review composition is
`content-packs/minnesota-northstar-limestone-review.crystalpack`, SHA256
`e549c178f37a3137e8ca7f119aeee79460164f1dbd3ea6d205be8928415fd04c`.
It retains the R20 cleft, R13 refuge, R12 ascent, R10 gallery, shore bends,
Ryan's house, current city links and existing regional story. Generated packs
have exact ignored copies in `.local/minnesota-city-revisions/`.

## Survey mine

R09's12×11 room now uses three cut benches, a fitted entrance climb, a scout
corner and two short downward hop lanes with9/11-step ordinary returns.
All original actors, text, native trainer party and quest programs are retained.
The [whole room and nine actual player views](../renders/caves/r09-native/b3b0c67847b6fa2a/review-01/README.md)
were personally inspected by Root and the author, including the widest upper
bench. The source candidate is243f04da49a01f59… .

The actual intermediate mine pack is
`content-packs/minnesota-northstar-mine-cleft-review.crystalpack`, SHA256
`b3b0c67847b6fa2ac1f65ba0e5075a55f5b7372657a9d69c7193b2a41ba15e69`.
The final limestone step preserves its whole R09 module and repaired mirrors.
Five assets cases, four controller cases,199 preservation/pixel checks,
nine headless production captures and all377 audio checks pass.

The existing foreman battle, real win/loss, prerequisite denials and separate
instrument recovery were replayed on the revised map. Victory sets only defeat;
the cache's separate acknowledged Yes sets recovery; No is retryable. Repeat
interactions, original scout pages, input ownership, passive return, all three
stairs, both hops/ordinary returns and Continue pass. The fixed local fixture
does not earn the seventh badge or earlier report. Exact evidence and all
original indices/party fields are in the [controller contract](../reviews/r09-mine-controller-contract.md).

The prior story overlay contained a real publication discrepancy: its cached
mine held the foreman/cache, while global split data still held only the scout.
The dedicated Rust revision certifies that exact historical preimage and repairs
only19 original cache-owned roots plus the two NPC/event mirrors. The cached
story itself is unchanged. Uncached reconstruction must then equal the whole
original native module, with its two raw transport section labels included.
Other stale/partially repaired payloads and typed/party/flag drift are rejected.
The ordinary cave API still rejects unsupported story actors.

Executed logs: `.local/r09-mine-native-assets-corrected-negative.log` (5 passed),
`.local/r09-mine-controller-geometry-first.log` (1 passed),
`.local/r09-mine-controller-story-first.log` (3 passed), and
`.local/r09-mine-native-audio.log` (377 passed). The initial assets run's optional
scout-stair removal was safe alternative geometry; the corrected negative blocks
the real scout standing floor. The failed test/source are retained.
The guarded room/contract promotion is applied with independent outside-record
byte proof at `renders/caves/r09-promotion/preflight-9e922e540b3ab0ea/application-1793071697845541000/`.

## Introductory limestone hollows

R01 stays8×7 with its original guide, Antidote and one mouth. Two different
short stairs lead from the entrance choice to a guide hollow and a tapered
discovery alcove. A small fitted ceiling cut replaces the first pass's broad
right rectangle; there is no added ledge, encounter, actor or field-move gate.
The [whole room and six actual player views](../renders/caves/r01-native/README.md)
were personally inspected by Root and the author. Source candidate214bd238…
passes its fifteen source negatives and147 native preservation/pixel checks.
All509 other modules and all data outside its56 blocks/owned payload are exact.

Both production-controller cases pass: both useful stairs and wall sides,
original active/passive mouth and Continue; all three fully printed guide pages,
movement ownership, original Antidote pickup, native poison cure through Pack,
consumption/once-only event, reentry and Continue. The final case also walks
onto the collected object's former tile. Its first run expected a turn-first
Right input to move immediately; the corrected test explicitly turns before
walking and retains the strict tile/repeat assertion.
Log: `.local/r01-hollows-controller-turn-fixed.log` (2 passed,4.76s).
Local full-bag retry is untested; the existing R10 shared engine case is separate.
All377 final-pack PCM hash/frame/loop checks pass in `.local/r01-hollows-native-audio.log`.

The room and interior contract are promoted. Full preimages/proposals and
independent exact unrelated-byte checks are retained at
`reviews/r01-hollows-promotion/e549c178-214bd238/application-1793071849544765000/`.
Fast source review remains independent of publication; the executed
`caves/MnR01Cave1F.review.py --self-test` loop took1.769s. Every original,
rejected render, source preview, native screenshot, GPU buffer, crop and receipt
is retained. Final user visual approval and regional earned progression remain open.

## Executed publication

```sh
target/runtime-fps/wasm-dev/revise_r09_mine \
  .local/minnesota-city-revisions/cleft-1630ad063aa4a1cf252087bdf743ca48a2a8cb6c9c46ad95cb7aceb1c74c3b05.crystalpack \
  planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.previous.json \
  planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.candidate.json \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-mine-cleft-review.crystalpack

target/runtime-fps/wasm-dev/revise_authored_cave \
  .local/minnesota-city-revisions/mine-b3b0c67847b6fa2ac1f65ba0e5075a55f5b7372657a9d69c7193b2a41ba15e69.crystalpack \
  planning/minnesota/renders/caves/r01-candidate/214bd2388b04598e/MnR01Cave1F.previous.json \
  planning/minnesota/renders/caves/r01-candidate/214bd2388b04598e/MnR01Cave1F.candidate.json \
  content-packs/minnesota-northstar-limestone-review.crystalpack
```

Replay `r09_mine_revision` with test-fixtures against the1630 baseline;
`r09_mine_controller_` and `iron_range_story_controller_` againstb3;
`r01_hollows_controller_` againste549. Each leaf names its explicit pack
environment variable. Scoped Rustfmt passes. Assets/Bevy all-target Clippy with
test-fixtures passes with the documented baseline allowances and
[pinned-toolchain lint exception](../reviews/clippy-useless-conversion-diagnosis.md);
log `.local/r09-r20-r01-all-target-clippy.log`. Unrelated workspace formatting
failures remain documented in the earlier runbook.
