# Iron Range rock clearance

This repairs the boulders drawn through cliff faces in the snowy R09 review.
Twelve rocks move onto the terrace interiors; six excess decorative rocks are
removed. Thirty metatiles change across the four chunks. The native faces,
corners and the pine shoulder replace those old decorative rocks. Existing
grass, native cave stairs, water, rail crossing, Cut gate, actors, encounters,
rewards, programs and rooms remain exact.

`source-v02.json` is the reviewed source. It binds the complete original modules,
their before/after rows, all thirty coordinate edits, the parent identity and
pack hashes. `source-v01.json` and its original renders remain as a rejected
candidate: keeping all thirty-seven rocks created regular rows and an overly
neat crown. V02 retains thirty-one rocks with smaller uneven interior groups.
The six rocks beside the stair and grassy shoulder retain their actual barrier
roles and do not move. The southwest Cut pocket is unchanged.

The north front regains `72`; its lower crown regains `70`. The stepped eastern
face uses the correct native inside corner `6e`. At `(21,17)` the mine shelf
needs closed outside corner `6d`, rather than a floor opening between heights.
The southern crown regains `6b`, with pines behind its original upper edge.
Coordinates here and in the source are global metatiles, not walking tiles.

## Review and publication

The source parent is the unselected ten-map winter candidate:
`content-packs/minnesota-inland-winter-belt-source-v02.crystalpack`, SHA256
`ce8b5ef030d9ff15b9cfe8436e8107a0ca4e327c76cbef02228a45090caa88b1`.
The new child is `content-packs/minnesota-iron-rock-clearance-v01.crystalpack`,
SHA256 `5b6435171e2be96d49cc8a5ebbfc45d8223f5701d202dd71aa79e658b52ab982`,
content hash `4c8caa7bed4f33e48187f1022bc7ddd030091663dd7398c412f18045e930a10c`.
It remains **unselected**; the existing review selector and running app are
untouched. Later Tamarack, descent-thaw and descent-cliff successors retain this
repair. Use the latest child documented in the
[winter-belt runbook](../../regions/inland-winter-belt-v01/README.md) for further
work so reviewed fixes are not discarded. Final latest-story composition is
still a separate integration requirement.

The fast read-only loop renders the whole route, all four complete chunks,
three before/after detail areas and their unconverted originals:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_iron_rock_clearance.py \
  --output planning/minnesota/renders/iron-rock-clearance-NEXT
```

Use a fresh output directory. The measured full V02 loop is 0.253 seconds after
pack loading. Original source views in `planning/minnesota/renders/iron-rock-clearance-v1089/`
were inspected, including the whole route, north crowns, mine apron and haul
toe. `planning/minnesota/renders/iron-rock-native-v1096/` retains nine native GPU originals: four
matched before/after daytime scenes and the exact played stair-return save at
its original clock/facing/state. These were all visually inspected.

Rust-only publication, after building `revise_iron_rock_clearance`:

```sh
target/verdigrim-scene/debug/revise_iron_rock_clearance \
  content-packs/minnesota-inland-winter-belt-source-v02.crystalpack \
  ce8b5ef030d9ff15b9cfe8436e8107a0ca4e327c76cbef02228a45090caa88b1 \
  planning/minnesota/revisions/iron-rock-clearance-v01/source-v02.json \
  content-packs/minnesota-iron-rock-clearance-v01.crystalpack
```

The output must be fresh and ignored. The builder rejects a stale parent,
modified source, incomplete maps, unlisted edits or any unrelated residual
change. Its actual wrong-parent/unreviewed-edit cases both refuse publication
and write no pack. The completed file passes strict readback.

## Verified scope

The static Floor/Grass graph retains every previously reachable ordinary floor
that is not deliberately occupied by a moved rock, both endpoints and both
stairs. It keeps the pre-Cut pocket sealed and all grass/water/Cut/cave quadrants
exact. This conservative graph excludes directional walls and ledges; native
input checks provide the separate movement evidence.

The production stdio tour records 3,532 calls through all four chunks, both
route ends, both stairways in both directions, six wild encounters handled with
Run, Start and actual Save/Continue followed by the stair course again. All
stderr is empty. Initial coordinates, healthy level80 Minnesota partners,
badges and lighthouse signal are declared setup, not earned regional progress.
No position, clock, RNG or progress is rewritten after inputs begin. Original
and played saves and calls remain in `planning/minnesota/renders/iron-rock-route-gameplay-v1095/`.

The production Cut regression separately verifies native refusal, removal,
the original reward, Continue and the native tree reset/re-cut. Mina's original
rightward sightline still triggers her battle below the rim, while the upper
rim crossing stays free of her challenge. All739 other complete maps, original
native art, all602 audio entries/source/PCM metadata, every trainer/team,
encounter, NPC/program, door and reward remain exact. The installed terrain's
2,359,296 pixels match the explicit source with zero differences. The regional
roster audit passes:225 active species,323 trainers and5,454 references.

Four additional production-MCP fixtures test the actual restored north face,
eastern inside corner, mine outside corner and southern crown. Each blocks its
cliff directions while allowing ordinary movement along the adjacent floor.
These are separate declared initial positions with zero badges, followed only
by input; they do not imply earned access to decorative upper terraces. Calls,
original/played saves and empty stderr are retained in
`planning/minnesota/renders/iron-rock-corner-gameplay-v1099/`.

The builder and affected-crate all-target Clippy pass. Changed Rust files pass
rustfmt; whole-workspace `cargo fmt --all -- --check` still reports pre-existing
formatting differences outside this repair. No compiled artifacts, packs,
captures, receipts or verification logs are source deliverables to commit.

This finishes the scoped rock/cliff repair as a local candidate. Tamarack/Rainy
River, fitted thaws, latest composed-parent integration and the broader map/cave
quality review remain open. It does not establish an earned campaign, old-save
terrain migration, deployment or final user visual approval.
