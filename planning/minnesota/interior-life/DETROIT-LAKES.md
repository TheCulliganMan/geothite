# Detroit Lakes: five published interiors

The Rust publication is
`content-packs/minnesota-detroit-lakes-v01.crystalpack`, built over the corrected
`minnesota-harbor-care-v02.crystalpack`. Strict write/read verification passes:
705 maps, 475 species, 270 moves and 377 original audio programs. This adds the
actual town and its five reserved non-gym interiors. The all-interiors goal
remains active; this is not completion of the six-stop Glacier Line.

## Places and people

| Room | Purpose and native behavior |
| --- | --- |
| Pokémon Center | Original native treatment desk, two local visitors, free HP/PP/status care, PC and a registered blackout return at `(8,13)`. |
| Mart | Counter clerk, a prairie-road shopper and native priced Great Ball, Super Potion, Antidote, Parlyz Heal, Super Repel and Awakening stock. |
| Glacier Line station | Timber waiting hall, conductor sealing the platform, a returning lodge worker and a porter minding an old canoe case. Both train doors remain inaccessible through ordinary walking. Rides and visit callbacks remain unfinished. |
| Canoe cabin | A patched family paddle records trips with a parent; the apprentice's first repair earns Grandpa's “deluxe Minnesota paddle” verdict. The host points to the real public launch and asks visitors to leave the reeds alone. |
| Ranger cabin | The actual SQ01 observation hand-in. The ranger revisits an old notebook's dismissive description of marshland and connects prairie, pine and wetland into one lake system. |

Town actors include the station steward, clinic gardener, beach warden, dock
angler and cabin neighbor. The five rooms contain eleven residents. Every
resident has a concrete native approach; nurse and clerk use actual counters.
The complete clinic, shop, timber cabin and station floorplans use original
native art. No source catalog or exported pixels are added to tracked files.

Sources are [the town dossier](../towns/MnDetroitLakes.json),
[explicit layout](../towns/MnDetroitLakes.layout.json) and
[station](../transit/MnDetroitLakesStation.json). Their preceding source-only
versions remain in `revisions/detroit-source-v1/`. The native art alias imports
the three already installed Headwaters dock blocks and six original
JohtoModern station blocks. A private 256-glyph bank accommodates these imports
without overwriting any source glyph, palette or tileset. Original field-move
rules are retained, with only the private alias receiving inherited behavior.

## Roads, entrances and lake

The existing `MnDetroitLakesApproach` is retained, including both trainers,
habitat, sign and its R16 connection. The real town inserts east of that wooded
approach, with three reciprocal roads:

| Town edge | Neighbor | Offset / exact core lanes |
| --- | --- | --- |
| East | `MnR15C00R00` | Town -3, route +3: `(47,18/19)` ↔ `(0,24/25)` |
| West | `MnDetroitLakesApproach` | Zero both ways: `(0,18/19)` ↔ `(47,18/19)` |
| North | `MnR19C00R01` | Zero both ways: `(24/25,0)` ↔ `(24/25,11)` |

Only two old R19 boundary metatiles change: `(11,5)` and `(13,5)` become native
forest from their exact lawn preimage `02`, closing surplus shoulders beside
the connected center pair. Every original actor, note, item, trainer, script,
warp and original recovery record remains exact. R15 and the wooded approach
change only their east/west connection attributes. Native connection projection
is checked for all twelve directed walking-lane bindings.

Each facade has an active DOOR followed by its own passive exterior FLOOR
return. Each room has two real exit carpets followed by its passive arrival3.
The shared reservation and world records retain exact door positions and the
wooded approach's role. The lake has a real launch at `(13,23)` toward `(13,24)`,
a closed-ended public pier and native rails. No outer lake quadrant is Water;
the unconnected perimeter is visibly closed by native rock/forest collision.

## SQ01 hand-in

