# Brainerd lakeside street and wooded loop

This complete32×24 town revision moves the existing camp-supply cabin onto the
lake street, closes its abandoned southern spur with pines and fits the ranger
court and south boulevard to their useful paths. The lake, Gym terrace and
regional dimensions stay intact. This is now the **selected local review**, reproduced as
`content-packs/minnesota-reviewed-maps-v07.crystalpack`. `world.json` selects
`brainerd_lakeside_loop_revision` after the Mankato v06 review.

The ordinary eastbound street had native49 railing across its second walking
lane at y25. The revised span sits two walking tiles farther south against the
forest margin, leaving both y24/y25 lanes clear. The pier's north cap repeated
left corner40 at both ends. Native42 now closes its right corner and opens the
proper second deck column; the three original deck/rail rows below remain exact.
These are native collision and assembly repairs, not edited render pixels.

- [Complete town](../../renders/brainerd-lakeside-source-v1387/after-whole-original.png)
- [Whole before/after](../../renders/brainerd-lakeside-source-v1387/before-after-original.png)
- [Eight native player views](../../renders/brainerd-lakeside-native-pixels-1793518019396975000/production-decisions.png)
- [Current immutable authoring source](source-v04.json)
- [Exact native camera positions](views-v01.json)

## Useful trips and passing room

The outfitters now sit beside the public pier approach and lake board, where
their existing gear gift is useful. The move removes a mostly empty southern
walk without shrinking the lake or creating a second decorative pond.

| Dry walking trip | Before | After |
| --- | ---: | ---: |
| Outfitter return to pier mouth |22 |8 |
| Outfitter to Mart |48 |34 |
| Outfitter to ranger station |50 |36 |
| Clinic to pier mouth |40 |44 |
| Clinic to ranger station |20 |20 |

These shortest paths count occupied native actors. The clinic-to-pier walk grows
by four steps around the attached grove; this is a deliberate tradeoff for a
wooded street edge, not a claim that all walks become shorter. The five existing
buildings, twelve exterior actors and seven posts remain part of the town.
Only the outfitter's two exterior warp coordinates move. No new encounter grass,
Cut tree, ice, admission, train station or story gate is added.

All coordinates below use16px walking tiles unless marked metatiles.

| Destination | Active door | Passive exterior return | Warp indices |
| --- | --- | --- | --- |
| Clinic, unchanged |(9,21) |(9,22) |1 /2 |
| Mart, unchanged |(19,21) |(19,22) |3 /4 |
| Gym, unchanged |(12,35) |(12,36) |5 /6 |
| Ranger station, unchanged |(23,15) |(23,16) |7 /8 |
| Outfitter |(51,23) |(51,24) |9 /10 |

The complete original cabin14/15 moves from metatiles(23,17) to(25,11).
Its actual interior, passive arrival(6,5), both native carpets(6,7)/(7,7),
keeper(3,3), helper(9,4), SUPER_REPEL×1 gift and
`EVENT_MN_BRAINERD_CAMP_REPEL` stay exact. Capacity refusal and repeat programs
remain unchanged. This terrain revision never grants the gift or progress.

The principal clear public rectangles are west street x8–31/y24–25, east street
x38–59/y24–25, outfitter street x50–55/y24–25, pier mouth x47–48/y20–21 and
south boulevard x32–33/y28–45. The walker at(36,25) retains a turnout at the
junction. He is outside the two declared straight street spans. The ranger at
(25,19), camper(9,12), fisher(48,16), outfitter(51,29), Gym spectator(16,37)
and all six checkpoint wardens retain their exact native positions/programs.

The pier cap is40/42 at metatiles(23,6)/(24,6). Its correct deck columns47/48
meet at y13; the original three rail rows y7–9 remain byte-exact. The fisher
occupies(48,16); column47 provides the ordinary passing route. The open dry
casting/launch bank is(44,20), facing north toward water(44,19).
`FISHGROUP_MN_SHORE` remains unchanged: the Old Rod has SCUTLET/CLINKLAW.
The contained lake's complete WATER/shore blocks remain exact. The cabin uses
four former dry shoreline squares, while the declared bank and pier access stay
open. No Surf, Cut or badge is required merely to cast from the public bank.

