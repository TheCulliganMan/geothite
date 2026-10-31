# R10 survey-cave rock spurs

The complete 24×18 survey cave now has a connected northern bedrock spur and a
bent supply hollow. These replace broad open floor and isolated stones while
keeping the established cave activities. The west and right-corner gallery
stairs remain useful, both short drops retain their ordinary returns, and the
existing Escape Rope still rewards the northeast branch. SILAS, NORA, the
reader, Revive and spring keep their exact positions and native programs.

`world.json` controls selection. The local candidate is
`content-packs/minnesota-r10-survey-spurs-v02.crystalpack`, rebased onto the
newer Detroit Lakes gate parent so that both regional changes are retained.
This is a complete-room terrain refinement within the ongoing state review,
not a claim that every cave or map is finished.

[Whole cave before/after](../../renders/r10-survey-spurs-v681/audit-v694/whole-before-after.png),
[twelve native player views](../../renders/r10-survey-spurs-v681/native-v693/MnR10Cave1F-production-cameras.png),
[actual reached-save views](../../renders/r10-survey-spurs-v681/acted-v697/actual-reached-views.png).

## Sources and native reference

`before.json` binds the original selected exploration cave. `after.json` is the
immutable publication input: fifty explicit metatiles change, without changing
dimensions. `contract.json` binds the full original native module and mouth.
`plan.json` records the exact paths, stairs, drops, wall checks, actors and
acceptance contracts. Original Union Cave and Mt. Mortar terrain and the actual
native cave metatiles were inspected before authoring; no new art or palette
was generated. The room retains its forced NITE palette.

The fast source renderer loads original art once and preserves every whole-room
version and original PNG. The measured warm revision took 0.081 seconds.
Use a fresh output directory:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-r10-survey-spurs-v02.crystalpack \
  --source planning/minnesota/caves/r10-survey-spurs-v01/after.json \
  --output planning/minnesota/renders/r10-survey-spurs-next-source --watch
```

All source versions, reference images, rejected inputs, native originals and
pre-rebase evidence remain under `planning/minnesota/renders/r10-survey-spurs-v681/`.
The earlier R10 exploration sources and renders remain historical evidence;
their original actors, rewards, scripts and mechanics continue unchanged.

## Playable geometry

- The real mouth remains `(3,13)`, with clear north step `(3,12)`, paired to the
  original `MnR10C00R00` entrance and safe return at `(36,18)`.
- The northern spur is attached to the ceiling. Its west face blocks Right
  from `(20,13)`. The walking route turns beneath its southern toe at `(25,21)`
  before climbing toward the bent supply hollow. Its rock nose blocks Up from
  `(35,8)`; the ordinary bend leads to the supply approach `(38,5)`.
- The original west stair uses native30 at metatile `(4,3)`. The right-corner
  stair uses native36 at `(10,3)`. Both are climbed and descended through
  production input; neither is a decorative stair beside a bypass.
- The two existing drop lanes retain approaches `(14,15)` and `(15,15)`, lips
  at y16 and landings at y18. Reverse and side courses remain blocked; their
  ordinary returns are still thirteen and fifteen walking steps.
- All 471 unoccupied ordinary FLOOR tiles form one dry component with all
  five actual bodies present. Every original actor and cache has a reachable
  approach. No new ladder, HM gate, story admission or water exit is added.
- The original spring blocks and casting bank `(36,28)` facing north toward
  `(36,27)` remain exact, including `FISHGROUP_MN_LAKE`. This revision freshly
  proves dry access to the bank; it does not claim a new fishing-bite replay.

The two optional trainer teams and scaling remain authored in
`../r10-exploration-v01/trainers.json`. Both actual battles, PP use, prizes,
defeat flags and Continue are exercised by the existing controller cases.
The reader's foreman clue and both once-only rewards remain exact. Full-pocket
refusal, production Toss/retry and collection after Continue remain covered.

## Publication and verification

Only the cave's complete block array, its uniquely owned encoded payload and
derived pack identity change. The Rust leaf is
`crates/crystal-assets/src/r10_survey_spurs.rs`; the guarded publisher is
`revise_r10_survey_spurs`, accepting `PACK SHA256 BEFORE AFTER CONTRACT OUTPUT`.
It checks the whole preimage, unchanged retained source fields, original anchors,
protected stair/water/drop blocks, all real bodies, full native floor/ledge
geometry, split payload and strict pack readback. Output must be fresh and ignored.

The executed rebase used the selected Detroit gate parent:

```sh
planning/minnesota/renders/r10-survey-spurs-v681/publisher-v683 \
  content-packs/minnesota-current-r19-detroit-gate-v05.crystalpack \
  b57516c0b77c7bcf94fa35e60ff8300bfd8a8781782827739742d8fc15b3f437 \
  planning/minnesota/caves/r10-survey-spurs-v01/before.json \
  planning/minnesota/caves/r10-survey-spurs-v01/after.json \
  planning/minnesota/caves/r10-survey-spurs-v01/contract.json \
  content-packs/minnesota-r10-survey-spurs-v02.crystalpack
```

Resolve the current world parent before publishing again. Do not use that
historical command to overwrite newer regional work. The first-parent v01 pack
and all its evidence remain retained at their original bindings.

`audit.py --parent PACK --pack CHILD --output FRESH_DIRECTORY` independently
reconstructs the complete authorized delta, checks all 732 other maps and every
other root, verifies the full dry graph and short-drop returns, and audits the
touched cave's encounters/fishing plus all 320 regional trainer teams and active
evolutions against the 225-species curated roster. All native art/audio, events,
programs, items, dimensions, seams, exterior terrain and palettes remain exact.

The three existing default-profile `r10_exploration_` controller cases pass on
the rebased pack. Their explicit local fixture supplies the original Minnesota
party, seven prior badges/depot clearance and supplies before input. Subsequent
battles, resource transactions, stairs, jumps, mouth travel and Continue use the
production controller; these fixtures do not prove earned regional arrival or
intended-level balance.

`check_mcp.py` independently walks the new spur and supply bend, proves both new
wall faces, both stair flights and short drops, original reader input ownership,
both challenge refusals, actual exterior travel and fresh-process Continue.
It starts from the controller's reached save with both rewards collected, uses
only production stdio input, and keeps stderr byte-empty. Native read-only save
inspection checks that neither optional defeat flag nor any new reward is granted.
The rebased replay passes 2,923 RPC calls, including a fresh-process return from
the real exterior. Its native reward/defeat readings and byte-empty stderr are
retained under `mcp-v696/`.

All twelve native camera/source comparisons pass at the complete 640×576 frame.
All 377 audio programs retain their PCM hashes, frame counts and loop ranges.
Affected-crate all-target Clippy and changed Rust formatting pass. Workspace
`cargo fmt --all -- --check` still reports pre-existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`; those unrelated files are preserved.

`current.json` is the verified source with identical terrain and retained
authoring fields; only its status annotation changes from the frozen input.
`sources.json` composes it over the latest selected regional source index.
The shared interior reservation changes only source/proof metadata; original
mouth, clear arrival, outside entrance and safe return coordinates stay exact.
