# Bemidji: two Cut hedges and a victory return

The6×10 native garden keeps its two offset Cut openings and two required
trainers. A gold gate in the south hedge opens after Orlis; one bush above it
becomes lawn so the gate leads somewhere. No additional switch, pad or field
move is introduced. This revises two metatiles and the return hints; all actors,
parties, requirements, native Cut replacements and city doors remain intact.

![Six illustrated room states](../renders/gyms/bemidji-return/sequence/bemidji-return.gif)

1. Beat Moslen in the foyer.
2. Cut the west tree and cross the garden to Elvren.
3. Beat Elvren, then cut the east tree to reach Orlis.
4. Beat Orlis. The gold gate opens for the direct return.

Brindel remains optional. Cutting early cannot substitute for either required
battle. Native Cut still requires a party member with the move and the original
badge permission. The guide's existing once-only HM gift remains unchanged.

| Feature |16px walking coordinates, except gate |
| --- | --- |
| Arrival / paired carpets | `(4,17)` / `(4,19)`, `(5,19)` |
| Moslen / Elvren / optional Brindel | `(7,14)` / `(8,9)` / `(3,9)` |
| West / east Cut tree | `(3,12)` / `(7,6)` |
| Orlis / approach | `(5,3)` / `(5,4)` |
| Victory gate / cleared approach | Metatiles `(3,6)` / `(3,5)` |
| Actual city return | `MnBemidji (10,10)`, exterior warp2 |

With both trees already pruned, the shortest return from `(5,4)` to `(4,17)`
is22 steps while the gate is closed and18 when open, including actor collision.
The fixed18-step control run uses `RR DDDDDDDDD L DD LL DD`. The gate stays
closed until the leader win. Walking through a cut opening records its existing
pruned event; both native replacements preserve their other collision quadrants.

## Executed publication and verification

Ignored `content-packs/minnesota-bemidji-return.crystalpack`:541 maps, identity
`95c4cd685949c3334c7691b48e8088ace5212a9659727ce32a2713259291072c`.
It extends the Brainerd dock-control pack and preserves the first six revisions.
Later city/story lineages still need integration. Existing saves require
migration review because private art indices change; installation grants no
badges, party members or progress.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_bemidji_return -- \
  content-packs/minnesota-brainerd-winch.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/bemidji-return.json \
  content-packs/minnesota-bemidji-return.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-bemidji-return.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  bemidji_return_literal -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test bemidji_return_revision --test brainerd_winch_revision \
  --test st_cloud_arrow_revision --test st_paul_loop_revision \
  --test minneapolis_circuit_revision --test mankato_terrace_revision \
  --test rochester_gate_revision -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-bemidji-return.crystalpack --audio --desktop

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-bemidji-return.crystalpack --verify-published \
  --output planning/minnesota/renders/gyms/bemidji-return/all-current
```

The literal production-controller course uses initial Typhlosion41, Crobat40
and Sandslash39, six prior badges, HM01, entrance placement and deterministic
RNG. Sandslash starts with legally taught Cut; the others retain native level-up
moves. All subsequent walking, Cut prompts, battles, saving, Continue, rewards
and city returns use production inputs. These fixtures do not prove earning
the party, HM, badges or regional journey. No healing medicine or subsequent
HP/PP/progress/position writes are supplied. The run earns both required
trainers and Orlis, verifies Brindel remains unbeaten, then takes the18-step
return, both carpets and reentry. Cut flags persist; the badge/TM cannot repeat.

All four existing Bemidji controller cases pass on the final pack: complete
garden/optional trainer/save/city returns; missing move/badge/cancel; cutting
ahead without required trainers; loss and missing sixth badge. The preceding
Brainerd ordinary-team course also passed on the provisional publication;
the final correction only canonicalizes Bemidji's coordinate scene IDs.

All fourteen asset cases across seven revisions pass. They verify identity,
uncached native scripts, other rooms, trainer identities/parties, indexed warps,
metadata, unrelated art/runtime files, audio and native Cut replacements.
Only the new return event is appended to constants; existing values stay exact.
The publisher normalizes an unconditional cached coordinate scene to native
`-1`, so cached and uncached parsing agree. Negatives reject early opening,
missing approach clearance, bypass, altered Cut replacement or party, and
duplicate installation. Every collision outside the cleared bush remains exact;
the closed gold gate retains the old hedge's solid collision.

All377 audio hashes, frame counts and loop ranges pass. Both locked/open native
cameras at `(6,15)`, noon/facing down, match all368640 source pixels each. The
whole room, six illustrated source states and native captures were inspected;
the GIF illustrates states rather than recording play. All originals and
conversions remain archived. All eight selected source terrains and collision
projections match publication. The current source index and world/interior
contracts select this revision; plan validation passes20 routes,19 hubs,
57 chunks,88 contracts and288 maps. Duluth remains to review.

Workspace formatting still first differs at the pre-existing
`crates/offset-allocator/src/lib.rs:408`. Full dependency Clippy stops at the
pre-existing always-false mask in `crates/crystal-core/src/multiplayer/mod.rs:1332`.
Scoped changed-file rustfmt passes. All-target scoped Clippy passes for both
affected crates with existing warnings using `--no-deps -- -A clippy::never_loop`;
no warning points to the new revision or Cut/gate changes.
User visual acceptance, later regional integration and a complete earned
campaign remain open.
