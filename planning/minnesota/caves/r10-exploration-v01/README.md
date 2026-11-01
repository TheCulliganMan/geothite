# R10 survey cave: complete exploration rebuild

This is the **verified and selected R10 exploration rebuild**, a complete 24×18-metatile replacement for the former cave. Native publication, gameplay and image review passed. Resolve `planning/minnesota/world.json` before publishing; never replace a newer selected parent with the older cave branch.

The complete native-art candidate is 24×18 metatiles. It keeps the real Iron Range Descent mouth at walking tile `(3,13)`, its native stair arrival `(3,13)` and clear north step `(3,12)`, the reader `(9,5)` and the original Revive `(29,23)`. The source uses original cave pixels and the forced NITE palette. The selected parent at authoring start was `iron_crosscut_rock_cache_revision`, pack SHA256 `bc38b8920bdc6d5e3c0e18ddac283aa4087e0020d694861f277d10314a3875d1`.

The route now has a raised survey gallery with two real native stair assemblies, a turning central loop around bedrock, a separate northeast supply pocket, and a lower spring bank. Two downward lanes shorten the loop. Their ordinary walking returns are 13 and 15 steps; the jump courses, reverse and sides must pass the native engine checks too. The spring retains the installed original-Minnesota lake fishing group. No new HM or story gate is proposed.

The optional surveyor SILAS uses Bandrake 40 and Haulgeist 41. Spring angler NORA uses Mudmender 40 and Voyagear 41. Both offer native Yes/No challenges and independent defeat flags. The existing trainer builder provides production battle-return dispatch, active-roster checks and inherited level/evolution/move scaling. An Escape Rope at `(39,5)` gives the far branch a resource; the existing reader/foreman hint and Revive remain. Exact teams, dialogue, approaches, positions and flags are in `trainers.json` and `after.json`.

The opening choice is visible from the mouth: climb to the reader's survey gallery or follow the lower passage around the western pillar. The gallery's second stair rejoins the central turn instead of ending in a decorative platform. SILAS occupies an optional approach beside that turn. The northeast branch ends at the supply cache; the southern branch reaches the Revive, NORA and the spring. The short drops lead toward this lower branch, with an ordinary walking loop back to the gallery. The reader keeps the existing foreman clue and the external story remains unchanged.

Visual review rejected the first wide flat draft, a doubled stair assembly and two isolated floor cells beside the water. The current version has continuous bedrock shoulders, a single correct corner stair, a bounded spring and destinations at the branch ends. The nine player-camera views include both stair landings, the central turn, the drop approach and the spring shelf, rather than only the best-looking room corner.

## Authoritative sources

- `before.json`: frozen former source, verified against the installed native block array during publication.
- `after.json`: immutable reviewed publication input, including its original prototype annotations.
- `current.json`: selected source with the same geometry/actors/review contract; only the purpose and status annotations change after acceptance.
- `contract.json`: complete native module SHA256 and the exact retained exterior warp.
- `trainers.json`: both optional encounters, legal original species, native sprites, choices and postbattle text.
- `plan.json`: route roles, coordinates, ledge returns, required evidence and honest implementation status.

The Rust implementation is `crates/crystal-assets/src/r10_exploration.rs`; `rebuild_r10_exploration` is the command-line publisher. It binds the full native preimage, installs terrain/cache through the existing map publisher, uses the existing regional trainer builder, validates all actual bodies and native ledges, and retains original programs/events. The command accepts `PACK SHA256 BEFORE AFTER CONTRACT TRAINERS FRESH_OUTPUT`. Output must be ignored and fresh. Normal Cargo compilation, strict publication/readback and the stale-module/quantity/foreign-species negative guards passed. The latest candidate is `content-packs/minnesota-r10-exploration-v03.crystalpack`, SHA256 `acc5e63d532de34614cbcc0ac4bd414b190d8cdc9e5d319fdf71f3de3302e073`, rebased onto the accepted Route1 stair-guidance successor, retaining the Brainerd roof fix. Its frozen publication command and parent are recorded in `publication-ready-v432.json` and `publication-v432.log` under the render folder.

