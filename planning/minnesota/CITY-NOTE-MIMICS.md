# City note disguises

The existing family is **Jotlet (MN_186) → Memofold (MN_187) → Recallisk
(MN_188)**, active roster family MN_F087. Its original habitat includes Twin
Cities libraries, workshops and skyway bulletin corners.

The initial v20 source inventory contains no ambient city papers: its only remaining
`SPRITE_PAPER` is Mankato's quest notebook. This overlay therefore adds optional
papers instead of repurposing quest objects. No existing object, dialogue,
terrain, warp, story trigger, trainer, grass table, asset or audio is changed.

`city-note-mimics.json` authors eight small paper discoveries each in Minneapolis,
Saint Paul and Duluth. Two per city (25%) are disguises; six remain ordinary
reminders. The existing native `SPRITE_PAPER` appearance is identical until the
player interacts. A short reveal is acknowledged before an ordinary catchable,
escapable wild battle. No note gives an item or quest progress.

The seed, minority fraction, explicit positions, family members and levels are
source configuration. Selection uses a stable SHA-256 ordering of city and note
index at pack build time. All saves using the same pack share those placements;
this initial implementation does not randomize each player's city. It consumes
no gameplay RNG and cannot be rerolled by reloading, repeated reads or map entry.
Changing the source produces a new pack identity rather than silently changing
an existing save's world.

Each city has one **level-6 Jotlet** that bypasses challenge level scaling. The
other slot is Memofold at a level-24 floor (Minneapolis), Jotlet at a level-12
floor (Saint Paul), or Recallisk at a level-44 floor (Duluth); these use the normal
party-based challenge scaling. Evolving the player's own caught Jotlet remains
normal. An encounter's level never substitutes its species with an evolution.
The low-level and challenge opportunities are distinct and testable separately.

A unique ordinary event flag hides each revealed mimic. It is set after the
reveal's acknowledgement and before battle starts. Catch, defeat, flee and
blackout all consume that paper, like a finite static item-disguise encounter.
There are no daily/map-entry respawns and no repeated rewards. A save before
acknowledgement retains the ordinary script cursor; a save during battle retains
the native static-wild origin and resume cursor. Loading an older pre-interaction
save can replay that encounter, as other unsaved gameplay can, but cannot change
its disguise/species. A partyless player receives a short hint and leaves the
paper available. Ordinary papers remain readable.

The publisher rejects duplicate identities, unsupported cities, invalid family
members/levels, occupied tiles, warps, coordinate/background events and
unreachable placements. It verifies that adding paper bodies removes no
previously reachable city tile except the paper tiles themselves. Existing
scripts and all unrelated maps are retained exactly. The candidate is isolated;
no default pack selection, remote publication or story composition is performed.

The isolated candidate uses the verified harbor-v02 parent selected when the
final checks began, retaining its North Shore shops and harbor repairs. Later
concurrent city revisions must be composed through the publisher and rechecked;
they are not implicitly included in this candidate. The initial v20 parent is an additional
compatibility check. Build from a completed local pack (choose a new ignored
output filename):

```sh
CARGO_TARGET_DIR=/tmp/geothite-city-note-build CARGO_INCREMENTAL=0 \
  cargo run --locked -p crystal-assets --bin pack_city_note_mimics -- \
  content-packs/minnesota-current-duluth-harbor-v02.crystalpack \
  planning/minnesota/city-note-mimics.json \
  content-packs/minnesota-city-note-mimics-v01.crystalpack
```

Focused source/pack checks:

```sh
CRYSTAL_CITY_NOTE_BASE_PACK="$PWD/content-packs/minnesota-current-duluth-harbor-v02.crystalpack" \
  cargo test --locked -p crystal-assets --lib city_note_mimics \
  -- --include-ignored --test-threads=1
```

Production controller checks require `CRYSTAL_CITY_NOTE_TEST_PACK` pointing to
the generated candidate and `CRYSTAL_CITY_NOTE_SAVE_DIR` pointing to an isolated
ignored directory. Run the `city_note_mimics` filter in `crystal-bevy --lib` with
`--ignored --test-threads=1`. Explicit initial positions, parties and balls are
fixtures; these checks do not establish earned campaign progression. Runtime
results and visual review must be recorded separately from source inspection.

