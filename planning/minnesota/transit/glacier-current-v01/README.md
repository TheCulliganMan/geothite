# Current Glacier Line passenger service

The five existing stations become useful return connections: Detroit Lakes,
Staples, St. Cloud, St. Paul and Winona. All four other stops appear in native
line order at each conductor. An `X` marks a station the player has not entered
on foot. The original St. Paul courier victory activates free passenger travel;
Grand Rapids' later depot chapter does not substitute for it. No Red Wing stop,
new fee, rail-pass errand, gym flag or route bypass is introduced.

This is a local integration over the selected Driftless fossil gallery pack.
`parent.json` records that complete parent. `service.json` binds its content
identity and all five actual halls, conductor bodies, protected arrivals,
public doors, native block hashes and boarding/reset movements. The historical
R12/R14/R15 candidates are review evidence, not publication parents.

The final review pack is `content-packs/minnesota-glacier-current-v03.crystalpack`,
SHA256 `fffff3a575234c28de03c6efe808fb46f6cd8d7910c76d97ce3e03bf0e5c4c5b`.
Its content identity is
`cb08d7ee3a0482d19e4db95b42fce8dadfc8654ccc0252d9b6644f3428b4ea80`.
Selection is recorded in `planning/minnesota/world.json` only after final
acceptance. It is not a browser publication or proof of an earned campaign.

## Play and connections

Enter stations through their real public doors to remember them. After Q03's
courier victory, speak to a conductor and choose a previously visited stop.
The conductor opens the protected throat, the player boards, the conductor
returns home, and the native Magnet Train animation plays. The destination
loads into its public concourse, facing south. The conductor and platform
remain blocked to ordinary public walking. B or CANCEL releases the ticket
surface without moving, spending money or changing progress.

| Stop | Public city/outpost door | Hall foot arrival | Ride arrival | Conductor approach |
| --- | --- | --- | --- | --- |
| Detroit Lakes | `MnDetroitLakes(11,9)` | `(11,17)` | `(13,11)` | `(13,10)` |
| Staples | `MnStaplesRailJunction(13,7)` | `(9,15)` | `(11,11)` | `(11,10)` |
| St. Cloud | `MnStCloud(35,11)` | `(11,17)` | `(11,11)` | `(11,10)` |
| St. Paul | `MnStPaul(66,47)` | `(15,21)` | `(17,11)` | `(17,10)` |
| Winona | `MnWinona(19,13)` | `(11,15)` | `(11,11)` | `(11,10)` |

Coordinates are16px gameplay tiles. Existing hall exits return to the same
safe outdoor porches. The five `EVENT_MN_STATION_*_VISITED` flags remain distinct
from `EVENT_MN_COURIER_DEFEATED`. Only the three previously absent visit flags
are added; Staples and Winona retain their existing callbacks. A train always
checks the courier event and destination visit again before boarding.

`city-access.json` records an existing connected road port for each public hall.
Winona's original starter checkpoint remains meaningful: the qualified inward
coordinate event moves through its wardens, and the outward return retains its
native direction check. This integration does not remove or bypass that gate.

## Review and verification

- [All five complete halls](../../renders/glacier-current-v794/whole-halls-v815/all-five-halls-original.png)
- [Actual native arrivals at all five stops](../../renders/glacier-current-v794/native-arrivals-v815/all-five-actual-arrivals.png)
- [Clean native ticket prompts](../../renders/glacier-current-v794/controller-v815/ticket-and-train-native-lcd-v816.png)
- [Seven controller/presentation cases](../../renders/glacier-current-v794/controller-v815/receipt.json)
- [Actual courier, refusal, two rides and Continue through stdio](../../renders/glacier-current-v794/mcp-v815/receipt.json)
- [Complete local acceptance](../../renders/glacier-current-v794/acceptance-v818.json)
- [Independent complete-parent audit](../../renders/glacier-current-v794/audit-v815.json)

The final seven default-profile cases pass; six cover gameplay and one covers
train-art presentation. The original six cover: five actual foot entries/Continue; all service
and destination refusals/cancels; every one of80 visit masks; all20 directed
animated rides, two repeat returns, exact resources/flags and Continue; an
actual courier victory followed by a guarded ride; and both directions of the
separate native train-art presentation fixture. The latter is explicitly CPU
composition, not evidence of a live journey. The five640×576 arrival views
load actual reached saves with no position, facing, clock or party overrides.
No exact-pixel comparison is claimed for these arrival views.

