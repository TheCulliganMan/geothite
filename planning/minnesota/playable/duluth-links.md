# Duluth links and the drainage reveal — 2026-10-26

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

## Current local candidate

`content-packs/minnesota-northstar-drainage-ice.crystalpack` strictly reloads:
509 maps, 251 species, 251 moves and 377 audio programs. File SHA-256:
`7adefd3ae9ee6e75dac1691b19fa17f10db14fd317e347949ddb6c1631c1adae`.
Packs are external, ignored build output. No ROM, disassembly, generated PCM
or audio catalog is a deliverable.

The story now follows the user's direction: Team Lode and its ally Team ICE
want to drain the Boundary Waters for copper and nickel beneath the lakebeds.
Lode plans to trap Kettlurk to power its pumps. Minneapolis-based ICE poses as
police, welcomes the destruction and displacement, and kidnaps anyone it
suspects of opposition. Lode knowingly shields the abductions. Their methods
and consequences carry the menace; neither team is redeemed.

MS10 has ten native pages: the copied meter log, lake-drainage objective,
Kettlurk, ICE's destructive motive and kidnappings, prior warnings and the
northbound relay. Every page precedes the harbor signal. MS11's chief admits
knowing the risks and choosing drainage anyway. The separate MS12 receiver
choice and alignment acknowledgment stop the drainage operation before the
lakes are emptied. This refresh changes only the three existing harbor text
bodies and the existing chief/receiver text bodies and their derived views.

Minneapolis's enforcer, captive dispatcher and separate rescue are **authored,
not natively placed or playable yet**. The actual enforcer win must precede
release; release must precede reading the recovered dispatch copy. These
upstream flags are not granted by the current pack's dialogue refresh. The
Kettlurk apparition/machine/encounter, earlier earned chapters, remaining
regional links and League completion remain implementation work.

## Pack lineage

| Stage | Maps | File SHA-256 |
| --- | ---: | --- |
| Grand Rapids/cities | 502 | `d92815b3840038d99b6b8ef400894824fc5e800b97429fa13b9351c81fb2addb` |
| Harbor reveal, retained earlier dialogue | 502 | `c0526b238a9c2a8da74c69e22635b69ad9a0b27b2a60af154a5afb644acccacf` |
| Expanded Northwoods on that story | 509 | `149e7f5b69de2f450eaaaddd2e57cdfc3d72c1d28cee226a89b3b5bfdece0419` |
| Duluth links, retained earlier dialogue | 509 | `7b72b32228395ba776b9a6dc4b65d17c6a7009dce6e87c9c8109a20725ed95ea` |
| Drainage and ICE dialogue, current | 509 | `7adefd3ae9ee6e75dac1691b19fa17f10db14fd317e347949ddb6c1631c1adae` |

The earlier 502-map Duluth-link publication remains retained as
`minnesota-northstar-duluth-links.crystalpack`, SHA-256
`069ae16ae721d944778c2b8c96ae0aa13006f027a0fee44319635506cb5cc376`.
Its source is frozen under `revisions/duluth-connections-v1/`; its native tests,
MCP sessions and renders describe the former 32×72 shore only. The current
56×96 shore has eight exterior chunks, and the southern link is R03, not R02.

## Native walking contracts

The current separate authority is `towns/MnDuluth.connections.json`. It retains
the city author's town layout and services. Six declared metatiles open R10's
mouth and move the existing west sign two tiles north, with its text intact.
The expanded shore mouth already has correct floor. Four new ordinary officers
and eight coordinate events implement two-sided walking checks. These public
checkpoint officers are not implicitly members of Team ICE.

- West: Duluth `(0,24)/(0,25)` ↔ `MnR10C01R00 (31,24)/(31,25)`, offset 0.
  Town departure requires badge 7 and acknowledged depot handover.
- North: Duluth `(32,0)/(33,0)` ↔ `MnR11C00R03 (46,47)/(47,47)`, offset −7
  from town and +7 on return. Town departure requires all eight badges and the
  acknowledged harbor signal. Inner checks sit at y3, bodies at y2 and outer
  checks at y1 so a real incoming step can reach the enforcement event.
- Unqualified incoming players can reach town and free clinic care, then are
  refused outward travel. Neither gate gives badges, story flags or resources.

The common checkpoint emitter also serves Grand Rapids. Its frozen-publication
regression proves the refactor reproduces the entire prior native dataset,
art and audio. Geometry validation rejects offsets, misplaced guards, missing
patches and unguarded walking/Surf shoulders.

## Verification

