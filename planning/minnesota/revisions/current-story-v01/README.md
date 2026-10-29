# Current regional story composition

This is a publication record for the exact packs and source state described
below. Later edits to the same authoring paths do not update those packs.
The current [story profile](../../STORY.md) and
[character profiles](../../playable/northstar-character-story.md) describe the
authored direction. No tests or publications were rerun for the Markdown cleanup.

The older 567-map rival-arc publication is historical. It contains the retired
oversized North Shore/Gunflint layout and is never used as a source here. This
revision runs the authored story builders against the accepted current regional
pack, retaining compact geography, active Minnesota species and later map work.

## What is installed

- Six Reed battles: Winona, Rochester, ICE relay, St Paul, the mine and Boundary
  intake. Recruitment, transfer work, the worker intervention and aftermath give
  him reasons to appear between battles. He knowingly chooses the alliance and
  remains hostile after losing it.
- The ICE headquarters network: cargo/relay plus four compartmented floors,
  panel pairs, a real key card, six persistent doors, optional rooms, holding
  power and a separate rescue. Defeating Reed cannot release the captive.
- The interleaved Lode mine: haulage, sump and ventilation shafts connected to
  the approach, gallery and core. Independent pressure controls open crosscuts;
  neither control substitutes for the drill shutdown.
- Twenty-four added ordinary sight grunts: eight headquarters, twelve mine and
  four intake guards, on top of existing opposition. Teams, legal moves, native
  sight and independent defeat events come from the regional faction policy.
- The acted finale: chief battle, acknowledged live link, rival battle, local
  drive shutdown, Kettlurk entering actual water, then crew clearance. Only the
  final acknowledged clearance restores the survey. No capture is required.
- Changed return conversations for allies, optional Reed aftermath, Ryan's
  existing Baptism River cabin and native gatehouses, and the fully written
  Fern/Flint/Veil/Halden/Casey League and Hall of Fame ending.

The authoring sources remain `story.json`, `story-events-v2.json`,
`story-arc-v3.json`, `story-pokemon.json`, the hand-authored `lairs/` layouts,
and `revisions/league-finale-v01/dialogue.json`. `composition.json` is an authored
exact-parent allowlist: a later parent is admitted only after inspecting its
residual changes. Every chapter still checks its native program/layout anchors.
The old individual publishers retain their original parent guards; only the
private in-progress composition marker enables the reviewed bound-parent path.
No partial pack containing that marker is returned or published.

## Publication

The first complete current composition is
`content-packs/minnesota-northstar-current-story-v01.crystalpack` (704 maps),
SHA-256 `e3e7395e620697f51b69eef1dc4c4c2c62db9a268dcf8811eed800cc631ae7cb`.
Its content identity is
`941f9961d1dfcb92c8587c0708c1be05ce96e52ac24a8dfcf23439f6e61c1f9a`.
A fresh replay after strengthening ordinary-team preservation produced the exact
same bytes in `minnesota-northstar-current-story-v02.crystalpack`.

The final combined candidate is
`content-packs/minnesota-northstar-current-story-v03.crystalpack`. Its parent
`minnesota-current-story-parent-r05-r01-v01.crystalpack` merges the exact accepted
R01 limestone cave onto the accepted R05-grass/League child of R20 using
`revise_authored_cave`. Independent comparison permits only the R01 room and
its two geometry mirrors; the other 690 maps and all teams/programs/art/audio
remain exact to that R05 parent.
It is identity-equivalent to the then-selected
`content-packs/minnesota-current-grass-r05-v03.crystalpack`, content identity
`b04a7ba2bbfe1c930eef15d41662e0a7c7bbd1bc64923da4fb1b10d47305f37f`.
Final story-v03 file SHA-256:
`560e49986d14e48c07526e9878eddc69be46d546ad079774f04b10f58e7e5c88`;
content identity
`f2808268fbda859cf97b79b8460086c8fc0251cc4870f68504d58f1426a822aa`.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-current-grass-r05-v02.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR01Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR01Cave1F.json \
  content-packs/minnesota-current-story-parent-r05-r01-v01.crystalpack
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_current_story \
  content-packs/minnesota-current-story-parent-r05-r01-v01.crystalpack \
  planning/minnesota content-packs/minnesota-northstar-current-story-v03.crystalpack
```

Use fresh ignored output filenames for replays. The resulting 704-map pack adds
13 authored rooms and changes 16 existing story modules; the seven written
League modules are already present in this merged parent and remain exact.
The initial R20 composition changed 23 existing modules including those seven.
It preserves all
outdoor terrain and connections, ordinary encounter tables, species/moves,
every existing art/runtime file, audio, title and the optional Vergloom module.
Only newly added/changed story parties are bound. Existing ordinary trainer
moves, levels, items, DVs and AI are retained. Five duplicate Reed names become
Halden, Bram, Nils, Otto and Eli; their parties are unchanged.

The final candidate differs from story-v01 only in the accepted R01 cave and
both R05 route chunks. All other 701 complete modules, the entire trainer
catalog, story constants, runtime map registry, art/runtime files and audio are
exact. Fresh final-pack saves are kept separately; no saves cross identities.

## Acceptance record

On story-v01, the earned New Game opening passed in 54.17 seconds. All 24 new
grunt fights, chief/link/rival/release/crew sequence, rival losses, optional
station aftermath, Rochester's three starters and St Cloud's two intervention
lanes passed. The St Paul retry case initially used the historical clinic/depot
coordinates; changing only its navigation to registered current warps gave all
three starter branches a pass in 108.61 seconds. This changes no game geometry,
party, reward or result. The continuous headquarters puzzle/rescue course passed
in 106.17 seconds. The two written-League cases (five real wins, ceremony,
credits, Continue and all five rematch-intro gates) passed in 116.01 seconds.

Production stdio Rochester/refusal, intake/refusal/retry/full release and station
aftermath replays passed, each with clean exit and byte-empty stderr. Native
headless 2D water/crew captures from reached same-pack saves were inspected at
`renders/story-arc-v3/current-native-v01/`. Geometry/art preflight checks original
native block pixels, all sight-grunt stations, the sealed/open chamber and every
scripted creature/crew path.

The final combined pack's earned opening passed again in 57.56 seconds. All seven
rival/grunt cases passed on that exact pack in 378.81 seconds, with fresh saves
in `.local/northstar-current-story-v03-saves/`. All three fresh production stdio
replays passed again with byte-empty stderr. The native desktop verifier passed
all 377 PCM hashes, frame counts and loop ranges. Scoped Rust formatting and
Assets/Bevy all-target Clippy pass with the documented existing lint allowances;
workspace formatting still has unrelated baseline differences. The focused
regional-source test rejects stale curation and foreign bindings.

At publication, `world.json` selected `current_story_revision`, retaining the
accepted R05 and R01 provenance. Later regional selection is recorded by the
current `active_local_review_revision`; this record does not reset it.
Logs are ignored local evidence under
`.local/northstar-current-story-*`. Launch the selected game with:

```sh
target/route-grass/wasm-dev/geothite play \
  content-packs/minnesota-northstar-current-story-v03.crystalpack
```

Chapter fixtures declare their initial party, badges and prior quest access;
only the explicitly earned opening starts from New Game. Every subsequent
movement, dialogue acknowledgment, battle, PP/item mutation and Continue uses
`VisibleShellController`. These chapter proofs do not claim one uninterrupted
ordinary-party Winona-to-Hall-of-Fame campaign or complete balance acceptance.
