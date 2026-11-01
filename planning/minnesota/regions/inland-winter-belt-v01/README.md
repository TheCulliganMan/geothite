# Connected inland winter belt

The requested cold country includes Bemidji and Grand Rapids, the Iron Range
between them, Border Pines and Tamarack/Rainy River. Ely remains snowy in pines.
Boundary Waters, its Safari and entry camp, Grand Marais, its cedar approach and
the North Shore stay green. Rain remains an occasional warm-outdoor effect.
This is a complete region requirement; the ten-map art candidate below is one
stage, not completion of that requirement.

## Current authoritative source

`winter-belt-source-v04.json` replaces the earlier isolated core/border art
proposals. Those older files and every original render are retained as history.
Do not install their map rows over the repaired Bemidji town.

The source parent is the reviewed roof/cliff candidate
`content-packs/minnesota-bemidji-roof-plateau-v01.crystalpack`, SHA256
`d14d30be991a78440e802bae1b5b5670be837e9c2dddef7017978952c3215fdd`.
Its parent is the selected coastal passing-width revision. The proposal binds
both parent identities, complete map module hashes, source art file hashes and
exact rows. It retains Bemidji's full original timber roof and closed east
plateau face, indexed door and useful south staircase.

The authored stage covers all of Bemidji, its Northwoods link and Grand Rapids,
all four R09 Iron Range chunks, and all three R17 Border Pines chunks. Map
shapes, walking collision, original block IDs, scripts, actors, doors, spawns,
encounters, rewards and native Cut/Whirlpool replacements stay unchanged. Snow
is ordinary floor. The existing 92 R17 ice quadrants remain; none are added.

| Private alias | Existing behavior source | Icy encounter block IDs |
| --- | --- | --- |
| `mn_winter_bemidji` | `mn_bemidji_plateau_stairs` | None currently placed |
| `mn_winter_bemidji_northwoods` | `johto` | `03` |
| `mn_winter_grand_rapids` | `mn_grand_rapids_riverfront` | None currently placed |
| `mn_winter_iron_range` | `mn_iron_range` | `03` |
| `mn_winter_border_pines` | `mn_winter_outdoor` | `3d` and `37` |
| `mn_winter_border_pines_south` | `mn_border_pines` | `37` |

Only safe blank/unreferenced glyph slots in private copies receive native donor
art. Both source banks retain their original logical addresses when expanded
from192 to256 glyphs. Collision and block IDs remain exact. Native donor icy
grass retains every bitplane and zero-index priority hole. Ground/flower/roof
foliage uses explicit recipes, never a new terrain generator or copied catalog.
The private palette keeps native inks1..3 and assigns snow to background0.
The two existing Cut trees and their replacement blocks have matching cold art;
removing a tree must not reveal green ground or destroy its remaining quadrants.

## Fast whole-map review

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_inland_winter.py \
  --output planning/minnesota/renders/inland-winter-belt-NEXT
