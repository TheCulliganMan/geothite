# R13 Cedar Shelter native acceptance contract

Status: all three production-controller cases and all six assets cases PASS on exact a85 native review pack; guarded source promotion applied after explicit Root authorization.
Source review acceptance is not user approval or proof of earned regional access.

The frozen v02 previous room is
`renders/caves/r13-candidate/v02/MnR13Cave1F.previous.json`
(SHA256 `2dacc5a39541c2093d310042dbd99927adaa9a547e3ed0d0b40e20d2fc532554`),
and candidate is the adjacent `MnR13Cave1F.candidate.json`
(SHA256 `e1e78dad0c18007d3b8b29f72097202193bf141b7d8b611cd6bdc7d0f1352e63`).
The room retains 10×8 native cave metatiles and forced `nite`/`PALETTE_NITE`.
Only its 80 blocks and uniquely owned block payload may differ. The complete
native events, programs, typed runtime commands, art, audio, recovery and HOME
must remain exact.

Root published the native review pack
`content-packs/minnesota-northstar-cave-refuges-review.crystalpack`, file SHA256
`a85fac64ab31c86722c486b50213053eb16ba3f6453f99e41bd81c962badb980`,
from the c16 CURRENT510 refinements baseline. A read-only inspection confirms
Harper's native `special HealParty` command remains index 4, the original
Max Ether pickup remains index 0, and the exterior passive return and following
south tile are ordinary FLOOR. This inspection does not establish input behavior.

## Original gameplay contract

- Room mouth `(7,15)` LADDER and clear arrival `(7,14)` remain exact. Room warp 1
  returns to `MnR13C00R01` warp 2, local `(14,6)` / global `(14,54)` FLOOR.
  Exterior active warp 1 is local `(14,5)` / global `(14,53)` CAVE. The passive
  record is original and is not an inferred automatic step away from a cave.
- Cave `12` at metatile `(5,3)` has right lane `(11,6)` and `(11,7)`, approaches
  `(11,5)`/`(11,8)`, and blocked left neighbor `(10,7)`.
- Cave `36` at metatile `(6,5)` has left lane `(12,10)` and `(12,11)`, approaches
  `(12,9)`/`(12,12)`, and blocked right neighbor `(13,10)`.
- Harper `MN_R13_HARPER` remains at `(13,5)`, approached from `(13,6)` facing Up.
  His exact original script prints two pages, waits, calls HealParty at index 4,
  then closes. Fully printed pages must be `Rest in the rock.\nI will tend to`
  and `your whole team.\nEly is not far.`. Text owns movement before acknowledgement.
- `MN_R13_SHELTER_ETHER` at `(15,11)` grants exactly one `MAX_ETHER`, guarded by
  original `EVENT_MN_R13_SHELTER_ETHER`. No actor, item, flag or reward was added.

## Authored regression leaves

`crates/crystal-bevy/src/bevy_shell/tests/r13_refuge.rs` supplies three ignored
production-controller cases. The tests use legal source-created Mew level 30 and
Totodile level 20 for explicit local fixture setup. Native learned moves remain
unaltered; injured/status/low-PP state is initial setup. Every later traversal,
service, item use, menu acknowledgement, save and Continue goes through the
production controller input path.

1. `r13_refuge_controller_true_passive_return_and_both_useful_stairs_bidirectional`
   enters from the actual exterior, walks both stairs in both directions, rejects
   wall-side input, visits all useful approaches, returns to stable passive FLOOR,
   walks away/back without a warp, and deliberately reenters.
2. `r13_refuge_controller_original_harper_pages_heal_two_legal_partners_and_continue`
   proves both fully printed original pages and movement lock, then exact native
   HP/status/all-move PP recovery for two partners, retained inventory/money/flags
   and Continue. It compares party state before the actual interaction, allowing
   honest poison steps during the preceding walk.
3. `r13_refuge_controller_original_max_ether_restores_native_learned_pp_once_and_continues`
   collects the original item with printed PocketText/input ownership, uses its
   real Pack→Party→Move path through the existing shared PP assertion helper,
   proves exact PP and consumption, leaves/reenters, proves once-only reward,
   then verifies Continue. The R12 helper body is unchanged; only its visibility
   was widened to `pub(super)` for reuse.

`crates/crystal-assets/tests/r13_refuge_revision.rs` has five baseline cases for
complete preservation, genuine uncached production assembly and actual written
readback, source/native service certification, geometry negatives, and exact
actor/door anchors. A sixth case enabled by `test-fixtures` reconstructs an
unchanged valid native control before corrupting the real typed HealParty index
or arguments. It checks valid identity separately so an identity-only rejection
cannot masquerade as service validation. Matching false before/after sources
that remove/change the service or original text must also be rejected.

Root registered and executed these commands successfully (shared warm target/profile flags omitted):

