# St Paul: native pad gym and public city doors

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

[Complete town](../renders/st-paul/whole-town.png) ·
[Native facade](../renders/st-paul/native-gym-facade.png) ·
[Native entrance](../renders/st-paul/native-gym-interior.png) ·
[Closed grille](../renders/st-paul/native-closed-court.png) ·
[Open grille fixture](../renders/st-paul/native-open-court-fixture.png)

The 32 × 24 city now publishes its existing native-source layout, clinic, mart,
Capitol hall and public Union Depot. Gym4 references the canonical 7 × 10 room
by object digest and glyph map. It retains Belva, Orven and Brannic, both required
trainer victories, four reciprocal pad pairs, the real court grille, prior
badge3 requirement, badge4 and one Steel Wing TM. All authored character names
are original. The city builder changes only the gym's first three exterior warp
bindings and landmark; all eight internal pad records remain exact.

| Room | Exterior door / safe return | Exterior active / passive | Interior arrival | Active carpets |
| --- | --- | --- | --- | --- |
| Union Depot | `(44,37)` / `(44,38)` | 1 / 2 | `(15,21)`, warp5 | `(14..17,23)`, warps1..4 |
| Clinic | `(7,15)` / `(7,16)` | 3 / 4 | `(3,5)`, warp3 | `(3,7)`, `(4,7)` |
| Mart | `(23,15)` / `(23,16)` | 5 / 6 | `(3,5)`, warp3 | `(2,7)`, `(3,7)` |
| Capitol hall | `(34,9)` / `(34,10)` | 7 / 8 | `(3,5)`, warp3 | `(2,7)`, `(3,7)` |
| Gym4 | `(54,15)` / `(54,16)` | 9 / 10 | `(2,17)`, warp3 | `(2,19)`, `(3,19)` |

The native city bank jumps from `HOP_DOWN (39,20)` to `(39,22)`, blocks the
reverse through `(39,21)`, and retains the ordinary brick return. Source review
measures 27 walking steps from the landing back to the approach `(39,19)`.
Production input executes the actual hop. The river stays enclosed; native
water cannot reach an unconnected outer edge. No incidental Cut tree is placed.
The two guards physically seal the unbound fairgrounds street.

Both public west-mouth lanes now connect to the installed R04 route:
St Paul `(0,24/25)` ↔ R04 `(55,18/19)`. Town west offset is `3`, route east offset
is `-3`. The builder verifies all four native projections. Production input
crosses both lanes in both directions. R04 terrain, encounters, actors and
scripts remain untouched; its only changes are the declared east connection
and connection flags. This is a local seam, not proof of the full regional gates.

The 488-map candidate uses the current 483-map Mankato candidate as input and
adds five maps. Mankato remains intact. Group76 and the city landmark are
additive; only the town and clinic receive new recovery records, in the addon
range. HOME and all pre-existing recovery entries remain exact. Unrelated maps,
all trainers, tilesets, runtime files and compiled audio remain exact.

## Executed build and checks

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_st_paul -- \
  content-packs/minnesota-mankato-gym-remake.crystalpack \
  planning/minnesota/towns/MnStPaul.json \
  planning/minnesota/towns/MnStPaul.layout.json \
  planning/minnesota/transit/MnStPaulUnionDepot.json \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-st-paul-gym-remake.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_st_paul.py \
  --pack content-packs/minnesota-st-paul-gym-remake.crystalpack --verify-published

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test mankato --test st_paul -- \
  --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-st-paul-gym-remake.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-st-paul-gym-remake.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_st_paul_city_controller -- --ignored --nocapture --test-threads=1
```

Four asset tests pass: two St Paul tests plus the two Mankato regressions.
Two St Paul controller checks pass, and both Mankato city controller regressions
pass on the resulting pack. The gym course executes all pad pairs both ways,
saves/Continues on their arrival pads without bouncing back, earns the practice
and leader victories through selected moves with PP use, gets badge/TM once,
and proves both city exits and reentry. The service course uses every public
room carpet, saves/Continues, restores HP/status/PP at no cost, sets clinic
recovery, opens the native PC, and purchases a Super Potion for its actual price.

Whole-city, room and native player images were inspected and retained. The
native open-grille image seeds the two trainer-victory/court flags explicitly;
it is a visual fixture. Earned grille opening is proved separately by the
controller course. Source/published room terrain and all eleven gym warp records
match. All 377 audio programs pass PCM hash, frame-count and loop-range checks.
Changed Rust leaves pass rustfmt; full workspace formatting still reports
pre-existing differences beginning in `offset-allocator/src/lib.rs:408`.
Full all-target clippy remains blocked by the existing bad bit-mask comparison
in `crystal-core/src/multiplayer/mod.rs:1332`.
Scoped all-target clippy passes for assets and Bevy with existing `never_loop`
findings allowed. No warning points to the changed city modules, shared
publication/room/geometry modules or gym controller tests.

## Remaining scope

Initial city placement, level100 Mew, prior badges1–3, injury/empty PP and money
are explicit local fixtures. These do not prove ordinary-party balance or an
earned whole-region journey. Later [regional publication](eight-gyms.md) adds
St Cloud and all eight city-bound gyms. The [fourth-gym approach](gym-four-approach.md)
also verifies actual Minneapolis/R04 arrival, earned third/fourth gym courses
and return from an explicit local fixture; the earned whole journey remains open.

The depot is currently a public room, with four real exit carpets. Its conductor
physically blocks the unfinished train platform and says no trains run today.
Neither station access nor talking to the courier grants Q03 completion. The
native room course tries to walk past the conductor and requires rejection;
both train doors and the platform remain unreachable in the collision graph.
The
courier battle, acknowledged DEEP BORE manifest, station-visit progression and
real rail menu/travel remain implementation work. The native public-room test
asserts the courier-completion event stays unset. There is no playable-train
claim, and no user visual acceptance is inferred from passing tests.
