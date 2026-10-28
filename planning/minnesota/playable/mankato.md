# Mankato: terrace and native Gym2

The gym terrain described here is historical. The current
[sand-terrace revision](../gyms/mankato-terraces.md) keeps this city's indexed
door contract, original battles and services while replacing the ambiguous
wooden banks and stair blockers. Its ordinary-party literal course and both
native hop views pass. This record retains the earlier town/service lineage.

[Complete town](../renders/mankato-city/Mankato-whole.png) ·
[Native facade](../renders/mankato-city/native-gym-facade.png) ·
[Native terrace](../renders/mankato-city/native-terrace.png) ·
[Native interior](../renders/mankato-city/native-gym-interior.png)

The 24 × 18 town places Torven's gym on the west bank above a low native ledge.
Hop down toward the prairie garden and mart; the east path returns uphill.
Three complete source facades have real doors. No decorative door is unbound,
no incidental Cut tree is placed, and the unconnected outer perimeter stays
closed. R02 east and R03 north retain the world reservations and reachable
landings; this local candidate does **not** publish those regional seams.

The initial preview left an oversized court and broad shoulders. The revision
narrows its paving and uses pine fingers to shelter the court, western lane
and eastern arrival. Whole-town and source/native player views were inspected.
All render passes and conversions remain archived under the ignored render
folder. Native source pixels, collision and noon palettes are retained.

| Building | Exterior door / safe return | Exterior active / passive | Interior arrival | Carpets |
| --- | --- | --- | --- | --- |
| Clinic | `(31,13)` / `(31,14)` | 1 / 2 | `(10,13)`, warp3 | `(10,15)`, `(11,15)` |
| Mart | `(37,27)` / `(37,28)` | 3 / 4 | `(6,5)`, warp3 | `(6,7)`, `(7,7)` |
| Gym | `(10,11)` / `(10,12)` | 5 / 6 | `(6,17)`, warp3 | `(6,19)`, `(7,19)` |

Approach the city ledge from `(10,17)`, step onto native `HOP_DOWN` at
`(10,18)`, then jump to `(10,20)`. The source lower quadrant `(10,19)` blocks
reverse movement. The ordinary return to `(10,17)` is 35 steps. The builder
executes the core native hop/reverse and checks the return graph; production
input separately proves the actual jump and uphill walk.

The complete canonical Gym2 is referenced by its object SHA-256 and glyph map
in `towns/MnMankato.json`. The builder reconstructs a private expected module,
art and trainer roster for exact comparison, rejects stale installed content,
and changes only the three gym return destinations and landmark. Caldren,
Jorel and Torven, all original names, retain their real trainer battles,
physical rock/stair/drop route, first-badge prerequisite, badge2 and one Mud-Slap
TM. There are no city-owned alternative gym rows, objects or puzzle scripts.

The local pack adds the town, clinic and mart to the current 480-map gym pack,
for **483 maps**. Group75 and one new landmark are additive. Only the town and
clinic gain recovery entries, in the addon recovery range. Existing HOME,
recovery entries, all other maps, private gym art, trainers, catalogs, runtime
files and audio are preserved. No player event or badge is granted by installing
this pack.

## Verified commands

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_mankato -- \
  content-packs/minnesota-gyms-remake.crystalpack \
  planning/minnesota/towns/MnMankato.json \
  planning/minnesota/towns/MnMankato.layout.json \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-mankato-gym-remake.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_mankato.py \
  --pack content-packs/minnesota-mankato-gym-remake.crystalpack --verify-published

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test mankato -- \
  --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-mankato-gym-remake.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-mankato-gym-remake.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_mankato_city_controller -- --ignored --nocapture --test-threads=1
```

Both asset tests and both controller tests pass. The asset checks compare the
written candidate with a fresh build, preserve every unrelated map and runtime
file, and reject a stale gym reference, misplaced facade, missing ledge,
changed leader and duplicate installation. The controller earns both practice
wins, the leader win, actual selected-move PP use, badge and one TM; verifies
both gym carpets, reentry and Continue; and separately checks free HP/status/PP
care, clinic recovery, native PC access, an acknowledged priced Super Potion
purchase, both clinic/mart carpets and Continue.

The published terrain/collision comparison passes for the town and canonical
gym. `verify_crystalpack ... --audio --desktop` passes all 377 audio programs'
PCM hashes, frame counts and loop ranges. Changed Rust leaves pass rustfmt;
workspace formatting still reports pre-existing changes beginning in
`offset-allocator/src/lib.rs:408`. Full all-target clippy is blocked by the
existing always-false bit-mask comparison in `crystal-core` multiplayer code.
Scoped all-target clippy passes for assets and Bevy with existing `never_loop`
findings allowed; neither run reports a warning in the changed Mankato modules
or gym controller tests.

**Fixture scope:** initial city placement, legal level100 Mew, prior first badge,
service injury/empty PP and spending money are explicit setup. This proves local
physical play and services, not ordinary-party balance or an earned journey
from Rochester. The all-region pack still needs the current gym remakes and
city/route bindings integrated together. St. Paul and St. Cloud remain separate
native-city work. No user visual acceptance is inferred from passing checks.