## Story, cliffs and actual connections

The original Brainerd sixth-Gym/report/Surf sequence remains authoritative.
The outside ranger still requires the Lake badge for his existing reward/report;
this revision changes none of those programs. The outfitter's west-road clue,
camper's pine shelter dialogue and fisher's lake observations remain exact.
No regional admission or badge is supplied by the publisher.

The complete Gym terrace/stair/return rectangle remains byte-exact at metatiles
(3,15,10,7). The actual climbing column21 retains upper39, treads40/41 and
lower42. Its side walls and closed east cliff retain their native geometry;
there is no new ledge on a cliff. No regional mouth or connection offset changes.

| Road | Existing connection | Native seam pairs |
| --- | --- | --- |
| West / Detroit Lakes road |MnR15C00R00, offset0 |town(0,24/25) ↔ route(63,24/25) |
| East / Mille Lacs road |MnR18C00R00, offset7 |town(63,24/25) ↔ route(0,10/11) |
| North / Leech Lake road |MnR07C00R01, offset6 |town(32/33,0) ↔ route(20/21,47) |
| South / St Cloud road |MnR06C00R00, offset−5 |town(32/33,47) ↔ route(42/43,0) |

West, east and north retain their Lake badge checkpoint planes and free inbound
behavior. The original Fly/recovery spawn512 remains(9,22), in front of the
unchanged clinic. Wardens, coordinate events and all map perimeter cells are
preserved. Static source reachability is separate from native guard/seam proof.

## Fast full-town loop and native publication

The frozen parent is `content-packs/minnesota-reviewed-maps-v06.crystalpack`,
SHA256 `f4cd87d34fba9696d67c9658052976b9f7ba2d9689371603fa358685b5592547`.
Its complete native town preimage has canonical JSON SHA256
`fbba14c471ac288f763a97ccc4fc23e59de12c2bbb0cd4d2ccd6d88a636b3dcb`.
Source-v04 has SHA256
`8ad8cd5a1e7347235009e103f183eb948d3218f66263a19b5c5d258125032bee`.
It changes57 metatiles and two exterior warp positions. All older sources,
working renders, failed outputs and published candidates remain retained.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_brainerd_lakeside_loop.py \
  --source planning/minnesota/towns/brainerd-lakeside-loop-v01/source-v04.json \
  --output planning/minnesota/renders/brainerd-lakeside-FRESH
```

The preview validates the complete source preimage, intact cabin, protected
terrace/pier/checkpoint planes, perimeter, every actor/post/door/recovery
approach, public lanes and dry fishing bank. It renders the whole town with
original native art. The measured source pass takes0.206 seconds after loading.
`proposal.json` is the editable copy; publication consumes the frozen source.
Use fresh ignored outputs and retain all originals before any conversion.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_brainerd_lakeside_loop
target/verdigrim-scene/debug/revise_brainerd_lakeside_loop \
  content-packs/minnesota-reviewed-maps-v06.crystalpack \
  f4cd87d34fba9696d67c9658052976b9f7ba2d9689371603fa358685b5592547 \
  planning/minnesota/towns/brainerd-lakeside-loop-v01/source-v04.json \
  content-packs/minnesota-brainerd-lakeside-loop-FRESH.crystalpack
```

The executed current candidate is
`content-packs/minnesota-brainerd-lakeside-loop-v02.crystalpack`, SHA256
`d94e6ccfa6460f58730fff0e5b23a4167a050cc0709214625af9199718e70a7f`,
content identity `6a9deafeeb226577012ec95864778686c0d047fccdccdbde9fcea986682bbbad`.
Rust publication and strict readback pass, with all602 audio/desktop checks.
The independent full-root projection in
`renders/brainerd-lakeside-preservation-v1388/receipt.json` preserves every742
other complete maps, all scripts, rooms, actors, roster, encounters, fishing
catalog, art/audio and the original legacy attribute table. The original town's
private stair alias lives in the typed module while its old global attribute
record names Johto; the first publication attempt correctly rejected accidental
normalization of that unowned record. The scoped publisher now preserves it
exactly, and the full inverse recovers the parent.

