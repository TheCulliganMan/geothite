# Inner Basin bedrock chambers

The current continuation adds the verified
[upper Haulage rock setback](../haulage-rock-clearance-v01/README.md).
The [combined thirteen-stage recipe](../../revisions/reviewed-maps-v01/README.md)
reproduces it as `content-packs/minnesota-reviewed-maps-v02.crystalpack`, SHA256
`110e658e66449fd6ca3dc52fb7de5fa3f0f2bd827d8103aa838a32d63193229c`.
All basin terrain/programs and other maps remain exact in that successor.
The basin-specific parent/child identities below remain reproducible history.

The complete14×14-metatile basin now uses a fitted rock enclosure and a narrow
central rib instead of a square room with rows of loose boulders. Two ordinary
routes run around the rib between the original indicator stations. The three
dials, lever, nine-block drain and voluntary Strandrak encounter remain in their
original positions and retain their original programs. No new field move,
reward, admission rule, staircase or one-way ledge is added.

- [Whole original, closed revision and drained revision](../../renders/inner-basin-installed-v1264/whole-before-closed-drained.png)
- [Reached central indicator](../../renders/inner-basin-native-v1268/basin-center-indicator.png)
- [Reached lower indicator](../../renders/inner-basin-native-v1268/basin-lower-indicator.png)
- [Reached drained encounter chamber](../../renders/inner-basin-native-v1268/strandrak-drained-inner-basin.png)

## Source and candidate lineage

Authoritative frozen proposal: [`source-v03.json`](source-v03.json), SHA256
`13c1312655b95c6fd5c1bb8103a56602d1c13e9a6f0bd7790d8ccb05df1631e4`.
It binds the complete original module, native cave definition, before/after rows,
all controls, real warp pair, nine original changeblocks and parent identity.
[`terrain-v03.json`](terrain-v03.json) is the corresponding complete layout for
the established live source-preview loop. Frozen proposal statuses are history;
this runbook records the verified result without changing their hashes.

Parent: `content-packs/minnesota-ely-pine-camp-winter-v03.crystalpack`, SHA256
`2d4f4e01216ee770f07bebd665a9d96a0badc2f346f802aceaa04f0143c409d6`.
Final child: `content-packs/minnesota-inner-basin-bedrock-v02.crystalpack`, SHA256
`83a63e4791f5de5bd72b8b874426b693bf03e3b2067665bfd1a4f7182aba58ba`.
This is the latest **unselected** continuation candidate. It retains the entire
Ely/winter/story/coast/river parent. `world.json` continues to select
`north_shore_passing_width_revision`; no app/browser release or selector changed.

The first native candidate passed gameplay but left the central floor too broad.
It remains in `minnesota-inner-basin-bedrock-v01.crystalpack` and the v1250/v1254/
v1259 review folders. Source version2's tiny meeting-corner rib was rejected
before publication. Version3 gives that rib proper continuous side faces and
keeps routes around both ends. Every source, full-resolution original, converted
image, comparison phase and rejected pass is retained.

## Native physical and gameplay contract

All coordinates below are16px walking tiles. The existing border changes from
loose boulder block1d to solid native bedrock2e; the outer perimeter is WALL.
No source art or tileset definition changes. The room uses thirteen original
cave pieces, matching the native Slowpoke Well/Union Cave vocabulary studied
in `renders/inner-basin-source-v1250/references/`.

| Feature | Exact position and behavior |
| --- | --- |
| Cave active exit / passive arrival | (3,25) LADDER / (2,25) FLOOR; original warp1/2 |
| Exterior active entrance / passive return | MnFormerShores (18,23) / (18,24), unchanged |
| Upper indicator | WALL (9,10), FLOOR approach (10,10), face left |
| Center indicator | WALL (18,14), FLOOR approach (17,14), face right |
| Lower indicator | WALL (11,18), FLOOR approach (12,18), face left |
| Lever | WALL (14,20), FLOOR approach (15,20), face left |
| Closed sill | (14,8) cannot walk north to (14,7), including adversarial Surf access |
| Encounter approach | (14,3), inaccessible before the original drain acknowledgment |
| Strandrak | Original actor (14,2), voluntary level60 encounter; original capture/retry rules |

The lower ordinary floor reachable from the arrival falls from332 to200 cells.
The outer map dimensions remain14×14; fitted chambers reduce unused floor. All
four controls remain reachable without a field move, and every lower ordinary
floor connects to the exit. The upper pools retain their exact32 WATER quadrants.
The original script still changes only the sill at metatile(7,3) and eight water
metatiles at x4/5/9/10 on rows1/2, all to native floor02. After draining, all287
ordinary floor cells other than the creature body connect to the arrival.