Ordinary Minnesota wild scaling also preserves native levels. Grass, caves,
Surf and rod fishing reuse the existing rate/bite byte: residues divisible by
four retain the selected table slot's authored level; the other residues use
ordinary challenge scaling. This reserves one quarter of the byte values
(successful-hit proportions depend on the discrete rate cutoff).
The species-slot draw, encounter rate, rarity, ecology, time/zone/swarm table,
and RNG consumption are unchanged. Early routes retain genuinely low levels
even with eight badges and a level-80 party. Late habitats retain their own
native levels rather than acquiring artificial level-2 creatures. Repel may
intentionally filter these low levels. Wild species never auto-evolve merely
because their level increases.

This broader policy is Minnesota-only. Johto/Kanto, roaming creatures, contest
rules, special script-driven field habitats and authored static/story battles
retain their existing behavior. The city-note native slots use their separate
exact script exemption. No daycare custody or save fields change. Actual-table
and paired-production-roll tests use the `wild_level_refuges` filter in the assets
library, with the same explicit base-pack prerequisite.

## Verification recorded October 30, 2026

The ignored candidate `content-packs/minnesota-city-note-mimics-v01.crystalpack`
verified with content identity
`f57aac3657d4c0365df1f25bb7001ec66296120c44aa2c5f6a9e2990af5a3a4c`.
Its parent is `minnesota-current-duluth-harbor-v02.crystalpack`, identity
`10e6681625aa566d083fefe49c8f94d0b82a44a456971a2c1427cead98793843`.
All 729 unrelated maps, original city geometry/interactions, creature data,
encounter catalogs, audio and runtime assets are preserved by the overlay.

Passed checks:

- Three asset tests against the harbor parent, plus the same three against v20.
- Four scaling tests, including every actual ordinary Minnesota table at high
  progression, paired production grass/Surf rolls and real rod fishing.
- Four production-controller tests covering nine city/outcome combinations,
  pre-reveal and midbattle save/Continue, repeats, empty party, ordinary papers,
  blackout, and the raw graphical Continue menu before its first input tick.
  The final rerun passed all four in 62.86 seconds. These are explicit fixtures
  rather than an earned playthrough.
- `cargo clippy --offline --locked -p crystal-assets --all-targets` and
  `cargo clippy --offline --locked -p crystal-bevy --all-targets --features location-tester`.
- Scoped Rust formatting and whitespace checks. `cargo fmt --all -- --check`
  reports pre-existing formatting differences across the shared repository;
  unrelated files were not reformatted.

Current-source native 2D screenshots were actually inspected: paper appearance
in all three cities, the fully printed two-line reveal, and a level-6 Jotlet
battle against a level-80 partner with FIGHT, PKMN, PACK and RUN visible. The
renderer executed the actual encounter script from a declared note-side save
using ordinary synthetic keyboard inputs, without changing the input save.
The retained receipt and logs are under `/tmp/city-note-verification`, with
captures under the task workspace's `note-native-captures-v02-current` directory.
An older renderer from an unrelated worktree was excluded from acceptance.
The 3D-only HOLD_BATTLE observer is unsuitable for native 2D; the successful
capture uses a normal timed input route instead.

Native rendering also exposed an existing Continue initialization gap: the
frontend could draw a saved live battle before creating its normal action
cursor. The narrow restore fix initializes that chooser before the first frame,
without adding save fields or advancing gameplay. Its dedicated first-frame
regression and full controller rerun pass, and the updated Bevy all-target
Clippy check passes. The rebuilt current-source renderer also loaded and
rendered that native battle successfully, with the complete chooser visible and the input save
unchanged. This was visually reviewed separately from the source tests.
Builds were serialized with the shared Cargo slot, explicitly released to the
next daycare rebuild before the final capture. All task-owned Cargo caches
are cleaned after retaining the verified executables and review evidence.
The candidate remains unselected; no push, merge or deployment was performed.