Eight complete640×576 clear daytime native views have zero differing pixels in
`renders/brainerd-lakeside-native-pixels-1793518019396975000/`. The whole source,
complete camera sheet, ordinary central junction and outfitter street were
personally inspected. These are explicit local CINDORE/hour10 fixtures, not
proof of an earned regional visit. Four current native input cases pass:

- `renders/brainerd-lakeside-controller-1793518306551241000/receipt.json`:
  the first two entries pass all five buildings/ten carpets/seven posts and
  the road/pier/rail/fishing/Continue course. The third entry is a retained
  failure from the initial one-HP poison service fixture and is not counted.
- `renders/brainerd-lakeside-care-controller-1793518444300164000/receipt.json`:
  native care restores actual HP/status/PP, the relocated cabin grants its
  original Super Repel once, repeat/Continue and both exits work; the separate
  original Gym cliff/stair/Continue case also passes. The revised initial
  injured partner has enough HP for the actual poisoned walk to the nurse;
  restoration assertions and all subsequent native input remain unchanged.
- `renders/brainerd-lakeside-stdio-v1391/receipt.json`: three independent
  processes,845 actual RPC calls and byte-empty stderr. The unedited producer
  saves drive the public town loop, seven posts, north gate refusal, both bank
  and road approaches, registered Old Rod animation, real regional fish, RUN,
  fresh-process Continue and Start. The preceding v1390 harness selected stale
  Mankato room names before movement; its failure remains retained.
- `renders/brainerd-lakeside-played-native-1793518450948288000/receipt.json`:
  four personally inspected native views from actual producer saves, without
  position, clock, facing, partner or state overrides. Actual night/rain is retained.

All new local companions are original CINDORE; the dry fishing case has zero
badges and no field moves. Rod/position/injury setup is declared fixture data,
not earned access or first-route balance. No state is rewritten after setup.
Assets/Bevy all-target Clippy and owned formatting pass. Workspace formatting
still reports pre-existing differences, beginning at the offset allocator.
Seven publisher refusals reject unsafe parents, sources and outputs.
## Selected combined review

The [eighteen-stage recipe](../../revisions/reviewed-maps-v01/recipe-v07.json)
reproduces the standalone candidate byte for byte as
`content-packs/minnesota-reviewed-maps-v07.crystalpack`, with the same SHA256 and
content identity listed above. The new master also reproduces v06 exactly.
Its [source index](../../revisions/reviewed-maps-v01/sources-v07.json) binds
30 native layouts, six route groups, 23 sources and 26 town anchors. Eight
corrupted-index cases and six master publisher refusals pass, in addition to
the seven standalone refusals. The older v06 source audit still passes.

- `.local/brainerd-reviewed-build-1793518729604502000/receipt.json`: build,
  exact v07 reproduction, exact v06 compatibility and all-target assets Clippy.
- `renders/brainerd-reviewed-sources-v1392/receipt.json`: complete current
  rows/aliases/borders, Safari and town anchor ownership.
- `renders/selected-brainerd-lakeside-loop-v1394/receipt.json`: local selection,
  exact preceding world, counted successful cases and complete proof hashes.
- `renders/early-fishing-brainerd-child-1793518730736923000/receipt.json`:
  actual zero-badge Old Rod gift/casting/encounter/Run/Continue and all three
  guru house/gift/repeat courses pass on byte-identical current content.

The whole installed town, eight native fixture views, four played night/rain
views and the refreshed 108-map atlas were personally inspected. Static atlas
checks retain three seam groups with no overlaps, contradictory cycles or
missing hubs; they do not establish earned regional travel. All 742 other maps
remain exact, including early fishing and the green Boundary Waters preserve.
This selects a local review; it does not publish a browser build, migrate saves,
complete a campaign or establish final user visual approval. Frozen source status text records its authoring epoch; `world.json` resolves
selection. The broader city/cave quality and earned regional progression goal
remains open.