The ranger requires `ENGINE_MINERALBADGE` and the three existing
`EVENT_MN_LAKE_OBSERVATION_01/02/03` flags. It grants exactly one `MAX_REPEL` and
sets `EVENT_MN_LAKE_OBSERVATIONS_REWARDED` only after `giveitem` succeeds. A full
Pack retains the unclaimed gift and gives a visible retry instruction. Repeat
dialogue discusses the notes without another item. Reading the R19 signs still
sets only their own observation events; no route sign grants the reward or
silently delivers the fieldwork. No chapter, badge or train-access event is
granted by this publication.

## Executed build and review

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --bin pack_detroit_lakes --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_detroit_lakes \
  content-packs/minnesota-harbor-care-v02.crystalpack \
  planning/minnesota/towns/MnDetroitLakes.json \
  planning/minnesota/towns/MnDetroitLakes.layout.json \
  planning/minnesota/transit/MnDetroitLakesStation.json \
  content-packs/minnesota-detroit-lakes-v01.crystalpack
```

The preservation/deterministic-rebuild case passes again after the verified-source
metadata update (37.58 seconds; the initial isolated run was 8.50 seconds), including
rejection of missing rooms, false facades, misdeclared counters, buried nurses,
weakened quest requirements, an open train throat and duplicate publication.
It checks all original maps/programs, split payloads, catalog data, runtime
files, audio, metadata, HOME and recovery, allowing only the explicit old-road
changes and additive town data. All 377 PCM hashes, frame counts and loop ranges
pass on the written pack.

```sh
export RUST_MIN_STACK=67108864
export CRYSTAL_DETROIT_BASE="$PWD/content-packs/minnesota-harbor-care-v02.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-detroit-lakes-v01.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test detroit_lakes --config 'profile.wasm-dev.opt-level=1' \
  -- --ignored --test-threads=1
