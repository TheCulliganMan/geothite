# River, lake and cave habitats

This phase adds native wildlife to 19 Surf areas and ten cave floors. The
tracked plan is `habitats.json`; `planning/minnesota/world.json` remains the
selection authority. Publication uses the Leech Lake pine
island parent, preserving its required grass, naturalist, rewards, ledge and
Whirlpool cache as well as the Rochester lab and five-stop train service.

The selected 735-map revision is `content-packs/minnesota-water-caves-v05.crystalpack`,
content identity `933857a91d19b2f638e3a1dc592c3eedb786321b73de7c09bb01aa8c75b5f175`,
SHA256 `20fcdad6de43bb2e46a41faa49d7a8916cfe5bd2224680709f135166d3d2801b`.
A fresh Rust publication replay matches it byte for byte.

## Ecology and access

Native Union Cave and Ice Path tables supplied the structure: seven walking
slots, three Surf slots, separate morning/day/night lists and modest rates of
5 or 10. Surf uses native 60/30/10 slot weights; cave floors use
30/30/20/10/5/4/1. Levels in the plan are base levels; existing regional scaling
and legal evolution handling still apply. All species are active Minnesota
originals. Original Johto and Kanto habitats are preserved.

The Mississippi towns use river fish, lake country uses walleye/pike/loon
families, and Superior/intake/mining waters use sculpin, burbot and local
crustaceans. Bemidji, Duluth and Moorhead have burbot Surf encounters only at
night. Limestone caves use drip, bat, snake and tessellated larvae; Iron Range
floors use mineral, ore-haul and drilling families. Voyageurs retains its cold
and wool/ghost ecology. The Fire research starter stays out of ordinary tables.

Three healer floors remain peaceful: Rock Cut, Trout Refuge and Cedar Shelter.
The two refuge springs still support Surf. Gym puzzle water and the separate
Safari expedition use explicit exceptions. Facility dry floors stay quiet.
Twenty-five dry template rooms lose inert original-region fishing headers;
the wet Boundary Intake uses the existing Minnesota shore group. Existing rod
catalogs, thresholds, timed groups and swarm tables are preserved.

Moorhead previously had fenced water with no launch and open unconnected map
edges. Twenty-three guarded metatile replacements close those edges with
native rocks and open a signed two-tile brick bank. The launch is land
`(10,12)` to water `(9,12)`; read the access sign from `(12,15)`. Both R16 road
lanes, people, stories, facades, clinic, notebook reward and return carpets
remain intact. The source town/layout contract records the same opening.

Leech Lake's zero-rate water fallback is explicitly replaced with its lake
fish table. Every other component, including the four-tile pine-island grass
zone, is retained exactly. No active encounter table is overwritten.

## Rust publication

The publisher reads the verified parent, tracked habitat plan and pinned
external native Sweet Scent source. It refuses occupied/nonignored output
paths, stale parents, illegal species/levels, unreachable samples, duplicate
or missing habitats, unexpected headers and unsafe river geometry. Complete
runtime verification and exact written readback precede acceptance.

The native source file belongs to the existing external exporter checkout at
pret commit `3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6` from
`pret-source.lock.json`. Its SHA256 is
`0031b07bd338e17d630ca5ada459df53f0c9452ec7bea82e8649f492c128b0c6`.
Rust imports only nine missing native field definitions and one global root;
all previous definitions remain exact. No source command dump, PCM or pack is
added to tracked files.

The executed publication uses a fresh ignored destination:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 \
RUST_MIN_STACK=67108864 cargo build --locked -p crystal-assets \
  --bin pack_minnesota_water_caves --example publish_pokedex_locations
RUST_MIN_STACK=67108864 target/verdigrim-scene/debug/pack_minnesota_water_caves \
  content-packs/minnesota-leech-island-v05.crystalpack \
  planning/minnesota/pokemon/acquisition-v01/water-caves-v01/habitats.json \
  /Users/ryanculligan/.local/share/geothite/export-workspace/vendor/pokecrystal/engine/events/sweet_scent.asm \
  content-packs/minnesota-water-caves-v05.crystalpack
```

Serialize Cargo through `.local/minnesota-cargo.slot.lock` as in the island
runbook. Check disk space before substantial builds. Use a fresh output name
to repeat publication; completed packs and review evidence stay ignored.

## Runtime and verification

A valid Surf-only map has no walking table. The shared loader now passes an
absent current surface component to the native zero-rate path, preserving its
random-call behavior. Positive-rate missing tables still fail validation.
This fixes long dry walks through water-only refuges/facilities without adding
fake walking encounters. Rebuild the current Rust client before using these
packs; older retained clients predate this fix.

Six external-pack controller cases cover all 19 real Surf mounts and captures,
ten cave captures, three night burbot captures, Moorhead's sign/Good Rod/Surf/
blocked boundaries,512 quiet-floor steps and native Sweet Scent before and
after Continue. Captures consume a real ball, update the party and caught Dex,
and 32 paths reopen through Save/Continue. Four original Moorhead cases retain
its stories, services, gifts and paired returns. Separate original-game
controller/battle/MCP and clean-stdio regressions protect the shared loader.

The habitat fixture commands use the resulting pack:

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-water-caves-v05.crystalpack" \
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo test --locked -p crystal-bevy --lib minnesota_habitats::water_caves:: \
  -- --ignored --nocapture --test-threads=1
```

Five inherited island cases also pass on the combined child, including its
actual naturalist victory, grass encounter, rewards, ledge, Whirlpool cache,
full-pocket retry and services. The independent preservation audit retains 708
complete maps, 26 header-only maps and the bounded Moorhead revision. It preserves
all prior active tables, island zones, native programs, assets and audio; only
the explicitly declared zero-rate lake fallback is replaced. The regional
policy check covers 4,599 species references and 322 trainers. All 377 audio programs pass
PCM hash, frame-count and loop-range verification. Reference/source/native 2D views are
retained under `planning/minnesota/renders/water-cave-habitats-v01/` and reviewed
offscreen. Initial party, badge, location and capacity fixtures are declared
setup; the subsequent inputs prove local gameplay, not earned statewide access.

## Pokédex and remaining work

The Rust web publisher uses this same completed pack to refresh locations,
time/level details and habitat search on all 225 active pages. Original species
facts, art, cries and move previews stay intact. Leech Lake's island grass and
new Surf fish are listed separately. Quiet zero-rate fallback data is omitted.
Nine species remain explicitly pending; the 216-species structural upper bound
is not an earned collection claim. The website source identity is separate
from its existing art/audio catalog identity.

This phase does not finish the statewide goal. Existing grass and rod ecology
still need regional review; Route 2 has inaccessible decorative water edges.
Native Sweet Scent on zone-specific grass also needs its own resolution review.
Nine rare acquisition routes and Verdigrim's compiled roster/completion
threshold still need integration. Unselected story review packs are not active
content. The web guide must continue to track verified selected updates.

Owned Rust formatting and assets/Bevy all-target Clippy pass, with existing
unrelated warnings. The full workspace format check still reports unrelated
differences beginning in `crates/offset-allocator`.
The shared Cargo cache is in active use by other reviews and is preserved.

The live guide is release `20261031-water-caves-28-v06`. All 2,203 remote files
match the checked local release. Full app/location regressions and live
desktop 1280px/mobile 390px checks pass expanded locations, habitat searches,
layout and browser errors. New bank/island rows and native captures were
visually inspected. The static deploy preserves the existing game image,
environment, ports, networks and mounts. A separate current-client check
passes 88 production stdio calls with caught Dex, movement, menus, sign/input
ownership and byte-empty stderr.