The solution is unchanged: exterior marks must actually be acknowledged and
compared; upper high, center middle, lower low, with all three controls touched.
The initially low lower control is not automatically acknowledged. Incorrect or
untouched settings keep the exit open and spend nothing. The tiles callback
restores the exact original drain after physical entry and Continue.

## Fast source review and guarded Rust publication

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_inner_basin_bedrock.py \
  --output planning/minnesota/renders/inner-basin-source-FRESH
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-ely-pine-camp-winter-v03.crystalpack \
  --source planning/minnesota/caves/inner-basin-bedrock-v01/terrain-v03.json \
  --palette nite --output planning/minnesota/renders/inner-basin-live-FRESH --watch
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_inner_basin_bedrock
target/verdigrim-scene/debug/rebuild_inner_basin_bedrock \
  content-packs/minnesota-ely-pine-camp-winter-v03.crystalpack \
  2d4f4e01216ee770f07bebd665a9d96a0badc2f346f802aceaa04f0143c409d6 \
  planning/minnesota/caves/inner-basin-bedrock-v01/source-v03.json \
  content-packs/minnesota-inner-basin-bedrock-FRESH.crystalpack
```

Use fresh ignored outputs and serialize Cargo with
`.local/minnesota-cargo.slot.lock`. The three-state source review and player crops
take0.10–0.17 seconds after loading. Python changes only an in-memory source view;
Rust publishes packs. New edits require a new reviewed proposal/hash binding.

Strict readback and the full residual guard pass. The independent installed
audit in `renders/inner-basin-installed-v1264/` restores only the owned terrain,
border and their two compiled mirrors, then compares the entire parent. All742
other complete maps, every original program/actor/warp, fishing/wild/trainer
tables, species, original runtime files, title/opening/recovery data and audio
remain exact. The first build's coordinate/border integer types were corrected
to the repository's native types before publication. All failed attempts remain.

## Production input, native views and checks

All three current controller cases pass on the final child in
`renders/inner-basin-controller-v1265/`: actual exterior marks and comparison;
held-dialogue/Continue semantics; all dials and the untouched-default denial;
real drain walking and active/passive stair return; optional refusal, native
Run, withdrawal/Continue and physical retry; selected-PC capture/once-only; and
full-storage refusal retaining the ball followed by actual native retry.
The same child also passes the actually awarded Old/Good/Super Rod course with
zero badges, no Cut/Surf/Waterfall, reachable dry banks, encounters and Continue.

The independent stdio course passes two sessions and389 calls with empty stderr
in `renders/inner-basin-mcp-v1269/`. It begins at an actual closed-arrival
controller checkpoint whose exterior reads were played, tests the blocked sill,
solves all controls and the lever using ordinary inputs, walks the real opening,
exits/reenters through the original mouth and repeats the return after a fresh
process Continue. Its opened source copy only plans inputs; no game state or
pack is rewritten.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/check_inner_basin_mcp.py \
  --pack content-packs/minnesota-inner-basin-bedrock-v02.crystalpack \
  --save planning/minnesota/renders/inner-basin-controller-v1265/strandrak_controller_terraces_all_dials_held_outlet_native_stairs_and_continue/native-saves/basin-closed-arrival.crystalsave \
  --client .local/inland-grass-cut-client-v1163-bin \
  --output planning/minnesota/renders/inner-basin-mcp-FRESH
```

All six reached GPU originals in `renders/inner-basin-native-v1268/` were
visually inspected: arrival, three indicator stations, acknowledged outlet and
drained encounter. They match the native source at the actual pose/facing,
visibility, block overrides and forced night palette: **2,211,840 pixels,
zero differences** in `renders/inner-basin-native-comparison-v1271/`. Each water
frame uses one coherent original animation phase. GPU originals are unchanged.
The review helpers now honor a cave's original PALETTE_NITE when its explicit
time field is empty, and accept actual player facing for comparison. A first
capture/replay attempt started before its checkpoint job finished; those failed
v1266/v1267 attempts remain separate from the completed runs.

Assets and Bevy all-target Clippy pass in `.local/inner-basin-publication-v1263/`
and `.local/inner-basin-runtime-v1253/`; no new warnings point to the changed
leaves. Rust audio/desktop verification passes all602 PCM hashes/frame/loop
records. Wrong-parent and unlisted-row probes refuse without writing a pack
(`.local/inner-basin-final-v1273/`). Owned Rust formatting and Python syntax pass;
workspace formatting still has unrelated differences beginning in offset-allocator
(`.local/inner-basin-format-v1274/`). Native actors and all acquisition tables retain the active Minnesota
roster. Local Pouchid/party/storage/ball and route-position fixtures are explicit;
these checks do not establish earned access, intended-level balance, a fresh KO
or blackout case, or the entire campaign.

Broader cave/mine and outdoor quality review, combined earned statewide
progression and final selection remain open. Existing saved terrain overrides
are not migrated. Do not use the superseded first cave pack as a new parent.
