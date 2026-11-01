# Driftless Trail: compact ledge review

This individually authored replacement is **32×20 metatiles**, one exterior and two small interiors. It replaces the rejected 96×72 nine-chunk blockout. The limestone bench, short meadow road, farmhouse and creek dock each serve a specific route decision. No new art or copied map layout is required.

[Whole map](../renders/driftless/whole-route-b8d05cd0.png) · [player-scale views](../renders/driftless/viewpoints.png) · [both interiors](../renders/driftless/interiors.png) · [native ledge](../renders/driftless/native-ledge.png) · [native ramp](../renders/driftless/native-ramp.png) · [native dock](../renders/driftless/native-dock.png) · [native cave](../renders/driftless/native-cave.png) · [native farmhouse](../renders/driftless/native-farm.png).

**Current status:** the bounded 2026-10-26 source repair below is ready for native
integration. Historical native captures and controller results farther down this
runbook describe the preceding publication; they have not been rerun against
these changed boundary/canopy/corner blocks. New native acceptance is pending.

## Creek and canopy repair

Whole-map and actual native ledge/dock captures were inspected first. The existing
composition and directional route hold, so the repair preserves every actor,
party, room, port, native ramp and ledge. Three concrete collision defects changed:

- The southern creek had eighteen open WATER cells at core y39, x2..19. Native
  blue-water rock `38` at metatile `(1,19)` and `39` at `(2..9,19)` close them
  visibly. All 204 outer core cells now contain zero WATER; the conservative
  walking/Surf component and separately seeded real creek water cannot escape
  across an outer water edge. The creek retains 85 WATER cells.
- Decorative tree toe `63` at metatile `(14,10)` contained incidental `CUT_TREE`
  core `(28,21)` without sealing any gameplay. Ordinary native `3d` replaces it.
  There are now zero Cut quadrants and an explicit empty `cut_gates` contract.
- North-west/south-west dock rail corners were missing, allowing direct Floor ↔
  Water adjacency around the closed end. Source-authentic `40` at `(5,16)` and
  `48` at `(5,18)` join the horizontal rails to the existing `44` west end.
  Every walking dock tile now has no adjacent WATER. Actual source art for these
  corner recipes was inspected before placement.

[Current dock camera](../renders/driftless/viewport-creek_dock-b8d05cd0.png) and
[actual emptiest camera](../renders/driftless/viewport-emptiest_ground-b8d05cd0.png)
were inspected with the complete revised render. The upper bench remains the
widest area and contains the climb, optional Pat battle, sign, Potion outcrop and
one-way descent. No broad layout redesign was needed for this audit.

The original ledge still saves forty walking steps: a three-tile downward hop
versus a 43-step ordinary return through the eastern ramp. The shared four-step
entrance grass and asymmetric bluff/meadow decision retain their checked
behavior. Current source checks reach 730 walking/grass cells, including 68 grass
cells; all one exterior/two rooms and original identities remain unchanged.
Ten focused negative fixtures reject an open creek, a brown land boulder in its
water-rock boundary, decorative Cut, missing ramp, flattened ledge, either
missing rail, blocked dock entry, and either removed corner cap:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_driftless.py --self-test
```

The current fast page defaults to port 8793 to avoid other live authors. Warm
render/check takes about 125 ms. Source `--self-test` and Python compile checking pass. The latest shared
`check_plan.py` run hit the world/chunk-count assertion while other authors were
changing route extents; root must resynchronize and rerun this global check. Root must republish the changed route and rerun actual
controller corner/Surf boundary/Cut inventory fixtures, as well as the existing
ramp/ledge/room regressions, before claiming this repair natively verified.

## Play decisions and connections

Coordinates below use 16px gameplay tiles. Authoring dimensions and terrain strokes use 32px metatiles.

| Place | Physical behavior |
| --- | --- |
| Winona arrival | East gate `(60,24)`, landing `(58,24)`. The shared grass band x=54–57, y=22–27 requires at least four grass steps; there is no dry bypass. |
| Meadow or bluff | Westbound, take the short central grass road or climb the eastern ramp, explore the longer bluff and drop onto the west trail. The bluff contains a Potion and optional Pat battle. Eastbound, the ledge blocks entering the bluff from below; the central grass is required before reaching the ramp. |
| Uphill ramp | Native Johto `74` at metatile `(20,9)` has a narrow real opening at core x=41, y=18–19. It connects the lower trail to the upper bench in both directions. No path is painted through a wall. |
| Downhill ledge | Native `57` at metatiles `(13,8)` and `(14,8)`: HOP_DOWN at core x=26–29, y=16, wall at y=17, safe floor at y=18. From `(29,15)`, Down enters the hop and lands `(29,18)`; Up from below is blocked. The continuous cliff closes both sides; climbing back up uses the eastern ramp. |
| Limestone Hollow | CAVE `(18,17)` enters `MnR01Cave1F`, an 8×7-metatile room with two upper hollows and a joining lower hall. Guide and Antidote are inside. Its exit returns to a separate exterior FLOOR warp `(18,18)`. |
| Nelson Farm | DOOR `(49,15)` enters a 6×5-metatile farmhouse. Read Nelson and Tamarack's note, and collect a Berry. Both carpet exits return to separate exterior FLOOR warp `(49,16)`. Walking across the return floor does not trigger entry. |
| Creek dock | Entry core x=20–21, y=34–35 through the bank rail. Native deck, north/south rails and closed west end prevent walking into water. Poké Ball `(13,35)` rewards the short spur. Water remains below the south rail. |
| Rochester arrival | West gate `(2,24)`, landing `(6,24)`. No Cut, Surf or badge is needed on this starter route. |

There are three optional trainers (Brook: Sentret 2; Lena: Caterpie 3/Pidgey 3; Pat: Rattata 4), two outdoor residents, five signs, two outdoor rewards and two interior rewards. Reserved trainer flags 04–06 remain unused. The route condition remains `EVENT_MN_STARTER_RECEIVED`; Nelson does not grant Q01's parcel or advance the first gym. Actual Winona/Rochester ports and the earned Q01 journey remain integration work.

Forty-four unchanged Johto metatile varieties supply the terrain. Source `57` provides actual directional collision, `74` the actual ramp, `43` open water under rails, `41`/`49` north/south rails and `44` the dock end/bank fence. Actors and sign anchors use their real passable approach quadrants. Source artifacts and generated packs remain ignored.

## Fast render loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_driftless.py --serve
```

