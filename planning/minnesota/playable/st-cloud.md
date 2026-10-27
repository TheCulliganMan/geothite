# St Cloud: native wind gym and river crossing

[Complete town](../renders/st-cloud/whole-town.png) ·
[Native gym facade](../renders/st-cloud/native-gym-facade.png) ·
[Native gym entrance](../renders/st-cloud/native-gym-interior.png) ·
[Native river bridge](../renders/st-cloud/native-river-bridge.png)

The 24 × 18 city now publishes its four authored facades, clinic, mart and
public station, with the existing canonical 9 × 11 wind gym. Wyrel's battle
opens the lower crossing; Hallen's win opens the upper route to Saven. Four
wind lanes execute native forced movement, including held opposing input and
safe return travel. The original names, parties, prior-badge4 requirement,
badge5, one Swift TM, source art and explicit wind collision remain canonical.
Only the gym's three city return bindings and landmark change.

| Room | Exterior door / safe return | Exterior active / passive | Interior passive arrival | Carpets |
| --- | --- | --- | --- | --- |
| Clinic | `(6,9)` / `(6,10)` | 1 / 2 | `(10,13)`, warp3 | `(10,15)`, `(11,15)` |
| Mart | `(14,9)` / `(14,10)` | 3 / 4 | `(6,5)`, warp3 | `(6,7)`, `(7,7)` |
| Gym5 | `(8,27)` / `(8,28)` | 5 / 6 | `(4,19)`, warp3 | `(4,21)`, `(5,21)` |
| Station | `(35,11)` / `(35,12)` | 7 / 8 | `(11,17)`, warp3 | `(10,19)`, `(11,19)` |

The two-core bridge is a necessary dry crossing over the Mississippi. Its
native side rails block water entry, the launch is separate, outer river rocks
retain actual BUOY collision, and the granite overlook uses the native ramp.
Source review proves the bridge has no free dry bypass, both walking/Surf
boundaries are safe, the overlook is reached only by its ramp and shows water
in the native player camera. No incidental Cut tree is placed. The published
whole town matches this exact source terrain.

Both real road mouths now connect reciprocally:

- City south `(24/25,35)` ↔ R05 north `(24/25,0)`, offsets `0` both ways.
- City north `(24/25,0)` ↔ R06 south chunk `MnR06C00R01 (16/17,39)`, city offset
  `4`, route offset `-4`.

All eight native edge projections are checked by the builder. Production input
crosses both lanes in both directions at each mouth. Existing route chunk links,
terrain, encounters, trainers, objects and scripts are preserved.

The 492-map pack uses the completed 488-map St Paul input and adds the town,
clinic, mart and station. Mankato and St Paul remain exact. Group77 and one city
landmark are additive; only town/clinic gain recovery entries in the addon
range. HOME and every pre-existing recovery record remain unchanged. Every
unrelated map, trainer, tileset, runtime file and compiled audio payload is
compared exactly. Installing the pack grants no event, badge or party member.

## Verified build and tests

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_st_cloud -- \
  content-packs/minnesota-st-paul-gym-remake.crystalpack \
  planning/minnesota/towns/MnStCloud.json \
  planning/minnesota/towns/MnStCloud.layout.json \
  planning/minnesota/transit/MnStCloudStation.json \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-st-cloud-gym-remake.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_st_cloud.py \
  --pack content-packs/minnesota-st-cloud-gym-remake.crystalpack --verify-published

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test st_cloud -- \
  --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-st-cloud-gym-remake.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-st-cloud-gym-remake.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_st_cloud_city_controller -- --ignored --nocapture --test-threads=1
```

Both asset tests pass. They compare the written candidate with a fresh build,
check all preservation contracts and reject a stale gym reference, misplaced
facade, active-carpet station arrival and duplicate installation.

Both production-controller courses pass. The gym course proves both road mouths,
the trainer order, wrong-direction rejection, all four wind lanes with opposing
input, actual battle move PP use, badge5 and one TM, Continue, both city exits
and post-victory reentry. The public-room course enters every other facade,
uses both carpets, saves/Continues, restores HP/status/PP at no cost, registers
clinic recovery, opens the native PC and purchases a Super Potion for its
actual price. The conductor blocks walking onto the unfinished platform; its
train doors are unreachable. Ordinary station access never grants the courier
completion event. No playable rail service is claimed.

Whole-city and native facade/entrance/bridge images were inspected and retained.
The source/published terrain, collision, warp and route comparisons pass.
All 377 audio programs pass PCM hash, frame-count and loop-range checks.
Changed Rust leaves pass rustfmt. Workspace formatting still reports unrelated
changes beginning in `offset-allocator/src/lib.rs:408`; full all-target clippy
is blocked by the existing bad bit-mask comparison in core multiplayer code.
Scoped all-target clippy passes for assets and Bevy with existing `never_loop`
findings allowed; neither run reports a warning in the changed St Cloud code
or gym controller tests.

The historical non-gym reconciliation audit remains available as
`--audit-reconciliation`. Its retained snapshot is unchanged. It fails on
pre-existing original-name corrections, publication status and the new native mart service/stock;
the station's no-trains dialogue is also newer than that snapshot. Current
source geometry and canonical-gym checks run separately. The stale hardcoded
12 × 12 gym inventory check now resolves the canonical 9 × 11 source dimensions.
Old source passes, rejected passes, images and conversions remain archived.

## Remaining progression

Initial street position, legal level100 Mew, prior badges1–4 and service
injury/PP/money are explicit fixtures. They do not prove ordinary-party balance
or an earned regional journey. This local city chain still needs merging with
the opening/story/northern candidates, refreshing their obsolete gym versions
and verifying actual sequential city progression. Courier/rail and remaining
regional story gates stay separate unfinished work.