```sh
CRYSTAL_R13_REFUGE_REVISION_PACK="$PWD/content-packs/minnesota-northstar-map-refinements.crystalpack" \
  cargo test --locked -p crystal-assets --features test-fixtures \
  --test r13_refuge_revision -- --ignored --test-threads=1
CRYSTAL_R13_REFUGE_CONTROLLER_PACK="$PWD/content-packs/minnesota-northstar-cave-refuges-review.crystalpack" \
  cargo test --locked -p crystal-bevy r13_refuge_controller_ \
  -- --ignored --test-threads=1
```

Both leaves cache only immutable loaded inputs with OnceLock; each controller,
party/session, save and Continue remains fresh. Source override variables are
`CRYSTAL_R13_REFUGE_SOURCE`, `CRYSTAL_R13_REFUGE_PREVIOUS_ROOM` and
`CRYSTAL_R13_REFUGE_REVISED_ROOM`. Do not point the assets baseline at the already
revised a85 room. No source promotion is authorized by this document.

## Executed evidence and held promotion preflight

Root's first assets run `.local/r13-refuge-native-assets-first.log` passed all five
ordinary cases. The sixth rejected its unchanged reconstructed control because
the synthetic-data fixture constructor normalized lazy native audio into PCM,
leaving no embedded `CRY_AIPOM` bytes. This was a fixture-construction failure,
not evidence of native service drift. The scoped correction retains the complete
original serialized pack, replaces only its typed data for the deliberate
negative, and refreshes identity through the existing test fixture method. It
requires whole unchanged-control equality and full runtime/audio verification.
Root reran the corrected sixth case with all six assets cases PASS in 13.79s, `.local/r13-refuge-native-assets-audio-preserved.log`. The valid-audio unchanged control and malformed native HealParty index/argument negatives all pass. All three controller cases PASS in 4.03s in `.local/r13-refuge-controller-first.log`.

Bemi completed the separate written-pack/capture proof on exact a85, retained in
`renders/caves/r13-native/a85fac64ab31c867/review-01/`:
`native-verification-and-captures-35e6517001bbad268db9804875dd89d6b4637b0bf52df0eaf52f36030991bea3.json`
records 145/145 checks and seven production captures. This covers all 509 other
modules, the full unchanged Harper native service/program/index/warp records,
and exact forced NITE/source pixels including native cliff foreground. The separate actual-input service cases now also PASS.

Source-only promotion script `caves/promote_r13_refuge.py` passed preflight with
`applied:false` against immutable freeze
`renders/caves/r13-promotion/preflight-231a78ae1acda8da/`. It freezes the complete
current route/interiors, original room, candidate and exact combined145 proof.
Only the owned cave room and its owned global interior contract may be spliced;
every byte outside those decoded objects is preserved. Exact current preimages:

- `routes/MN_R13.json`: `4cba536583bae8e8dc021641b4fe3b63b46edf8a467c8d014816e59691704564`
- `interiors.json`: `2d901cf7150a9a1e59dfb5527a30cd1ed9194fa6b4d683a71f214fe45b37c1c0`

The script checks frozen source, exact a85 pack, all145 preserved native checks,
original metadata/service/reward/door retention and unchanged owned preimages.
`--apply` additionally requires all three passing production-controller case
logs. Application was held until Root supplied all three passing cases and explicit authorization. It has now been applied; candidate/render files were not rewritten. The previous freeze
selected the earlier143-check comparison and correctly rejected before any
shared write; it and its rejection receipt remain retained.


## Applied source promotion

After all3 actual controller, all6 assets, native145/seven capture and audio377
checks passed, Root explicitly authorized application of frozen preflight
`231a78ae1acda8da`. The application is retained in
`renders/caves/r13-promotion/preflight-231a78ae1acda8da/application-1793070111491159000/`.
Its `promotion.json` retains exact before files and controller log, and
`outside-owned-byte-proof.json` independently confirms every byte outside the
owned route room/global interior record is unchanged. The successful assets log
is also copied alongside the application. Final authoritative source hashes:

- `routes/MN_R13.json`: `a53a4f92780d9a04c833c2ac3a5579b29aad31e94373663aed7f98f0ab3036fe`
- `interiors.json`: `8d932177895dc14721c44c1bd25ba50e7cb09a757839597b5d69d7dd15ecb93b`

The original whole Harper actor/text/native special index4, item identity,
quantity, event/program and all noneditorial/nonterrain room data remain exact.
Rows match the unmodified frozen e1e78 candidate. The genuine passive warp2
return to `(14,6)` local / `(14,54)` global remains FLOOR and stable under actual
input. Both useful native stairs work in both directions, and Harper's two
printed pages precede native healing of HP/status/every move's PP. The real
Max Ether grant, native move selection/use, once-only persistence and Continue
were proven through the production controller. No production Rust changes were
needed in this R13 phase, except approved visibility reuse of the unchanged R12
PP test helper.

These are local legal party/position/RNG fixtures. They do not prove earned
Q08/badges/survey/League progression, arbitrary historical-save migration, or
final user visual acceptance. Root owns shared plan checks, STATUS/runbook,
atlas-latest updates and lint; this record does not claim those subsequent
shared checks have run.