cargo test --locked --profile wasm-dev -p crystal-bevy --lib detroit_lakes_controller \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
```

All eighteen headless production frames match full native source frames exactly
and have been visually inspected: twelve room/interaction cameras and six town
views. Views are [rooms](detroit-room-views.json) and
[town](detroit-town-views.json). Review includes the
[whole town](../renders/detroit-lakes/native-v01/MnDetroitLakes-whole-native-source.png),
[first room page](../renders/interior-life/detroit-v01-native/native-review-page-1.png),
[second room page](../renders/interior-life/detroit-v01-native/native-review-page-2.png)
and [town cameras](../renders/detroit-lakes/native-v01/native-review-page-1.png).
Native scale, complete counter/table joins, readable service approaches, closed
platform throat and useful lake frontage were reviewed. The source render/check
loop took 438.2 ms after loading, including eight deliberately rejected cases.
Every source version, raw GPU frame and conversion remains archived and ignored.

All seven new controller cases pass on the written pack (27.58 seconds). They cover all five
rooms/residents and both carpets, all three roads, care/PC/blackout recovery,
native shop money/items, each missing SQ01 requirement, actual walking to all
three field signs, full-pocket Continue, native Potion Toss, reward retry and
repeat denial. Explicit starting position/resources/badge fixtures are setup;
the subsequent actions use `VisibleShellController::press`. The poison case
checks the registered clinic after Continue, full healing, native money loss
and preservation of the previous battle result. Additional step-event cases
check input ownership, no repeated idle damage/notices and real hatch/nickname
completion before walking and Start resume.

The clinic check exposed a shared controller gap: core poison damage happened,
but its faint/whiteout presentation was omitted from renderer-neutral input.
The controller now presents native step events and advances the original poison
and hatch clocks. Poison recovery has a distinct authoritative mutation, with
atomic rejection for empty/healthy parties, active battles or pending static-wild
results; it preserves the previous battle result. The original battle-loss
terminal checks remain intact. The hatch check additionally exercises B through
the shared native nickname owner. These are controller fixes, not replacement
dispatchers in the TUI or MCP.

The two authoritative asset blackout cases pass, retaining saved-spawn recovery,
atomic rejection and the draw-result guard. An additional runtime Rock terminal
case stops in its pre-existing fixture setup at
`runtime_tests/runtime_basics.rs:2440` (`RockMonEncounter export`), before recovery;
it is not counted as passing. Changed leaf modules pass rustfmt. Whole-workspace
formatting remains blocked by existing unrelated diffs beginning in
`offset-allocator`. Unmodified Clippy also reports existing `bad_bit_mask` and
`never_loop` failures. The four affected crates pass the all-targets scoped retry
with only those two existing lints allowed; no warnings point to the new Detroit
or step-event modules or the new recovery API. Existing unrelated warnings remain.

All three repaired St. Paul controller cases also pass on this exact child
(21.51 seconds): 39 ordinary facades/returns, both clinics and PCs, bridge walks,
Surf underpasses, reciprocal river seams and Continue. Its entire St. Paul map
equals the previously reviewed 699-map parent. Its runtime content identity is
`e325f300d4f1f5bdd2736840f6346889155b6b6562a5f62f2693f49dd9f953c4`;
the unchanged file SHA-256 is
`70bad00adc9bb88b0d4b4378b1c76fce0c9af42d37f8c4966341c57bd5f1e549`.

The seven existing interior-life/layout/care cases pass again (395.35 seconds),
including all 117 added residents and their room returns. The original R19
two-ramp/observation/Continue case passes (23.52 seconds). On the local external
`text-tui.crystalpack`, the required Mom/New Bark controller case passes
(9.54 seconds), as does the real battle-command/PP-mutation case (6.88 seconds).
The original poison faint/white-hold presentation case also passes (7.68 seconds).
These results use the rebuilt controller; the explicit source fixtures do not
claim earned regional progression. The rebuilt `geothite` MCP movement/production
Start-menu case passes (3.35 seconds), and the spawned-client clean-stderr case
passes (4.30 seconds). The latter requires stderr to remain byte-for-byte empty
through Mom's dialogue. The final all-targets scoped Clippy pass completes in
1 minute 30 seconds:

```sh
cargo clippy --locked --profile wasm-dev \
  -p crystal-assets -p crystal-runtime -p crystal-bevy -p geothite \
  --all-targets --features crystal-assets/test-fixtures \
  --config 'profile.wasm-dev.opt-level=1' \
  -- -A clippy::bad_bit_mask -A clippy::never_loop
```

Focused leaf rustfmt checks and the scoped documentation/source diff check pass.
The wider all-interiors objective and the independent fixture/format failures
above remain open.

The client was built with `cargo test --locked --profile wasm-dev -p geothite
--no-run --config 'profile.wasm-dev.opt-level=1'`. The resulting
`geothite-123d07bcbc5be822` and `stdio_clean-f7765a58e99e1071` test executables
ran the named cases with `CRYSTAL_RENDER_TEST_PACK` set to the completed local
`content-packs/text-tui.crystalpack`. Both live under the ignored
`target/interior-life/wasm-dev/deps/` cache; they are verification artifacts,
not deliverables.

## Remaining scope

The subsequent [Moorhead child](MOORHEAD.md) publishes its two rooms and town,
with four production cases and thirteen native frames passing. International
Falls' two rooms and the old Grand Marais shore-gate contract remain open. A broader source audit also
found three named Glacier Line stations outside the 218-entry reservation
catalog: Staples, Red Wing and Winona. They are real planned work, not installed
rooms. Their separate authored station/outpost geometry and all six-stop
transport requirements remain open; the public Detroit hall does not replace
that work. The [coverage record](COVERAGE.md) retains these and the final regional
quality audit. The Moorhead follow-up resolves the bedroom camera transform;
a separate city follow-up resolves the missing second-hop Mississippi scenery
in the off-map comparison, with two complete native frames passing.