Open `http://127.0.0.1:8793/`. Edit [terrain strokes](MN_R01.layout.json) and [events/dialogue](MN_R01.json). Whole map, eight viewport crops, both rooms and anchors refresh together without Cargo. A warm render/check measured about 140ms. Publication is separate, and changing pack identity requires regenerating fixture saves through the controller.

Static checks verify both gate connections, mandatory entrance grass, every actor approach, dock closure and both rooms. A directed graph proves the westbound bluff alternative avoids central grass, its reverse cannot, closing the eastern ramp prevents uphill access, and opening it restores access. These checks establish the intended one-way topology; native controller input must additionally prove the engine's hop behavior.

## Historical native build and verification (before boundary repair)

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_driftless -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R01.layout.json \
  planning/minnesota/routes/MN_R01.json \
  content-packs/minnesota-driftless.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_driftless.py \
  --pack content-packs/minnesota-driftless.crystalpack --verify-published
CRYSTAL_DRIFTLESS_TEST_PACK="$PWD/content-packs/minnesota-driftless.crystalpack" \
CRYSTAL_DRIFTLESS_FIXTURE_SAVE="$PWD/.local/driftless-review.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester driftless_controller_ -- --ignored
CRYSTAL_DRIFTLESS_TEST_PACK="$PWD/content-packs/minnesota-driftless.crystalpack" \
CRYSTAL_DRIFTLESS_FIXTURE_SAVE="$PWD/.local/driftless-review.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test driftless -- --ignored
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test driftless --test central_lakes --test headwaters \
  --test league_approach --test gunflint --test north_shore
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-driftless.crystalpack --audio --desktop
```

The native isolated pack contains 391 maps. Publication comparison matches all 640 exterior blocks, seven outdoor objects, five signs, exact text/parties and both interiors. The asset regression composes it with Central Lakes in memory, checks preservation of all existing maps/files/audio/HOME and rejects duplicate installation, changed starter gating, a removed shared grass band, a floor replacement for the native ledge and a landing inside its wall face. All six route asset regressions and all 377 PCM hash/frame/loop checks pass.

Native headless ledge, ramp, dock, cave and farmhouse captures have been inspected. The controller geometry fixture uses explicit placement and an empty party: it does not establish wild-encounter balance, trainer victories, recovery or earned regional progress. Connected Winona/Q01/Rochester gameplay and visual approval remain open.

The production-controller fixture passes the ramp in both directions, the native
ledge hop and rejected reverse, every sign/resident, all four rewards, dock
barriers, both cave hollows, both farmhouse carpets and safe exterior returns.
Dialogue owns input while visible. The exported fixture continues through stdio
MCP from `(52,24)` to `(52,25)` and opens/closes production Start with byte-for-byte
empty stderr. The production terminal client loaded the same fixture, opened
Start and Pack, returned to movement and exited cleanly. This remains an
empty-party geometry fixture, not trainer or encounter progression evidence.

Changed Rust files pass scoped rustfmt. The workspace format check still fails
on existing formatting beginning at `crates/offset-allocator/src/lib.rs:408`.
Focused all-target Clippy passes with `--no-deps` and the existing `never_loop`
allowance. It reports existing warnings and none in the changed route modules.
