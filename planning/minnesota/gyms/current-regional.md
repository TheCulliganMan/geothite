# Current gym revisions in the regional packs

This record describes the two frozen gym publications named below. The current
[gym profiles](../GYMS.md) summarize the courses; the
[story composition record](../revisions/current-story-v01/README.md) describes a
later regional publication. Results here retain their original pack and fixture
scope. Legacy test parties are historical evidence, not approved Minnesota teams.

All eight selected rooms now rebuild through Rust inside two frozen regional
branches. The selected source is [current.json](current.json), which layers the
eight single-room revisions over the unchanged historical collection.
`read_minnesota_gym_design` resolves that index, checks unique identities and
neighboring source paths, then validates the complete design. The original
`pack_minnesota_gyms` review command also accepts this index.

`refresh_current_minnesota_gyms` and `pack_current_gyms` recompile the private
art, room geometry, native scripts and gates. They preserve each parent's
existing exterior destination records, map identities, trainer rosters/numbers,
story content, services, recovery and audio. New gym events are allocated after
the parent's existing events; existing numbers remain exact. Rochester retains
its parcel requirement. No player progress is supplied by installation.

| Branch | Frozen parent | Published current-gym pack |
| --- | --- | --- |
| Story and Ely League, 554 maps | `minnesota-northstar-ely-league.crystalpack` | `minnesota-current-gyms-story.crystalpack` |
| Compact Twin Cities, 660 maps | `minnesota-twin-cities-compact-v01.crystalpack` | `minnesota-current-gyms-cities.crystalpack` |

All packs are ignored under `content-packs/`. Exact content identities:

- Story parent: `e96cd37fd94ca71f39d72a4a7924fe613ddb1cecf64f12f5a5985095a5fea7d4`.
- Story result: `28efc3f0246504a8f8c3628de972f173bec9c4c25f2b1b2d65b2f69e220c0d31`.
- City parent: `23b3ca0c19fbc6c7f687645540aa9bdd12becfbdb7b693804d6f3b125a41ed41`.
- City result: `a26ece5a7431db9d0c3e33a8921ebc3b985eb0240ef02e160c2757dbcbaa466f`.

The publisher deliberately certifies these two parent identities and eight
reviewed room hashes. An unreviewed parent, stale source, missing revision,
altered name/reward or repeated installation fails. It does not silently merge
the separately owned Fair/species branch or redesign any city.

## Executed build and checks

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_current_gyms -- \
  content-packs/minnesota-northstar-ely-league.crystalpack \
  planning/minnesota/gyms/current.json \
  content-packs/minnesota-current-gyms-story.crystalpack

target/gym-interiors/wasm-dev/pack_current_gyms \
  content-packs/minnesota-twin-cities-compact-v01.crystalpack \
  planning/minnesota/gyms/current.json \
  content-packs/minnesota-current-gyms-cities.crystalpack

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test current_gym_refresh \
  -- --include-ignored --nocapture --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-current-gyms-story.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  current_gyms_controller_ -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-current-gyms-story.crystalpack --audio --desktop
target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-current-gyms-cities.crystalpack --audio --desktop
```

The same door controller case also passed with `CRYSTAL_GYM_TEST_PACK` set to
the city result. It exercises both actual carpets and city reentry in every
room, with Continue after each return: sixteen paired transitions per branch.
Each room begins as a separate explicit entrance/prior-badge/party fixture.
These cases do not claim earned travel between towns.

The original review CLI was also built and executed with the current index,
the `minnesota-boundary-basecamp-connected.crystalpack` input and slot8. It
published and read back one gym plus its review vestibule successfully; the
task's `/tmp/current-gym-duluth-review.crystalpack` is verification scratch.
An initial compile hit a concurrently added item field before its type was
available. The type completed and the same command then passed; no unrelated
item or Fair source was changed for this task.

All four asset cases pass. They recompile and compare written identities,
uncached native rooms, original exterior wires and complete unrelated datasets.
Only explicitly owned room/script/block/art entries, owned Cut replacements,
room dimensions and appended gym events are normalized. Original constants,
parties/numbers, other maps, metadata, encounters, services, spawn/recovery,
HOME and all unrelated runtime payloads remain exact. Loader tests reject
duplicate selections, room mismatches, escaping paths and unknown fields;
historical or altered reviewed designs and duplicate installation fail.

Both packs pass all 377 PCM hash, frame-count and loop checks. All eight source
terrains and collision projections match publication in each branch. Sixteen
headless native arrival cameras match all 368,640 source pixels each, at noon,
facing south with explicit empty-party placement. Both complete native sheets
were inspected. Originals, transformations and earlier rejected renders remain
archived under `../renders/gyms/current-regional/`.

All eight full puzzle courses pass on the story result. They use the ordinary
parties and fixed walking/slide/pad instructions recorded in each room's runbook,
earn every required trainer and leader, spend actual PP, verify the one-time
badge/TM, Continue and both actual city carpets. St. Cloud's integrated replay
adds four initially carried Hyper Potions and native care after Wyrel/Hallen;
its earlier medicine-free two-partner trial lost. Duluth retains its documented
four-partner/medicine fixture. The shared Pack input helper verifies HP effects,
unchanged PP and actual item consumption; it does not write party or progress.
Opponents and gym designs remain unchanged. These are separate local fixtures,
not one continuously earned regional party.

Rochester's complete switch course and Bemidji's complete Cut/victory-return
course also pass on the city branch, exercising new event states against that
parent's distinct event table. The final Duluth story rerun passes after the
care helper extraction. No later input or state grant is supplied by the helper.

## Scope and saves

The [city-by-city progression design](../PROGRESSION.md), badge order and story
gates remain intact. These publications integrate the eight gym revisions into
their two existing regional branches; they do not implement the still-unearned
remaining campaign or claim an entire fresh Winona-to-League playthrough.
User visual approval is separate from the recorded source/native review.

Use a fresh compatible save for these results. Old saves inside revised rooms
need position and block-override migration review before switching pack identity.
No migration or downstream progress is silently applied.

Workspace formatting still first differs at pre-existing
`crates/offset-allocator/src/lib.rs:408`. Full dependency Clippy retains the
pre-existing always-false mask in `crates/crystal-core/src/multiplayer/mod.rs`.
Scoped changed-file rustfmt and all-target Clippy for both affected crates pass
with existing warnings using `--no-deps -- -A clippy::never_loop`. No warning
points to the new selection, publisher, preservation, door or care code.