```

Use a fresh output directory each pass. The measured whole-ten-map composition
plus complete R09/R17 route renders takes **0.488 seconds after pack loading**.
The tool validates the selected lineage and proposal bindings, mutates only its
in-memory source view and never serializes a game pack. It preserves all original
PNGs, route overviews, crops and rejected passes. Source reviews are retained at
`planning/minnesota/renders/inland-winter-belt-v1060/`.

The current art audit finds no remaining green terrain fragments across the ten
maps. Four icy grass recipes preserve native masks; both Cut trees retain native
before/after collision and original replacement data. Its before/after art is
retained at `planning/minnesota/renders/inland-winter-art-audit-v1062/`.

## Native builder and current limits

`build_inland_winter_belt` and its dedicated CLI implement the same explicit art
recipes in Rust. The first isolated publication correctly failed its complete
residual check because an ordinary whole-town publisher re-encoded an unrelated
Iron Range NPC payload. The correction changes only the art binding in the map
and attribute tables; it never reserializes NPCs or programs. The complete
preservation assertion remains in place. Both rejected attempts are retained.

The corrected isolated builder produced
`content-packs/minnesota-inland-winter-belt-source-v01.crystalpack`, SHA256
`ce8b5ef030d9ff15b9cfe8436e8107a0ca4e327c76cbef02228a45090caa88b1`.
Its 5,488,640 source/installed art pixels match exactly. The other733 maps,
original runtime files, all602 audio entries and NPC payloads, trainers,
encounters, scripts, rewards and field-move inheritance remain exact. These are
native art/pack checks, not production renderer or earned-gameplay acceptance.
The corrected builder/CLI are registered in the shared source. The main checkout
build and `cargo clippy --locked -p crystal-assets --all-targets` pass. The main
builder's `minnesota-inland-winter-belt-source-v02.crystalpack` is byte-identical
to the reviewed candidate above. Neither candidate is selected.

## Native presentation integration

`inland_winter_render.rs` explicitly resolves the six aliases to their original
presentation sources. Seasonal lighting now recognizes those exact aliases;
private background and foreground colors follow the same native clock. The
existing animation schedules and frame indices remain intact. Bemidji's
pre-existing stair alias also forwards to its original `mn_bemidji` water and
flower behavior. Iron Range and Border Pines gain no forest animations.

The broader stair check exposed a pre-existing omission: Headwaters' four
authored Whirlpool clips were not inherited by its stair clone. The exact
ancestor chain now loads authored animation metadata when a clone has none of
its own, retaining a clone's own metadata when present. The four affected glyphs'
native bitplanes, palette addresses and foreground metadata are identical in
source and clone. All five outdoor stair aliases now pass the existing
production animation frame/schedule comparison at all four times of day.

The grass cache and actual-block resolver use the authored icy colors, including
both `3d` and `37` in northern Border Pines. Production renderer asset checks
pass at morning/day/night/dark: all six terrain bindings, animation registrations,
schedules and frame indices, clock-lit foregrounds, and all four icy rustle styles'
native OAM opacity and color distribution. The actual production rustle buffers
and nearest-neighbor previews are retained in
`planning/minnesota/renders/inland-winter-rustles-v1074/` and the final corrected
renderer pass `inland-winter-rustles-v1081/`. These CPU asset checks
are separate from native GPU scene review and traversal.

Weather resolves these inland aliases to snowfall. An hourly regression confirms
that snowfall replaces showers inland while the preserve/Safari/entry camp and
Superior coast exclusions retain occasional rain. Unknown similarly named
aliases do not acquire winter behavior. Existing warm weather, fade, midnight
continuity and indoor/original-region exclusions still pass.

Native scene originals from the corrected renderer are retained in
`planning/minnesota/renders/inland-winter-native-v1083/`. All nine scenes were
visually inspected: full tower roof, flush south stairs, river pier, four icy
grass styles and matching day/night snow/foreground colors. The unaltered
originals and contact sheet are separate; the sheet is at
`planning/minnesota/renders/inland-winter-render-review-v1084/native-scenes-original.png`.
Warm Boundary Waters, Grand Marais and entry-camp rain views, a same-position
clear/rain pair and exact played Bemidji saves are retained in
`inland-winter-native-followup-v1078/`. They use the prior renderer before the
independent Headwaters ambient-inheritance repair. The clear/rain pair confirms
subtle darkening without modifying source assets or clock palette identity.

The scene review also exposes an older terrain issue, not visual acceptance of
all Iron Range geometry: grouped rock `0a` replaces face `72` at R09C00R00
metatiles `(24,7)` and `(29,7)`, and crowds the neighboring upper edge on row8.
Back those rocks off the face in a separately guarded terrain revision, restore
continuous native edges and recheck its grassy uphill and dry return routes.
This winter art stage deliberately retains all original rows/collision.

That scoped terrain follow-up is now implemented and verified in
[`iron-rock-clearance-v01`](../../revisions/iron-rock-clearance-v01/README.md).
Its unselected child pack is
`content-packs/minnesota-iron-rock-clearance-v01.crystalpack`, SHA256
`5b6435171e2be96d49cc8a5ebbfc45d8223f5701d202dd71aa79e658b52ab982`.
That child was subsequently extended by the verified
[`Tamarack snowline`](../tamarack-snowline-v01/README.md). Its latest unselected
pack is `content-packs/minnesota-tamarack-snowline-v01.crystalpack`, SHA256
`13554605a69f58a1ad3855f47bf64ad577ee2623c86742bc75ffb789399ba248`.
That was followed by the
[`Iron Range descent thaw`](../iron-descent-snowline-v01/README.md), whose latest
unselected child is `content-packs/minnesota-iron-descent-snowline-v01.crystalpack`,
SHA256 `83aae389479014f62a12bfd2f97d494972a27ca3c2b7fd796b9c4bea9ac3874f`.
The descent's later
[`cliff/boulder repair`](../../revisions/descent-rock-clearance-v01/README.md)
was followed by the verified
[native grass-Cut repair](../../revisions/inland-grass-cut-v01/README.md) and the
[complete Headwaters thaw](../headwaters-snowline-v01/README.md). The latest
unselected continuation candidate now includes its separately verified
[stair-boulder clearance](../../revisions/headwaters-rock-clearance-v01/README.md):
`content-packs/minnesota-headwaters-rock-clearance-v01.crystalpack`, SHA256
`d2b9d98f8204d216b9cdf7eb386441efd374a47424ee615c891071764f630bf9`.
That child is followed by the verified [Itasca snowline](../itasca-snowline-v01/README.md),
`content-packs/minnesota-itasca-snowline-v01.crystalpack`, SHA256
`d7139c4a1e4ada8aa9b389675b07aaf119bb670c2003fa21974ddf9ac6856c88`.
The subsequent [Ely pine-town integration](../../towns/ely-pine-camp-v01/WINTER-INTEGRATION.md)
preserves that child and every later story/opening resource. Its latest unselected pack is
`content-packs/minnesota-ely-pine-camp-winter-v03.crystalpack`, SHA256
`2d4f4e01216ee770f07bebd665a9d96a0badc2f346f802aceaa04f0143c409d6`.
The subsequent [Inner Basin bedrock revision](../../caves/inner-basin-bedrock-v01/README.md)
keeps every exterior and the complete winter/Ely parent. Its latest unselected
child is `content-packs/minnesota-inner-basin-bedrock-v02.crystalpack`, SHA256
`83a63e4791f5de5bd72b8b874426b693bf03e3b2067665bfd1a4f7182aba58ba`.
That child is now extended by the verified
[upper Haulage rock clearance](../../caves/haulage-rock-clearance-v01/README.md).
The [combined thirteen-stage recipe](../../revisions/reviewed-maps-v01/README.md)
reproduces the current unselected continuation as
`content-packs/minnesota-reviewed-maps-v02.crystalpack`, SHA256
`110e658e66449fd6ca3dc52fb7de5fa3f0f2bd827d8103aa838a32d63193229c`.
Use that child for continuation; rebuilding from an earlier
candidate would discard reviewed rock/edge, field-move and snowline repairs.

The final renderer build, its focused weather/seasonal/winter tests, installed
six-alias pixel checks and `cargo clippy --locked -p crystal-bevy --all-targets
--features location-tester` pass. The four external legacy winter, seasonal and
outdoor-stair regressions also pass after the Headwaters repair. Changed leaf
modules pass rustfmt. `cargo fmt --all -- --check` remains blocked by pre-existing
formatting differences across the workspace; unrelated files were not reformatted.

The candidate's Bemidji roof/cliff correction also passes all seven production
stdio-MCP cases (109 recorded calls), including both cliff sides, both stair
directions, roof blocking, heritage-house entry/return, Start and Continue.
`planning/minnesota/renders/inland-winter-bemidji-gameplay-v1076/` retains the
original initial and played saves, observations and empty stderr. Initial
positions, healthy Minnesota partners and zero badges are explicit setup; this
is not earned statewide progression.

Before promoting a candidate, finish all of the following:

- The fitted thaw toward Itasca now passes complete native art, weather, four-chunk
  walking, field moves, battle, real gatehouse and Continue checks in its runbook. The complete Leech approach now has reviewed
  snow/melt/green banks, unchanged lake/stair collision and actual connected walks. The Duluth descent
  now has reviewed snowy benches, a broad melt bank, green Superior shore and
  passing native stair/Grand Rapids/Duluth/Continue walks. Its pre-existing
  eastern cliff boulders are now repaired in the separately verified successor. The four
  Rainy River/Tamarack road chunks now have reviewed winter/melt/green art and
  passing real walking seams to snowy Rainy River and green Boundary Waters.
  Review each remaining connected snowline in ordinary player cameras.
- Rebase onto the actual latest composed parent, inspect changed module/art
  bindings and preserve its newer public-road story, services and fauna.
- Verify native GPU images and production controller movement, seams, encounters,
  Cut denial/success/replacements, resources, rooms and Continue. Setup fixtures
  must remain explicit. No current source image proves those gameplay results.

The candidate is **unselected**. No running app, browser release or world
selector was changed for this art stage. Generated packs, renders, logs and
verification output remain ignored and are not source deliverables.