- Two link asset cases and two harbor asset cases pass. The full installed-story
  refresh preservation/idempotence case passes: only authorized text changes;
  every other map, command, event, trainer, item, catalog, art and audio remains
  exact. Duplicate fresh installation still fails.
- Five Q07 component cases pass, including exact canonical native text, actual
  battle turns before victory, loss whiteout, separate receiver decline/retry,
  every badge/signal refusal and unchanged free care.
- All 377 PCM hashes, frame counts and loop ranges match on the written current
  pack. The source plan passes: 20 routes, 19 hubs, 57 chunks, 86 contracts and
  157 planned maps. Symbolic reachability is not an earned playthrough.
- Eight production-controller cases pass on the current pack: all missing badge/event refusals on both checkpoint lanes, actual harbor reading releasing the shore, both reciprocal seams and Continue, unqualified inbound free HP/status/PP care, ten-page reveal/input ownership, and all four chief/receiver victory/loss/retry/free-care cases. The chief's actual visible admission includes copper/nickel, deliberate drainage and ICE's abductions.
- Three actual stdio MCP sessions pass: the revised reveal and repeat plus START, and both directions of the second lane at each link. The relocated west sign stays readable. Every stderr stream is byte-empty.
- Source whole-map and actual noon/down-facing 2D review covers both checkpoints
  and the R10 arrival. All three reference/native comparisons are exactly
  0 different pixels out of 1,024,000. The current Northwoods terrain is kept.
  Every source image, crop, conversion, earlier comparison and rejected pass is
  retained under `renders/duluth-connections/`, including its expanded folder.
  This is local review, not final user visual approval.

Scoped Rust formatting passes. Workspace formatting first fails in the existing
offset allocator. Affected-crate Clippy currently stops in the unrelated existing `crystal-core/src/multiplayer/mod.rs:1332` (`clippy::bad_bit_mask`), before checking the new leaves. Earlier checks also recorded the eleven pre-existing mutation/parser `never_loop` failures; this run does not claim lint completion.

## Executed publication commands

The harbor input is the completed external candidate above. The expansion
uses the frozen before/after sources from the northern map authoring work:

```sh
target/northstar-story/wasm-dev/expand_minnesota_northwoods \
  content-packs/minnesota-northstar-harbor.crystalpack \
  planning/minnesota/revisions/northwoods-expansion/before/MN_R11.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/MN_R11.layout.json \
  planning/minnesota/revisions/northwoods-expansion/before/MN_R11.json \
  planning/minnesota/revisions/northwoods-expansion/after/MN_R11.json \
  planning/minnesota/revisions/northwoods-expansion/before/boundary-waters-safari.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.json \
  content-packs/minnesota-northstar-expanded-base.crystalpack
target/northstar-story/wasm-dev/connect_duluth \
  content-packs/minnesota-northstar-expanded-base.crystalpack \
  planning/minnesota/towns/MnDuluth.connections.json \
  content-packs/minnesota-northstar-duluth-expanded.crystalpack
target/northstar-story/wasm-dev/refresh_northstar_story \
  content-packs/minnesota-northstar-duluth-expanded.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-drainage-ice.crystalpack
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-drainage-ice.crystalpack --audio --desktop
```

Rust executables were built with the pinned toolchain, `CARGO_TARGET_DIR=target/northstar-story`,
`--profile wasm-dev --locked`, and the two affected package optimization
settings shown below. No shared source export workspace was run concurrently.

```sh
CARGO_TARGET_DIR=target/northstar-story cargo test --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  -p crystal-bevy --lib duluth_story_controller_ --no-run
RUST_MIN_STACK=33554432 \
CRYSTAL_DULUTH_STORY_TEST_PACK="$PWD/content-packs/minnesota-northstar-drainage-ice.crystalpack" \
CRYSTAL_DULUTH_STORY_SAVE_DIR="$PWD/.local/northstar-drainage-ice-review-saves" \
  target/northstar-story/wasm-dev/deps/crystal_bevy-666c15187ce242ec \
  duluth_connections_controller_ --ignored --test-threads=1
```

Use the same executable/environment with `duluth_story_controller_` for the ten
harbor pages. For the four finale controller cases, filter
`boundary_basecamp_story_controller_` and set
`CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK` to that same absolute pack path.
These are explicit local fixtures, followed by production `press` and Continue.
They do not prove earning the initial party, badges, depot or harbor signal.
Asset/component tests use the same Cargo profile and the
`northstar_story_refresh`, `duluth_story`, `duluth_connections` integration tests
and the library filter `boundary_basecamp_q07_`.