The seventh, public-approach case checks walking from each actual city road
port to its hall, remembered foot entry/Continue, the original same-door return
and the walk back to the port. All setup positions/parties/prerequisites are
explicit QA fixtures. The original starter receipt is supplied only for that
Winona checkpoint case. These checks do not establish earned regional access.

The stdio replay uses115 public RPC calls, refuses an unknown marked stop,
cancels, rides St. Cloud→St. Paul→St. Cloud, checks free fares and opens the
production Start menu after a fresh process Continue. Its stderr is empty and
its pack, client and source save remain unchanged.

The independent audit restores the five changed station programs, their unused
fishing headers and three new visit flags and recovers the entire parent. All729 other maps,
original station terrain/actors/doors/stories, recovery destinations, catalogs,
native art/runtime files and audio remain exact. All377 audio programs pass
PCM hashes, frame counts and loop ranges. Final source/table replay also
checks serialization, all20 guarded bindings and unsafe-contract rejection.

The first local render left a sentence fragment beneath the menu; the final
welcome uses four complete two-line pages ending with `Choose a stop.` /
`Or CANCEL to stay.`. Initial harness failures are retained separately: one
observed the wrong animation event after automatic settlement, another compared
absent false movement flags with native explicit false entries, and the first
public-approach walk assumed one input could not invoke a coordinate movement.
No gameplay dispatcher or map was changed to satisfy those assumptions.

The regional policy audit found inherited Johto fishing references in the dry
station headers. Final v03 replaces only those unused references with
`FISHGROUP_NONE`; all original fishing tables remain exact for their original
regions. The publisher first proves every hall dry and rejects a wet-hall
variant. All five touched rooms,321 regional trainers and225 active Minnesota
species pass the roster/evolution policy audit. The positive round-trip and
13 unsafe-input rejections run against the exact final pack.

## Executed Rust build and fast render loop

Use the shared Cargo slot before heavy builds in this checkout. Commands below
were run with the pinned toolchain; choose a fresh ignored pack output on replay.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin pack_glacier_line_service
.local/glacier-current-publisher-v813 \
  content-packs/minnesota-driftless-fossil-gallery-v03.crystalpack \
  planning/minnesota/transit/glacier-current-v01/service.json \
  content-packs/minnesota-glacier-current-v03.crystalpack
```

The fresh publisher is frozen at `.local/glacier-current-publisher-v813`.
The default-profile asset regression reproduces its output from Rust source,
round-trips the complete serialized pack and rejects unsafe inputs.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/transit/glacier-current-v01/render_halls.py \
  --pack content-packs/minnesota-glacier-current-v03.crystalpack \
  --output FRESH_REVIEW_DIRECTORY
```

The warm whole-five-hall render took0.24 seconds after pack loading. Every
source/native original, derivative, crop, rejected prompt, GPU file, save and
execution receipt is retained. Terrain source images, actual native arrivals,
CPU ticket composition and the explicit train-art fixture are separate evidence.
Final all-target Clippy passes for both affected crates, and changed Rust
formatting passes. The earlier shared-checkout exporter errors were corrected
before the successful final replay. Workspace formatting still retains existing
unrelated differences. Wider map and campaign review remains active.

The final controller command used the absolute final pack in
`CRYSTAL_INTERIOR_PACK`, this `service.json` in `CRYSTAL_GLACIER_SOURCE`,
this `city-access.json` in `CRYSTAL_GLACIER_CITY_ACCESS`, and fresh save/render
directories in `CRYSTAL_GLACIER_SAVE_DIR` / `CRYSTAL_GLACIER_RENDER_DIR`:

```sh
CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-glacier-current-v03.crystalpack" \
CRYSTAL_GLACIER_SOURCE="$PWD/planning/minnesota/transit/glacier-current-v01/service.json" \
CRYSTAL_GLACIER_CITY_ACCESS="$PWD/planning/minnesota/transit/glacier-current-v01/city-access.json" \
RUST_MIN_STACK=67108864 \
  .local/glacier-current-controller-v813 interior_life::glacier_line:: \
    --ignored --nocapture --test-threads=1
```

All seven cases and every retained save/capture are bound to the final v03
pack. Historical failures and earlier v01/v02 evidence remain retained under
their original bindings; they are never relabeled as final passes.
