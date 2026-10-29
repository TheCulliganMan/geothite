# Iron Range → Grand Rapids → Duluth's eighth gym

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

[All eight current rooms](../renders/gyms/leech-shore-audit/all-gym-interiors.png) ·
[Room designs](../GYMS.md) · [City progression](../PROGRESSION.md)

The current 541-map `content-packs/minnesota-northstar-leech-shore.crystalpack`
contains the Range roads, mine, Grand Rapids depot, Duluth services and the
current ice gym. This audit consumes that published pack without modifying it.
Internal content identity:
`43b3d65bda72a8904cfad17781cc0da408045c64d91ee22fb5c6b3b1634006db`.

One continuous production-controller course now covers the final gym approach:

1. The locked mine cache refuses an early pickup. Defeat the actual foreman.
   Victory alone leaves both meter recovery and depot clearance false.
2. Separately accept the meter and acknowledge its recovery text. Save/Continue,
   use the mine's real exit, walk R09 and enter Grand Rapids through its checkpoint.
3. Use native free clinic care. The east checkpoint still refuses departure:
   winning and carrying the meter cannot substitute for its handover.
4. Enter the actual survey depot, accept the separate handover, read its text
   and Continue. Walk the now-qualified checkpoint and all of R10 to Duluth.
5. Use Duluth's clinic and actual gym facade. Defeat Livra, Tordel and Anver
   through the ice course. The eighth badge is still absent when the court opens.
6. Save/Continue, retreat through the opened return course and use the real
   clinic. Re-enter: all three trainer wins and the physical openings persist.
7. Defeat Norek, receive Harbor and one Icy Wind TM, Continue, and repeat his
   interaction without receiving another TM. The separate harbor signal remains
   false; the gym never silently acknowledges the workshop's story pages.
8. Exit through the real facade, heal, walk R10 home through both checkpoints,
   and Continue in Grand Rapids with all custody flags, eight badges and one TM.

**Initial setup is explicit:** mine placement `(13,18)`, seven prior badges,
the upstream headwaters report and legal level45 Mew/Lapras with specified
learnable review moves. The report is supplied, not earned by this course.
The existing deterministic input fixture is initialized once. No position,
party, resource, badge, event or RNG is rewritten after setup.

A solo level45 attempt exhausted Psychic PP; the next solo attempt lost.
The passing course uses two partners, type advantages and free clinic care.
Test battle control supports native faint/switch commands, checks real PP consumption
across the party and requires a surviving partner. It does not seed healing or
raise the solo partner to level100. This is a stronger local integration case,
not proof of normal starter availability, full-team balance or an earned
Winona-to-Duluth campaign.

## Executed command

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-leech-shore.crystalpack" \
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-leech-shore.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  gym_eight_approach_controller -- --ignored --nocapture --test-threads=1
```

The course passes in25.76 seconds. After the shared battle helper gained
reserve-switch support and available-move selection, the combined eight-city local
gym course also passes again in76.19 seconds. Those eight fixtures remain
separate initial placements rather than a continuous earned badge journey.

Current source publication was rechecked for all eight rooms: every terrain
pixel and native collision quadrant matches the authoring data, including the
explicit directional wind permissions. The complete rooms were visually
inspected together at one native scale. All eight complete map modules also
remain byte-equivalent as structured data from the 538-map regional gym
publication to this 541-map story successor: scripts, gates, objects, battle
requests and city returns are preserved. Every generated source render and
conversion is retained in the immutable render history.

The final interior audit also passes eight focused required-trainer/puzzle-gate
cases and sixteen loss/prior-badge/court refusal cases on the canonical room
pack. The focused return helper derives the actual compiled carpet/arrival
pair and proves re-entry, so single-room and all-eight review halls no longer
conflict through a hard-coded door-zero assumption. Scoped formatting and
all-target Bevy Clippy pass with the existing allowance and unrelated warnings
recorded in the [implementation audit](../gyms/implementation-audit.md).
