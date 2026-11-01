# R13 healer refuge review

The written510-map review composition is
`content-packs/minnesota-northstar-cave-refuges-review.crystalpack`, SHA256
`a85fac64ab31c86722c486b50213053eb16ba3f6453f99e41bd81c962badb980`.
Its immediate baseline is the completed cave/shore composition SHA
`c16af00c82167c809620b956981ee80641ddc9fd1eeaf1e2c91f59ab16a9cc7d`.
Both exact packs have retained ignored copies in `.local/minnesota-city-revisions/`.

The10×8 healer room now bends around a fitted ceiling buttress. Native stone
stairs lead to Harper's raised care terrace and a separate low supply notch.
It has61 actor-present Floor cells instead of the original143-cell hollow.
All original actors, healing program, two dialogue pages, Max Ether quantity
and event, mouth and passive return are unchanged. No room or service was added.

Open the [whole room](../renders/caves/r13-native/a85fac64ab31c867/review-01/whole-actual-pack-source-night.png)
and [seven actual player views](../renders/caves/r13-native/a85fac64ab31c867/review-01/production-decision-cameras.png).
Root and the author inspected these images, including the widest foyer bend.
Harper's cliff lip uses the native foreground rule and visibly covers four
lower player sprite rows at his approach; this is preserved in the capture.

## Executed evidence

The [native report and capture archive](../renders/caves/r13-native/a85fac64ab31c867/review-01/README.md)
passes145/145 checks and seven headless captures. All161,280 complete LCD pixels
match native art, actors and foreground composition. All509 other modules,
scripts, typed commands, parties, catalogs, audio, HOME and recovery remain exact.
Only R13's80 terrain blocks, their uniquely owned payload and derived content
identity change. There is no new manifest or service installation.

All six assets cases pass, including valid-identity corruption of the original
typed healing command and uncached/written round trips. All three production
controller cases pass: both stairs and real passive return; Harper's fully
printed original pages, movement ownership and healing two injured partners;
original Max Ether pickup, real Pack/Party/Move use, once-only reward and Continue.
These use explicit local initial fixtures; they do not earn regional progression.
The [controller contract](../reviews/r13-refuge-controller-contract.md) records
the exact source hashes, cases and limitations.

Logs are `.local/r13-refuge-native-assets-audio-preserved.log` (6 passed),
`.local/r13-refuge-controller-first.log` (3 passed), and
`.local/r13-refuge-native-audio.log` (all377 PCM hash/frame/loop checks passed).
The initial sixth asset fixture incorrectly reconstructed lazy audio; its failed
log remains. The fixture now preserves actual pack storage and verifies an
unchanged valid control before injecting corruption. Production audio is exact.

The guarded source promotion retained full preimages and independently proved
all bytes outside the one room/contract unchanged. Its receipt is
`renders/caves/r13-promotion/preflight-231a78ae1acda8da/application-1793070111491159000/`.
The current source plan passes; the [complete atlas](../ATLAS.md) was rerendered
from this actual pack and inspected. Final user visual approval remains open.

## Executed publication and replay

```sh
target/runtime-fps/wasm-dev/revise_authored_cave \
  .local/minnesota-city-revisions/refinements-c16af00c82167c809620b956981ee80641ddc9fd1eeaf1e2c91f59ab16a9cc7d.crystalpack \
  planning/minnesota/caves/MnR13Cave1F.previous.json \
  planning/minnesota/caves/MnR13Cave1F.candidate.json \
  content-packs/minnesota-northstar-cave-refuges-review.crystalpack

CRYSTAL_R13_REFUGE_REVISION_PACK="$PWD/content-packs/minnesota-northstar-map-refinements.crystalpack" \
  cargo test --locked -p crystal-assets --features test-fixtures \
  --test r13_refuge_revision -- --ignored --test-threads=1
CRYSTAL_R13_REFUGE_CONTROLLER_PACK="$PWD/content-packs/minnesota-northstar-cave-refuges-review.crystalpack" \
  cargo test --locked -p crystal-bevy r13_refuge_controller_ \
  -- --ignored --test-threads=1

.local/minnesota-review-env/bin/python planning/minnesota/tools/check_plan.py \
  --pack content-packs/minnesota-northstar-cave-refuges-review.crystalpack
```

The completed Rust runs used the shared warm `target/runtime-fps`, one build job,
32MiB Rust stack and `wasm-dev` with `opt-level=0` and offset-allocator override2.
All-target assets/Bevy Clippy with test-fixtures passes with the documented three
baseline lint allowances and pinned-toolchain `useless_conversion` exception;
see the [isolated toolchain diagnosis](../reviews/clippy-useless-conversion-diagnosis.md).
Workspace formatting has unrelated existing failures, recorded in the previous
[composition runbook](northwoods-map-refinements.md). Generated packs, binaries,
original renders, GPU buffers, crops and rejected revisions remain ignored.