## Source and native acceptance

All renders, including rejected passes, remain under `planning/minnesota/renders/r10-exploration-v402/`. The watch loop loads native art once and writes immutable full-room originals and 2× views on every source edit; measured warm renders are about 0.1–0.2 seconds. `latest.json` points to the current preview, and the original Union Cave/Mt. Mortar reference terrain and native metatile atlas are retained there. Earlier haulage sketches are rejected and have no active source-index or publication wiring.

`floor-preflight-v405.log` proves 530 free FLOOR tiles are connected with all five bodies present, all actors have approaches, and the two ordinary ledge returns exist. This is static source evidence, not production input. The first wide, flat draft and the subsequent platform/stair revisions remain intact. Two isolated shoreline floor cells were removed after the source graph caught them.

Nine actual native GPU cameras passed all 3,317,760 pixel comparisons. The published whole-room render matches the reviewed source exactly. Native input phases 415/416/420 and fishing 425 contain 326 recorded RPC calls: both stairs, both jumps/reverse blocks/ordinary returns, reader ownership, both challenge refusals, original Revive and new Escape Rope, actual exterior exit/reentry, Continue, Escape Rope consumption and native fishing/Run. The fishing bank is `(36,28)` facing north into water `(36,27)`; Nora's approach `(33,25)` is a separate west-cliff shelf. The observed native hook was SCUTLET 30. `compound-input-proof-v428.json` preserves the scope and the intermediate harness failures; those individual failure receipts are not relabeled as passes.

`whole-root-audit-v433.json` independently checks the complete declared delta, all 731 other maps, every retained program/event/actor, all dimension mirrors and all art/audio roots. The touched cave's wild/fishing tables, all 318 regional trainer entries and all active evolutions pass original-roster checks. A broader regional audit still finds inherited unrelated `FISHGROUP_LAKE`/`FISHGROUP_SHORE` references; this cave revision does not claim to repair those maps. The strict audio verifier passed all 377 programs' PCM hashes, frame counts and loop ranges.

The current-source controller cases are in `crates/crystal-bevy/src/bevy_shell/tests/r10_exploration.rs`; they also cover physical full-pocket Toss/retry and once-only collection, both real PP-spending victories and Continue. All three cases pass on the final v03 child in `native-results-v434.json`. Both affected crates also pass all-target Clippy in `native-results-v430.json`, with existing unrelated warnings. Use a fresh `CRYSTAL_R10_EXPLORATION_REVIEW_DIR` and the actual published child in `CRYSTAL_INTERIOR_PACK`. Scoped Rust formatting passes; full workspace formatting has existing unrelated differences beginning in `offset-allocator`.

Run the reusable source preview loop from the repository root with the existing renderer Python environment:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-r10-exploration-v03.crystalpack \
  --source planning/minnesota/caves/r10-exploration-v01/current.json \
  --output planning/minnesota/renders/r10-exploration-live --watch
```

The tool requires the existing Pillow and CBOR renderer dependencies. It loads native art once, retains every source and full original/2× render in a new immutable version folder, and updates only a JSON latest pointer. The tested warm render took 0.216 seconds and matched the published whole-room pixels. Source previews do not replace Rust publication or native gameplay review.

Acceptance is recorded in `acceptance-v435.json`: strict publication/readback and negative guards; full-pocket Toss/retry; both real battles and returns; reader/resources/mouth and reached Continue; native fishing; all native 2D camera comparisons and visual inspection; complete pack preservation; scoped roster/evolution legality; audio/art equality; formatting and affected-crate Clippy. Initial local positions, parties, badges and supplies are explicit QA fixtures and do not establish earned regional access. The broader cave/map goal remains incomplete.
