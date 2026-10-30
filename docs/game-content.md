# Build and run game content

Geothite's Rust builders generate the game content from the source and toolchain
pinned in `pret-source.lock.json`. The initialization script assembles that source,
exports runtime catalogs, compiles and verifies the packs, and publishes local
agent inputs. Source checkouts, ROMs, intermediate exports, PCM, and packs stay
outside tracked source.

## State Fair building repair (2026-10-29)

The [fair runbook](../planning/minnesota/playable/state-fairgrounds.md#contiguous-building-repair-2026-10-29)
records the executed `revise_state_fair_buildings PACK LAYOUT OUTPUT` Rust
publication over the selected730-map regional pack. It fixes15 solid building
cells using three explicit native glyph compositions, preserving all entrances,
walking collision, existing art, actors, encounters and audio. Six actual2D
captures, every room return, Save/Continue, full preservation and all377 audio
programs pass. The repaired local pack remains ignored; no web deployment is
claimed.

## Combined Minnesota browser publication (2026-10-29)

The [live release record](deployment.md#combined-minnesota-release-2026-10-29)
identifies the deployed 730-map/476-species/272-move/602-audio pack and its
preservation checks. It is a child of the reviewed Itasca world with the current dialogue revision, with
completed Verdigrim assets/research and starter signatures. The excavation scene
remains in progress; this is not full encounter acceptance.

The executed Rust composition uses the canonical Pokemon source, its completed
native banks and the reviewed field manifest. Set a fresh ignored prefix on
reruns; existing packs remain immutable.

```sh
stage=content-packs/combined-native-2d-itasca-20261029
source=planning/minnesota/pokemon
native=target/minnesota-pokemon/native
bin=target/northstar-story/debug
"$bin/revise_northstar_story" content-packs/minnesota-current-grass-itasca-v01.crystalpack \
  planning/minnesota/story.json content-packs/combined-itasca-dialogue-20261029.crystalpack
"$bin/pack_minnesota_pokemon" content-packs/combined-itasca-dialogue-20261029.crystalpack \
  "$source" "$native" "$source/species-profiles.json" "$stage-species.crystalpack" --family MN_F145
"$bin/pack_minnesota_cries" "$stage-species.crystalpack" "$source" "$stage-cries.crystalpack"
"$bin/pack_minnesota_overworld" "$stage-cries.crystalpack" "$stage-field.crystalpack" \
  "$source/overworld/20261027-v1/sources/mn_f145/verdigrim.manifest-v3-eye-registration.json"
"$bin/pack_minnesota_dex_roster_sync" "$stage-field.crystalpack" "$source" "$stage-dex.crystalpack"
target/fishing-rods/debug/pack_northwoods_starter_signatures \
  "$stage-dex.crystalpack" "$stage.crystalpack"
"$bin/verify_crystalpack" "$stage.crystalpack" --audio --desktop
```

These are repository Rust publishers; their Cargo package is `crystal-assets`.
The cry and selected overworld publishers preserve the previous programs and
banks. Publication does not move the shared regional authoring pointer or
change existing saves' content identity. Packs and compiled tools remain ignored.

## Minnesota canoe

The [canoe publication runbook](../planning/minnesota/canoe/README.md) records
`pack_canoe BASE OUTPUT`, its verified Rust build, native 2bpp source inputs and
current gameplay acceptance status. It publishes a separate ignored overlay
with a St. Paul canoe-shop handoff, passive key item, paddling sprite banks and
updated shore directions. The candidate uses
`content-packs/minnesota-current-grass-r12-v01.crystalpack` as its parent and
writes `content-packs/minnesota-canoe-v04.crystalpack`. All 377 original audio
programs pass PCM hash, frame-count and loop-range verification.

## Short coast and compact Gunflint

The [verified compact foundation](../planning/minnesota/revisions/superior-gunflint-quality-v01/VERIFICATION.md)
is `content-packs/minnesota-superior-compact-v03.crystalpack` (689 maps).
Use `planning/minnesota/world.json` → `active_local_review_revision` for the
current descendant; later regional work must not be rolled back to this foundation.
The guarded Rust `build_compact_superior` stage consumes the exact earlier
694-map winter pack, removes the straight coastal approach and long inland Neck,
and publishes the reviewed32×32 Gunflint. Its runbook records all input/output
hashes, the executed fresh-output replay command, canonical-source promotion,
thirteen passing controller cases, audio verification and actual headless views.
The native pack and every render remain ignored.

## Earlier Superior coast and inland winter correction

The [publication runbook](../planning/minnesota/revisions/north-shore-ely-geography-v01/VERIFICATION.md)
records the executed three-stage Rust build from the exact supplied
`minnesota-superior-inland-art-v01.crystalpack` parent to the 694-map
`minnesota-superior-inland-winter-v01.crystalpack`. It includes required input
hashes, frozen source contracts, fresh-output commands and the actual connected
render. Native readback, local controller checks, headless visual review,
canonical Rust parity and byte-identical replay are verified. All packs remain
ignored. The older south-quay/gatehouse geography is not this correction.

## Minnesota cry revision

The [cry authoring runbook](../planning/minnesota/pokemon/CRIES.md) records
revision 8: longer developed performances for all active legendary and mythical
Pokémon and the complete pseudo-legendary line. Nine rare adult cries run
2.71–3.48 seconds; the other 213 performances retain exact revision-7 PCM,
including the loon and Velcro refinements. The separate ignored candidate is
`content-packs/minnesota-cries-v8.crystalpack`, built over revision 7. The
[current listening gallery](http://127.0.0.1:8937/crafted-v8/listen/) presents
these voices first, with revision 7 as Previous version and effective Lugia,
Ho-Oh, Tyranitar and Suicune renders from the local pack. The runbook gives
reproducible Rust commands and verification results. Shared synthesis kernels
and published campaign packs remain unchanged.

## Minnesota ice dungeon

The [Voyageurs Ice Cavern runbook](../planning/minnesota/ice-caves/README.md)
records the Rust `pack_voyageurs_ice BASE SOURCE_JSON OUTPUT` publisher. Its
executed inputs are `content-packs/minnesota-cave-quality-v01-09.crystalpack`
and `planning/minnesota/ice-caves/voyageurs-ice.json`; the output is
`content-packs/minnesota-voyageurs-ice-v02.crystalpack`. This optional R20
extension preserves the original Ice Path maps, source art and audio. It adds
two authored floors and a paired dry mainland entrance. Generated packs and
all verification renders remain ignored. See the runbook for exact commands
and current controller/native-capture acceptance status.

## Minnesota machine distribution

The [TM/HM runbook](../planning/minnesota/playable/tmhm.md) records the Rust
`pack_minnesota_tmhm PACK DISTRIBUTION_JSON OUTPUT` publisher. It uses the
completed `content-packs/minnesota-cries.crystalpack` and
`planning/minnesota/tmhm-distribution.json` to stock all 57 machines at the
existing Minneapolis Department Store fifth-floor counter and add 20 one-time
hidden TM caches. HMs cost ¥3,000 and remain reusable; TMs retain their existing
prices. The output is `content-packs/minnesota-all-tmhm.crystalpack`. The
runbook records validation commands and results; generated packs stay ignored.

## Northstar story relay

The current [dialogue overhaul](dialogue-overhaul.md) publishes staged scenes
and rewritten native pages on the accepted regional descendant. Its Rust
revision stage runs last in the current story composer. The earlier text-only
publisher below cannot overwrite a pack containing that revision.

The reviewed dialogue is a final Rust publication stage over the completed
relay/supplies pack. It reads `planning/minnesota/story.json`, rewrites only
existing native text, and preserves executable progression and original audio.
Use the [dialogue revision runbook](../planning/minnesota/playable/northstar-relay.md#dialogue-quality-revision--2026-10-27)
for the executed command and verification scope. Its output is the ignored
`content-packs/minnesota-northstar-dialogue.crystalpack`. Existing saves retain
their original content identity; do not relabel them for the revised pack.


The [Northstar relay runbook](../planning/minnesota/playable/northstar-relay.md)
records the Rust `pack_northstar_relay PACK SCENE_JSON OUTPUT` publisher and
`planning/minnesota/northstar-relay.json` scene contract. Its completed candidate,
`content-packs/minnesota-northstar-relay-v2.crystalpack`, combines the selected
eight-gym story parent with the existing reviewed 474-species/270-move pipeline
and the native Kettlurk remote feed. The publisher requires the installed free
Q07 shelter and KETTLURK's reviewed catalog/assets; it preserves unrelated maps,
services, recovery and all original audio. Controller/stdio Continue, matched
headless source/native views, preservation and all 377 audio checks pass.
The runbook gives exact inputs, executed build commands and actual replay saves.
Earlier campaign saves retain their own pack identity. This candidate now has
its own fresh Winona course and actual saved continuations through four badges,
ICE rescue/evidence, the separate courier manifest and Minneapolis return.
Both evidence scenes pass actual stdio replay. Wind onward and full earned
progression remain open. Generated packs stay ignored.

The same runbook records `pack_northstar_supplies PACK SUPPLIES_JSON OUTPUT`
and `planning/minnesota/northstar-supplies.json`. Its published
`content-packs/minnesota-northstar-supplies.crystalpack` appends the existing
Awakening to St Cloud's native stock. The written identity and full preservation
case pass; all other datasets, runtime files and original audio stay exact.
All 377 audio checks and affected-crate all-target Clippy pass as well.
The new identity has its own fresh two-badge saves; that course fails after four
ICE losses. The user's explicit level-75 story review continues its actual
clinic checkpoint with separate boosted saves. See the runbook for commands
and the distinction between supplied levels and played story progression.
That continued party now passes all eight badges, the separate northern
readings/custody/depot actions, Harbor reveal, coast, chief and relay shutdown.
The review minimum rises to 100 for the final three League matches; native Hall
of Fame, credits and postgame Continue pass. The runbook records the actual
same-pack saves and distinguishes these checks from unboosted balance.

## Minnesota cave reference pass

The [eleven-room cave review](../planning/minnesota/caves/johto-quality-v01/REVIEW.md)
records the explicit layouts, Crystal reference study and verification of
`content-packs/minnesota-cave-quality-v01-09.crystalpack`. All nine route dossiers
retain the same revised rooms. All 68 headless production cameras match native
source art exactly, and all 377 audio programs retain their PCM hashes, frame
counts and loop ranges. Complete cave traversal, original rewards, resident
dialogue and Save/Continue pass on both the stone pack and the additive Voyageurs
ice addition. The completed 683-map pack is
`content-packs/minnesota-voyageurs-ice-v02.crystalpack`. Its two ice floors also
pass all three rewards, forced sliding, both ladders, exterior return and
Save/Continue. All 77 production views match native source art, and all 377
audio checks pass. The final review records preservation checks and the unrelated
workspace formatting/lint failures. Generated packs and renders remain ignored.

## Minnesota city residents

The [interior-life runbook](../planning/minnesota/interior-life/README.md) records
`pack_minnesota_interiors BASE STORIES_JSON OUTPUT` and the explicit
`planning/minnesota/interior-life/stories.json` v4 contract. First run
`pack_northstar_dialogue` over `minnesota-northern-winter-v01.crystalpack` with
`planning/minnesota/story.json`, producing the canonical base. The completed
narrative publication is `content-packs/minnesota-interior-life-v04.crystalpack`:
187 non-gym interiors, 117 new residents, free cabin care and remembered errands.
The publisher rejects overrides of reviewed chapter text as well as unsafe actor
placements and service replacement. Earlier v03 flavor text incorrectly replaced
the Duluth receiver reveal; the current lineage rebuilds from the corrected source.

The [physical interior runbook](../planning/minnesota/interior-life/LAYOUTS.md)
records `pack_minnesota_interior_layouts BASE LAYOUTS_JSON OUTPUT`. The v04 story
parent and `layouts.json` v6 produce `minnesota-interior-layout-v06.crystalpack`,
with 24 room contracts and 25 readable details. Ely then publishes
`minnesota-interiors-ely-v03.crystalpack`; the
[St. Paul repair](../planning/minnesota/towns/st-paul-repair/README.md) applies
its explicit facade/river layout and native contract with
`recompose_authored_city`, producing `minnesota-st-paul-repair-v03.crystalpack`.
The [club continuation](../planning/minnesota/interior-life/CLUBS.md) applies
`club-layouts.json`, producing `minnesota-interior-clubs-v03.crystalpack` with
13 additional contracts and 16 details. Original actors, service programs, stairs,
door indices and river connections remain in place.

The [current composition runbook](../planning/minnesota/interior-life/RAIL-STOP-CORRECTION.md)
records the withdrawal of Red Wing's station by user direction. The preserved room branch,
`content-packs/minnesota-interiors-no-red-wing-v01.crystalpack`, uses the protected
Staples parent and corrects the Detroit Lakes and St. Paul native timetables to
list only the five approved stops. It contains 719 maps, 475 species, 270 moves,
377 audio programs and 217 non-gym interiors. The depot publisher and active
source/registration have been removed; rejected packs/renders remain historical.
The St. Paul building/waterfront repair, Staples' supplies, Split Rock's clinic/PC,
fishing homes, shore gate and Winona depot remain in the protected lineage.
Current visual work is native 2D only.
The 719-map interior branch is now **preserved history, not an approved new
publication parent**: it retains the obsolete coast/Gunflint geometry. Current
interior publishers refuse it. See [the compact rebase](../planning/minnesota/interior-life/COMPACT-REBASE.md) for the protected room work, geometry conflicts
and pending physical entrance integration. No new stale-geometry pack was published.
 Canonical coast/Cedar and separate Ely
geography integration, the Staples ledger repair, five-stop passenger service
and regional room-quality review remain open.
The historical
[city renderer runbook](../planning/minnesota/interior-life/CITY-COAST-FOLLOW-UP.md)
records the subsequent facade/shoreline review. Packs, saves and every render
remain ignored. Use a fresh save for the resulting pack identity.

The [city-life runbook](../planning/minnesota/playable/city-life.md) records the
Rust `pack_minnesota_city_life BASE POPULATION_JSON OUTPUT` publisher and the
explicit `planning/minnesota/city-life.json` population contract. It adds eight
residents per small town and sixteen per major city, including one native trader
per installed town. Separate outputs preserve the compact-city, Ely/League and
State Fair parents. Generated packs and all review renders remain ignored.

## Original Minnesota species packs

The [lynx starter revision](../planning/minnesota/pokemon/sprites/mn-f125/DESIGN.md)
selects Fire → Fire → Fire/Ice and retains Cindore's family as ordinary active
species. Final native refinement11 has four colors in each of its six views.
The approved-family review pack is `content-packs/minnesota-lynx-review.crystalpack`
(260 species, including the original251 and three complete review families).
Its runbook preserves the earlier focused review and its then-current art blocker.
Packs remain ignored.

The current roster has 223 active regional species across 101 families. Its
[Fairy balance ruleset](northwoods-fairy.md) adds the eighteenth type, thirteen
regional retypes, thirteen returning retypes, five Fairy attacks and physical
Poison Jab. Follow that runbook's four-stage Rust pipeline, starting from the
existing local native pack. Current authored Fairy types require the Fairy base
overlay before `pack_minnesota_pokemon`.

The earlier Fairy baseline is `content-packs/northwoods-fairy-v2.crystalpack`: 471 total
species and 270 moves, with the modern physical/special split. The earlier
`northwoods-balanced-v1.crystalpack` and `northwoods-moves-v6.crystalpack` are
historical snapshots of earlier authoring. Keep generated packs ignored.

The current completed [sticky-note restoration](northwoods-sticky-notes.md) is
`content-packs/northwoods-sticky-v1.crystalpack`: 474 species and 270 moves. Its
runbook records the full Fairy-base/species/moves/animation pipeline and verified
native evolution, battle, audio-preservation and web checks.

The [roster audit](../planning/minnesota/pokemon/BALANCE.md) constructs profiles
in memory and exercises real battle and evolution rules. `audit_pokemon_balance
PACK planning/minnesota/pokemon --check-pack` additionally compares the completed
pack with current authoring, including both extension catalogs. This does not
establish earned encounter availability or full campaign pacing; regional
encounter placement remains separate work.

The 266-species aggregate and commands below describe the earlier catalog milestone.

The original [266-species Pokédex](../planning/minnesota/pokemon/README.md)
contains complete entries, stats, evolution methods, individual normal/shiny
palettes and level/TM/HM/tutor/egg move lists. The Rust
[`pack_minnesota_pokemon` publisher](../modpacks/minnesota-pokemon/README.md)
adds the reviewed native sprites and profiles to an existing verified local pack.
It preserves the source pack, its original species, maps, runtime files and audio.

The verified local base is `content-packs/minnesota-geography-review.crystalpack`.
It contains 471 maps and 251 original species. The aggregate species extension is
`content-packs/minnesota-pokemon.crystalpack`, with 517 species. Family outputs
live in `content-packs/minnesota-pokemon-families/`; each includes the original
game and one complete evolution family. These generated files remain ignored.

```sh
python3 -S planning/minnesota/tools/build_minnesota_pokedex.py
CARGO_TARGET_DIR=target/agent-minnesota-species cargo build --locked -p crystal-assets --bin pack_minnesota_pokemon --config 'profile.dev.package.crystal-assets.opt-level=0'
target/agent-minnesota-species/debug/pack_minnesota_pokemon \
  content-packs/minnesota-geography-review.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-pokemon.crystalpack \
  --families-dir content-packs/minnesota-pokemon-families --jobs 4
target/runtime-fps/wasm-dev/verify_crystalpack content-packs/minnesota-pokemon.crystalpack --audio --desktop
```

The supplied-base catalog regression constructs all 266 species at levels 1, 50
and 100, checks their dex/party/Day Care/capture references, exercises every level
evolution, executes actual new-species battle turns and preserves original
audio/runtime bytes. The publisher reads back every output and matches its
identity. The detailed runbook includes the opt-in regression command.

New regional encounter placement and fossil-choice/revival services remain
separate map authoring. Front animation currently uses a valid static frame 0;
menu icons, footprints and cries explicitly reuse existing base references.

## Original Minnesota map tile items

The [State Fairgrounds counter map](../planning/minnesota/playable/state-fairgrounds.md)
uses exact original Crystal counter tiles and separate NPC vendors. Its Rust
`pack_state_fair` publisher adds the fair, three rooms and a guarded St Paul
walking entrance to a completed local pack. The runbook records the candidate
inputs, exact native captures, passing controller/preservation cases and all377
audio checks. Its local fair pack is separate from newer regional integration
lineages.

The current local fair crowd candidate is
`content-packs/minnesota-state-fair-maze-trainers-v7.crystalpack` (255 outdoor
characters). Rebuild with Rust `pack_state_fair` from
`content-packs/minnesota-fair-grass-base-v1.crystalpack` and the checked-in
`planning/minnesota/zones/state-fairgrounds.layout.json` / `state-fairgrounds.json`.
Use a fresh ignored output filename. The fair runbook records exact commands,
movement counts, exhibition behavior and verification status.

The [native tile-item workflow](../planning/minnesota/tiles/README.md) builds
an additive Johto-compatible tileset from original ImageGen PNG art and its
hash-pinned catalog using Rust `pack_minnesota_tiles`. Source crops, integer
offsets, palettes and all four collision quadrants are explicit. Original blocks,
runtime files, maps, species and audio are preserved. New tiles are available to
map authors in a separate ignored pack; active map layouts are not republished
by the art importer. The runbook contains the exact local inputs, build commands,
item IDs and review paths.

## Minnesota story review

### Smaller, tighter major cities

The660-map `content-packs/minnesota-three-cities-tight-final-v02.crystalpack`
publishes hand-authored64×48 Minneapolis, St Paul and Duluth from the543-map
Itasca/quality parent. Source contracts preserve old rooms/story programs and
add117 rooms, indexed doors/stairs, distinct marts and optional local activities.
The Rust `expand_authored_city PACK LAYOUT CONTRACT OUTPUT` command, exact
publication receipts, ignored pack hash, fast source-review commands and separate
native/controller evidence are documented in the
[major-city runbook](../planning/minnesota/playable/major-cities.md).
It is a separate reviewed lineage from the newer544-map North Shore successor
below; no merger of that successor or original-species/art work is implied.

### Grand Marais, native coast gate and Gunflint

The 544-map `content-packs/minnesota-northstar-north-shore-story.crystalpack`
is built from the exact 543-map Itasca story below. It adds the actual
IlexForestAzaleaGate room with paired lane warps, an ordinary two-lane Gunflint
connection and bounded native harbor reefs. Eight badges plus the Duluth
signal permit either outbound checkpoint; both inbound returns and native
care remain free. The optional spare-part/lamp repair replaces the old demo
starter/supply handout and grants no regional progress. Six local controller
checks, two preservation cases, three actual stdio replays, the full-bag keeper
regression and all377 audio checks pass; the production client builds. Both
regional clinics register actual blackout recovery, proved after healing,
Continue and played losses. A complete local Brainerd-to-Harbor-to-Boundary
Waters-and-home course also passes; initial six badges/Mew45/Cut remain setup.
[Frozen identities, Rust build/run commands and current evidence](../planning/minnesota/playable/north-shore-story.md).

### Ely rest camp, native League and postgame Continue

The `554`-map `content-packs/minnesota-northstar-ely-league.crystalpack` builds
from the frozen544-map shore parent above. File SHA-256:
`ab049e907b8712f5ee5f174b474c38fcc52460f272e60336aff17445d62cb6c4`.
It adds Ely's real R13 seam, clinic/PC/shop, authentic Ilex Forest gatehouse
registration, five sequential native trainer battles, a separate Hall of Fame
recording, real ceremony/credits, Ely postgame Continue and rematches.
Five controller cases, two preservation cases, native/source review and all377
audio checks pass. Initial eight badges, survey restoration, Mew100 and medicine
are explicit review fixtures. Fresh Winona progression and the Kettlurk set
piece remain unverified. The original544-map earned finale saves retain their
own identity; they are never relabeled for the new pack.
[Exact source contracts, Rust build/run, captures and stdio replay](../planning/minnesota/playable/ely-league.md).

### Earned chief, receiver and reopened League trail

The same544-map shore pack additionally passes the [earned northern
finale](../planning/minnesota/playable/northstar-finale.md): Continue from its
actually earned Boundary arrival, the real chief battle, separate receiver
restoration and complete R13 traversal. Two actual stdio replays pass. The
chief win alone cannot open the League gate. This is not an Ely publication
or a completed fresh-start campaign; those remain open.

### Itasca gatehouse, independent reading and Bemidji report

The 543-map `content-packs/minnesota-northstar-itasca-story.crystalpack` is built
from the shore parent below. SHA-256:
`75e31a904065d00275b4ac932d38c77346485112e856ca5222d29b50055f8fc1`.
It adds the actual IlexForestAzaleaGate room with two side-carpet lanes and
safe returns, the dry headwaters plaque, the separately acknowledged officer
comparison, Lake-only R14 and Pine/report-gated R09. Early visits grant no
reading; reading alone grants no report or Iron Range access. Inbound care
remains free. Five controller/two preservation cases, three actual stdio replays,
four exact exterior views and all 377 audio checks pass. Explicit prior badges,
partner and RNG are setup; the full earned campaign remains unfinished.
[Source, exact frozen input, Rust build/run, saves and replay commands](../planning/minnesota/playable/itasca-story.md).

The unchanged Itasca pack additionally passes the [continuous Pine-to-harbor
story course](../planning/minnesota/playable/northstar-iron-course.md): actual
Pine/report, mine victory and separate recovery, depot handover, Harbor and all
ten reveal pages. A real wild loss preserves earned custody through native care
and Continue. Four actual stdio earned-save replays pass. Six prior badges and
Mew45/Cut are setup; this does not prove the full Winona campaign. The frozen
parent has no Grand Marais regional links; the successor above adds them.
The separate Ely/League successor above now implements that destination; the full fresh campaign remains open.

### Leech Lake shore, Bemidji and the Pine retry

The541-map `content-packs/minnesota-northstar-leech-shore.crystalpack`, file
SHA-256 `03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97`,
adds the actual shore town and two cabins to the lake-country pack below.
Native R07/R08 links check Lake only; ranger Surf/report and courier completion
remain independent. Free care, both cabins' carpets, testimony, the closed pier,
Surf launch/coast/dismount and actual recovery at the new care cabin pass.
A continuous local course earns Pine after a played loss, native clinic recovery
and retry with an existing Fire move. Six controller/two preservation cases,
three actual stdio replays, source/native views and all377 audio checks pass.
Prior badges, review partner and deterministic RNG are setup. This parent
precedes the Itasca successor above; the full earned campaign remains open.
[Exact source, Rust build/run and replay commands](../planning/minnesota/playable/leech-shore.md).

### St Cloud, Brainerd and the ranger report

The538-map `content-packs/minnesota-northstar-lake-country.crystalpack`, file
SHA-256 `da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50`,
extends the courier pack below. Minneapolis's north checkpoint checks Current
and Capitol; St Cloud's checks Wind. Native R05/R06 crossings retain free inbound
travel and permit postponing the courier. The existing ranger's canonical two
report pages precede the successful once-only Surf gift and saved report flags.
Five local controller/two preservation cases, actual stdio road/ranger replays,
four exact native cameras and all377 PCM/frame/loop checks pass. Prior badges
and strong partner are explicit setup. R07/R15/R18 remain visibly closed until
their actual connections are published; the full earned campaign remains open.
[Exact Rust build/run and review commands](../planning/minnesota/playable/lake-country-story.md).

### St Paul courier, separate manifest and R04

The538-map `content-packs/minnesota-northstar-courier-story.crystalpack`, file
SHA-256 `37a569f7cd6cb46bca8c3d34d5deb096b108f8c2db6db38b07408e6add4d64c0`,
adds the native Current-only Minneapolis/R04 crossing and binds Union Depot's
existing courier. Current plus the ICE dispatch note permits his real
Magnemite23/Drowzee22 battle. Victory and separately acknowledging three
DEEP BORE manifest pages have distinct saved flags; neither grants a badge or
later road permission. Both refusal lanes, free inbound care, missing-note
refusal, gym access without courier completion, played loss and a continuous
local win/read/return pass. Two preservation cases, actual stdio, source/native
views and all377 PCM/frame/loop checks pass. Review party/prior badges/dispatch
are setup; earned regional progression and rail remain unfinished. This chapter
follows the progression contract: dispatch is a courier prerequisite, while
Current alone opens R04. The corrected fourth-gym v2 candidate below also follows the Current-only
road contract.
[Exact sources, Rust commands and replays](../planning/minnesota/playable/courier-story.md).

### Fourth-gym river approach

`content-packs/minnesota-gym-four-approach.crystalpack` extends the River Valley
candidate below and retains538 maps/all eight gyms. The new reciprocal
Minneapolis/R04 mouth reaches St Paul's existing facade. Both lanes require
the Current Badge alone, with free inbound return. Dispatch remains a separate
courier prerequisite.
Two preservation/rejection cases, three controller cases and actual stdio
crossing/facade/Start/return pass. A separate continuous course actually earns
the rescue/record, badges3/4 through their puzzles/trainers, restores spent PP
at the clinic and walks home; its initial level45 partner and two prior badges
are explicit setup. Both native views match source pixels and all377 audio
checks pass. The courier, north departure and complete earned journey remain
unfinished. [Exact identities, Rust build and evidence](../planning/minnesota/playable/gym-four-approach.md).

### River Valley city links and consecutive gyms

`content-packs/minnesota-northstar-river-story.crystalpack` extends the Prairie
Road chapter below, retaining538 maps and all eight current gyms. Its two native
R03 mouths link Mankato and Minneapolis; leaving Mankato north requires the
second badge, while inbound return is free. Six visible south-bank caps close
the partial candidate's stray floor exits. Two preservation/rejection cases
and five production-controller cases pass, including earning badges2/3 through
their puzzles/trainers and walking the complete route both ways. That continuous
local course starts with the first badge and a strong legal party; it does not
prove ordinary-party balance or a fresh earned regional journey. Matched native
city frames and all377 audio checks pass. The second continuous fixture walks
to the ICE battle, separately releases the captive and reads the dispatch, then
returns without badge3. Actual stdio road/office return and rescue/record checks
pass with byte-empty stderr. The earlier `river-cities` filename retains the
uncapped v1 candidate for historical comparison.
[Exact input, Rust publication command, coordinates and evidence](../planning/minnesota/playable/river-cities.md).

### Prairie story and the copied-reading notebook

`content-packs/minnesota-northstar-prairie-story.crystalpack` contains538 maps,
file SHA-256 `f7efd72c181143eef61834343c5a2b9fd2664a0d8f14cb250ed3721f8259ed8c`.
It binds native Rochester/R02/Mankato walking seams on a frozen current regional
gym input. Care alone opens departure; inbound return remains free. The garden
paper records optional MS02 after both pages are acknowledged. It awards no
item/badge and adds no Gym2 prerequisite. Two exact preservation/rejection
cases, three local controller cases, actual stdio and all377 PCM/frame/loop
checks pass. Source/native notebook pixels match exactly. Explicit local setup
does not prove earned progression; the strict New Game chapter audit remains
unresolved before R02. [Inputs, executed Rust commands, contracts and limits](../planning/minnesota/playable/prairie-story.md).

### Rochester delivery and R01 connections

The 526-map `content-packs/minnesota-northstar-rochester.crystalpack`, SHA-256
`51d455252b0b876a2a9d48a1b6666d72354f67bdb3cfc20f702e04382c421670`,
adds the city and reciprocal R01/Winona seams to the opening input below.
A New Game journey earns the starter/readings, walks the original trail,
delivers both acknowledged pages at the clinic and returns home. Both lanes
deny departure without a starter. Canonical Gym1 is explicitly refreshed and
bound to the city; its battles, badge, one TM, returns and Continue pass a
separate legal-partner combat fixture. One full New Game Cyndaquil/real-capture
journey now earns the first badge and TM and passes stdio Continue. A later
live-RNG run loses at the leader, so balance/retry coverage remains open alongside
an earned R02/Mankato journey. Two exact-data/uncached/rejection cases, actual stdio opening and
delivery, all 377 audio checks and shared controller/event regressions pass.
The core matcher recognizes native `SCENE_ALWAYS`/`-1`, required by the
serialized checkpoint. [Commands, evidence and limits](../planning/minnesota/playable/rochester.md).

### Winona New Game opening

The 521-map `content-packs/minnesota-northstar-winona.crystalpack`, SHA-256
`2f8c74a72a51affc94fc416763505fca5e8842bbb52b17f5662ee212e40eb2da`,
adds six Winona maps to the exact 515-map integrated input below. Native New
Game starts upstairs; actual stairs lead to the guardian's three-page scene.
All three level 5 starter choices, nickname choice/grid, acknowledged original
readings, five balls once and Continue pass from New Game. A separate full-
pocket fixture proves refusal/retry without losing or duplicating the reward.
Actual stdio MCP and all 377 audio checks pass. Original maps/art/audio stay
exact; HOME intentionally becomes Winona, with original house recovery retained
under an addon identity. R01 remains unbound and the regional journey is
unfinished. [Exact commands, contracts, limits and evidence](../planning/minnesota/playable/winona.md).

### Integrated story, Minneapolis and Ryan's House

The verified 515-map story candidate is
`content-packs/minnesota-northstar-city-story.crystalpack`, SHA-256
`fce69c6da8ad0d4579ef8aebc6b4922d8d6d2321c87891a572bde7b68150fb7d`.
It combines Minneapolis's ICE battle, separately acknowledged rescue and record
reading, the dispatcher's return home and the canonical Avrin switch gym with
the northern story, revised mine/hollows and Ryan's Baptism River cabin.
Guarded Grand Rapids/Duluth links and the harbor/chief/receiver drainage plot
remain intact. The Rust build first imports the pinned reviewed Gym3 into the
completed northern input, preserving unrelated content and native event numbers;
it then installs the city. It does not silently accept the obsolete gym.

Twenty chapter/travel/cabin controller cases, shared Mom and battle/PP checks,
actual MCP, byte-empty stderr and all 377 audio PCM/frame/loop checks pass on
this exact pack. The current single-gym import/city asset cases pass; the
independent all-eight-room fixture now passes on the current 480-map remake
pack. Its separate 488-map St. Paul successor preserves all four internal pad
pairs, binds all five public facades and verifies the reciprocal R04 crossing;
Mankato remains intact. The 492-map St. Cloud successor adds four facades,
the complete canonical wind course, services and reciprocal R05/R06 crossings;
see [St. Cloud's executed build and fixture scope](../planning/minnesota/playable/st-cloud.md).
Exact native build inputs, commands and local fixture
limits are in [the St. Paul runbook](../planning/minnesota/playable/st-paul.md)
and [the gym circuit](../planning/minnesota/gyms/README.md). These gym/city
candidates are now combined with the 526-map Rochester story successor in the
[538-map eight-gym candidate](../planning/minnesota/playable/eight-gyms.md), which
passes every actual city-facade gym course and the existing Rochester chapter
regressions. Existing caught-location IDs stay stable; new cities share explicit
regional areas if the native seven-bit table is full. Minneapolis's
R03/R04/R05 ports remain unbound in that lineage. The southern
opening, remaining regional links, courier, League finish and Kettlurk encounter
still need implementation; local fixtures do not prove the full earned journey.

See [the exact integrated inputs, build commands and playable saves](../planning/minnesota/playable/minneapolis.md),
[the cabin's original native proof](../planning/minnesota/playable/ryans-house.md)
and [the retained northern map lineage](../planning/minnesota/playable/mine-and-hollows.md).
Generated packs, saves and every render generation remain ignored. Earlier
records below retain their original pack/source scope.

The [Rochester two-switch revision](../planning/minnesota/gyms/rochester-gates.md)
publishes `content-packs/minnesota-rochester-gates.crystalpack` through Rust
`revise_rochester_gates`, using the former readable-maze pack as its parent.
The new7×7 room requires the left training bay, upper switch and a return to
swap to the right lane. Its literal controller course, native closed/open
views, both asset cases and all377 audio checks pass. Other maps and battles
stay exact. The runbook records exact inputs/commands, current town bindings
and the remaining lineage/design review scope.
The [Mankato terrace revision](../planning/minnesota/gyms/mankato-terraces.md)
publishes `content-packs/minnesota-mankato-terraces.crystalpack` with Rust
`revise_mankato_terraces` over that Rochester parent. It preserves all indexed
city doors and battles while replacing the floor/ledge/stair presentation.
The shared room rebuild retains original trainer numbers and source audio;
Rochester's identity regression, Mankato's ordinary-party literal course and
native source comparisons pass. The runbook gives exact executed commands.

The [Minneapolis staged circuit](../planning/minnesota/gyms/minneapolis-circuit.md)
publishes `content-packs/minnesota-minneapolis-circuit.crystalpack` through Rust
`revise_minneapolis_circuit` over that Mankato parent. Lenvi powers the first
switch; Jossa stands beyond its visible electric barrier and powers the second.
The ordinary-party literal route, reset/Continue, rewards and city exits pass.
Both native barrier-state cameras match the source at every pixel; audio's377
PCM hash/frame/loop checks pass. Other maps and trainer rosters remain exact.

The [St. Paul pad loop](../planning/minnesota/gyms/st-paul-loop.md) publishes
`content-packs/minnesota-st-paul-loop.crystalpack` through Rust
`revise_st_paul_loop` over that Minneapolis parent. Four pairs connect entrance,
Belva, Orven, leader landing and a direct return. The ordinary-party literal
course verifies all eight endpoints, trainer/leader wins, Continue and city
exits; loss, reverse order and missing badge permission checks pass. Eight
revision asset cases and all377 audio checks pass. Existing trainer rosters,
city-door indices, other maps and the first three revisions stay exact.

The [St. Cloud arrow revision](../planning/minnesota/gyms/st-cloud-arrows.md)
publishes `content-packs/minnesota-st-cloud-arrows.crystalpack` through Rust
`revise_st_cloud_arrows` over that St. Paul parent. It paints the native arrow
font pixels on the existing wooden lanes, matching their native forced direction.
Every collision, battle and door binding stays exact. The ordinary-party literal
course, loss and missing-badge cases, ten revision asset cases, source/native
pixel checks and all377 audio checks pass. It preserves the first four revisions.

The [Brainerd dock control](../planning/minnesota/gyms/brainerd-winch.md) publishes
`content-packs/minnesota-brainerd-winch.crystalpack` through Rust
`revise_brainerd_winch` over that St. Cloud parent. A visible blue control
replaces the right statue and toggles center/left piers without a choice menu.
The ordinary-team literal course, loss/badge/poolside denials, twelve asset
cases, all377 audio checks and three exact native camera comparisons pass.
Battles, gate conditions, city bindings and the first five revisions stay exact.

The [Bemidji garden revision](../planning/minnesota/gyms/bemidji-return.md) publishes
`content-packs/minnesota-bemidji-return.crystalpack` through Rust
`revise_bemidji_return` over that Brainerd parent. Two native Cut hedges retain
their original required trainers; a gold gate opens after Orlis for an18-step
return. The ordinary-team literal course, four existing Cut/controller cases,
fourteen revision asset cases, all377 audio checks and two exact native camera
comparisons pass. The first six revisions, parties, Cut replacements and indexed
city doors remain intact. Later regional lineages still need integration.

The [Duluth ice revision](../planning/minnesota/gyms/duluth-gold-gates.md) publishes
`content-packs/minnesota-duluth-gold-gates.crystalpack` through Rust
`revise_duluth_gates` over that Bemidji parent. Four gold gate recipes distinguish
trainer-operated openings from fixed silver stops; every collision quadrant,
slide route, battle and indexed city door stays intact. Four existing controller
cases, sixteen revision asset cases, all377 audio checks and five exact native
camera comparisons pass. A fixed four-partner level44 route also earns all
three trainers and Norek through native slides, switches, Pack healing,
Continue and both city carpets. The cumulative541-map candidate retains all eight
revisions. The regional integration below supersedes that separate room branch.

The [current regional gym publisher](../planning/minnesota/gyms/current-regional.md)
now reads `planning/minnesota/gyms/current.json` through Rust and recompiles all
eight selected rooms inside the frozen Ely story and compact Twin Cities parents.
The results are `content-packs/minnesota-current-gyms-story.crystalpack` (554 maps)
and `content-packs/minnesota-current-gyms-cities.crystalpack` (660 maps).
All four source/preservation cases, both branches' actual city doors/paired
carpets/Continue, sixteen exact native arrival comparisons and both sets of
377 audio checks pass. The ordinary-team course replay is recorded separately
in the runbook; this does not claim a whole earned campaign.


### Earlier Duluth harbor reveal

The retained earlier local story candidate is
`content-packs/minnesota-northstar-harbor.crystalpack`, SHA-256
`c0526b238a9c2a8da74c69e22635b69ad9a0b27b2a60af154a5afb644acccacf`.
It retains the 502-map Grand Rapids candidate below and binds MS10's three
canonical text bodies to the existing harbor receiver. Its nine pages identify
Kettlurk and establish that Team Lode continued despite its own warnings.
The original eight-badge/depot gate and acknowledgment-before-signal script
remain exact. No prerequisite, item or other reward is supplied by this overlay.

Executed build, publication and verification:

```sh
CARGO_TARGET_DIR=target/northstar-story cargo test --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  -p crystal-assets --test duluth_story --no-run
CRYSTAL_DULUTH_STORY_BASE_PACK="$PWD/content-packs/minnesota-northstar-midgame-cities.crystalpack" \
  target/northstar-story/wasm-dev/deps/duluth_story-2f4f4d8c4b3dda01 \
  --ignored --test-threads=1
target/northstar-story/wasm-dev/pack_duluth_story \
  content-packs/minnesota-northstar-midgame-cities.crystalpack \
  planning/minnesota/story.json content-packs/minnesota-northstar-harbor.crystalpack
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-harbor.crystalpack --audio --desktop
CARGO_TARGET_DIR=target/northstar-story cargo test --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  -p crystal-bevy --lib duluth_story_controller_ --no-run
RUST_MIN_STACK=33554432 \
CRYSTAL_DULUTH_STORY_TEST_PACK="$PWD/content-packs/minnesota-northstar-harbor.crystalpack" \
CRYSTAL_DULUTH_STORY_SAVE_DIR="$PWD/.local/northstar-harbor-review-saves" \
  target/northstar-story/wasm-dev/deps/crystal_bevy-666c15187ce242ec \
  duluth_story_controller_ --ignored --test-threads=1
RUST_MIN_STACK=33554432 \
CRYSTAL_DULUTH_CITY_CONTROLLER_PACK="$PWD/content-packs/minnesota-northstar-harbor.crystalpack" \
  target/northstar-story/wasm-dev/deps/crystal_bevy-666c15187ce242ec \
  duluth_city_controller_q07_ --ignored --test-threads=1
CARGO_TARGET_DIR=target/northstar-story cargo build --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' -p geothite
```

All five targeted asset/controller cases pass, including every missing badge
and missing depot, all nine visible pages, movement ownership, final-page signal,
repeat and Continue. All 377 audio programs pass PCM/hash/frame/loop verification.
Compiled executable hashes in filenames can change after dependency edits;
the emitted paths above identify the actual checked build. At publication, source metadata and these three canonical text bodies agreed.
The later ten-page drainage/ICE revision is documented above.

The actual fullscreen client was run with the retained pre-reading fixture:

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-harbor.crystalpack \
  --load .local/northstar-harbor-review-saves/ms10-ready-82849-1793063368002897000.crystalsave
```

Press D to face the technician, then Z to read. Z advances each of nine pages.
The same command with `mcp` instead of `play` starts the checked stdio server;
its real JSON-RPC inputs read the reveal/repeat, open START and move the player.
Both actual clients keep stderr empty. This local fixture supplies initial
position, a legal partner, eight badges and upstream clearance; it does not
prove earning them or regional travel. Duluth's R10/R11 links, earlier earned
chapters, League completion and Kettlurk's encounter remain open. Broad lint
and formatting limitations remain those recorded in the Grand Rapids section.

### Grand Rapids and the repaired city composition

The current candidate is
`content-packs/minnesota-northstar-midgame-cities.crystalpack`: 502 maps.
Its frozen input is `content-packs/minnesota-northstar-cities-base.crystalpack`,
a copy of the repaired 498-map city pack with SHA-256
`37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6`.
The [city runbook](../planning/minnesota/playable/three-cities.md) records that
input's captured gym/town epoch. The overlay preserves it and adds Grand
Rapids, its native evidence handover and guarded reciprocal R09/R10 links.

The current text client and offscreen capture example were built with this
executed command. The per-package overrides apply only to the local
verification build; `location-tester` is required by the capture example:

```sh
CARGO_TARGET_DIR=target/northstar-story cargo build --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  -p crystal-bevy -p geothite --features crystal-bevy/location-tester \
  --example render_at_location --bin geothite
target/northstar-story/wasm-dev/pack_grand_rapids \
  content-packs/minnesota-northstar-cities-base.crystalpack \
  planning/minnesota/towns/MnGrandRapids.json \
  planning/minnesota/towns/MnGrandRapids.layout.json planning/minnesota/story.json \
  content-packs/minnesota-northstar-midgame-cities.crystalpack --connect
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-midgame-cities.crystalpack --audio --desktop
```

The publisher was built by the `crystal-assets --test grand_rapids` Cargo
invocation. Final replays used the emitted executables below, so the final
JSON inputs and final city-based pack were checked without changing the
compiled controller epoch:

```sh
CRYSTAL_GRAND_RAPIDS_BASE_PACK="$PWD/content-packs/minnesota-northstar-cities-base.crystalpack" \
  target/northstar-story/wasm-dev/deps/grand_rapids-eb8f20e7ce56305c \
  --ignored --test-threads=1
RUST_MIN_STACK=33554432 \
CRYSTAL_GRAND_RAPIDS_TEST_PACK="$PWD/content-packs/minnesota-northstar-midgame-cities.crystalpack" \
CRYSTAL_GRAND_RAPIDS_SAVE_DIR="$PWD/.local/northstar-midgame-cities-review-saves" \
  target/northstar-story/wasm-dev/deps/crystal_bevy-666c15187ce242ec \
  grand_rapids_controller_ --ignored --test-threads=1
RUST_MIN_STACK=33554432 \
CRYSTAL_IRON_RANGE_STORY_TEST_PACK="$PWD/content-packs/minnesota-northstar-midgame-cities.crystalpack" \
CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$PWD/content-packs/minnesota-northstar-midgame-cities.crystalpack" \
  target/northstar-story/wasm-dev/deps/crystal_bevy-666c15187ce242ec \
  story_controller_ --ignored --test-threads=1
```

All four asset, six Grand Rapids controller and seven earlier story cases
pass. All 377 audio comparisons pass. Fixture party, local placement, badges
and upstream flags are explicit setup; battle outcomes, pickups, the handover,
guarded crossings, services and Continue then use production input. This is
not a verified earned Winona-to-Ely journey. The Duluth walking/story link,
earlier chapters and Kettlurk's encounter remain open.

The actual retained before-handover review is runnable from this checkout.
Press Up then Z to speak to the technician; Enter opens Start.

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-midgame-cities.crystalpack \
  --load .local/northstar-midgame-cities-review-saves/ms09-declined-73695-1793062236643993000.crystalsave
```

The real MCP client also loads the earned handover save, reads the repeat
direction and visible archive shelf, opens START/PACK and moves the
authoritative player. Keyboard/alternate-screen play opens the real ITEMS
pocket and exits normally. Both keep stderr byte-empty.

The native exterior/checkpoint views and office use original source pixels;
the office reference uses the native indoor palette. Every rejected pass,
original, conversion, crop and native readback is retained in
`planning/minnesota/renders/grand-rapids/`. The source review command is:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_grand_rapids.py \
  --pack content-packs/minnesota-northstar-midgame-cities.crystalpack \
  --connected --verify-published
```

Scoped Rust formatting passes. Workspace formatting retains the recorded
existing differences; strict affected-package all-target Clippy stops at
eleven existing `never_loop` errors in unchanged source files. These broad
checks are not reported as passing.

### Earlier mine-only review (retained)

The earlier `content-packs/minnesota-northstar-story.crystalpack` extends the
completed `content-packs/minnesota-prairie-review.crystalpack`. The source must
contain the authored R09 mine, Grand Marais and native Boundary Waters finale.
The Rust publisher adds the foreman/cache and refreshes existing dialogue;
it does not invent replacement rooms or grant a player's progression.

The following executed asset command builds the publishers and checks current
source/native preservation. The per-package override belongs to this local
verification build rather than distributable optimization settings:

```sh
CRYSTAL_IRON_RANGE_STORY_BASE_PACK="$PWD/content-packs/minnesota-prairie-review.crystalpack" \
CARGO_TARGET_DIR=target/northstar-story cargo test --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  -p crystal-assets --test iron_range_story -- --ignored --test-threads=1
target/northstar-story/wasm-dev/pack_iron_range_story \
  content-packs/minnesota-prairie-review.crystalpack \
  planning/minnesota/routes/MN_R09.json planning/minnesota/story.json \
  content-packs/minnesota-northstar-story.crystalpack \
  --refresh-finale --grand-marais-dialogue planning/minnesota/playable/grand-marais.json
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-story.crystalpack --audio --desktop
```

All four asset cases and all 377 audio comparisons pass. The production
controller tests were built and run with:

```sh
RUST_MIN_STACK=33554432 \
CRYSTAL_IRON_RANGE_STORY_TEST_PACK="$PWD/content-packs/minnesota-northstar-story.crystalpack" \
CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$PWD/content-packs/minnesota-northstar-story.crystalpack" \
CARGO_TARGET_DIR=target/northstar-story cargo test --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  -p crystal-bevy --lib story_controller_ -- --ignored --test-threads=1
```

The emitted `crystal_bevy-8ddf5a560e93701b` executable was also replayed directly
against the final prairie-based candidate: all seven story cases and both R09
controller cases pass. The larger stack is a test-thread setting for this
verification profile. Badges, upstream events, local position and legal
partners are explicitly initialized; battles and recovery then use production
inputs. This does not establish an earned eight-badge Minnesota journey.

The production text client also passes stdio MCP and keyboard/alternate-screen
checks on the earned recovery save, with byte-empty stderr. The actual local
review can be opened with:

```sh
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-northstar-story.crystalpack \
  --load .local/northstar-story-review-saves/q06-meter-recovered-38137-1793057500549669000.crystalsave
```

Packs, saves, captures and build products remain ignored. See
[Minnesota status](../planning/minnesota/STATUS.md) for the exact output hash,
the retained native/source image comparison and remaining chapter work.
Grand Rapids handover and Duluth activation are not implemented by the mine
binding. Workspace formatting and strict Clippy have the recorded pre-existing
failures; those broad checks are not passing.

## Desktop app packaging

The packaging commands consume the completed local native pack; they do not
re-export content or rebuild packs. The default is
`content-packs/core-modular.crystalpack`. Packs are copied only into ignored local
distribution output, never embedded into Rust source or tracked in Git.

```sh
just desktop-content     # Once: prepare external content, if missing
just desktop-build       # Compile and validate; no window or installation
just desktop             # Compile, validate and install for the current OS
just desktop-run         # Launch the installed copy without rebuilding
just desktop-package     # Compile and produce the native distributable
just desktop-repack      # Repackage the last successful build, no Cargo
just desktop-install     # Install the last successful build, no Cargo
just desktop-check       # Revalidate content and artwork, no window
```

The Unix implementation is `python3 tools/desktop.py`, requiring Python 3.9+.
Windows uses `tools/build-desktop.ps1`; its existing `-Pack`, `-InstallDirectory`
and `-Launch` options remain available, with `-Action` and `-SkipBuild` added.
All commands build for the current host OS and architecture. Windows/Linux
runners need native verification before claiming release support there.

Both use persistent `target/desktop-build` and the `release` profile by default,
so normal builds, installs and packages use full game and engine optimization.
`CARGO_TARGET_DIR` selects another cache. For faster development builds, select
`GEOTHITE_DESKTOP_PROFILE=wasm-dev` (or pass the existing profile option). That
profile uses optimization level 1 for game crates and level 2 for engine
dependencies; it trades runtime performance for shorter compilation. Non-release
desktop builds keep an incremental cache in this dedicated target directory;
set `CARGO_INCREMENTAL=0` to disable it when disk space matters more than update
time. Keep the same profile when repackaging or installing with `--skip-build`.
The first toolchain/content/Bevy build is not a two-minute operation. Warm-cache
updates and repackaging avoid that work; the script reports actual elapsed time.
Each packaging run verifies the actual pack's desktop title/intro metadata.
The Rust scenery and people validators also parse the artwork;
all platforms load the staged production client with `--check-installation`
before publishing. This checks resources and performs a temporary write probe at
the save location, including atomic backup suffixes, without opening a window.
A package check alone does not certify gameplay; use the AGENTS.md controller,
battle, MCP and clean-stderr regressions when gameplay changes.

Verified on Apple Silicon on 2026-10-18: `just desktop-package` completed in
132.9 seconds with a warm cache, including incremental game-crate compilation,
Rust content/art validation, signing and DMG creation. `just desktop-install`
completed in 51.8 seconds. Initial compilation took over ten minutes. The DMG
was mounted read-only, its SHA-256 and app signature checked, and its actual
client passed `--check-installation` from `/tmp` with the artwork override unset.
The bundled pack matched the original byte-for-byte. The installed app opened
through Launch Services and reached trainer selection and the opening dialogue.

| Host | Installed app | Distribution under `.local/desktop-dist/` |
| --- | --- | --- |
| macOS | `~/Applications/Geothite.app` | `Geothite-VERSION-macos-ARCH.dmg` and `.sha256` |
| Windows | `%USERPROFILE%\Games\Geothite` and Desktop shortcut | `Geothite-VERSION-windows-ARCH.zip` and `.sha256` |
| Linux | `~/.local/opt/geothite` and application-menu entry | `Geothite-VERSION-linux-ARCH.tar.gz` and `.sha256` |

The macOS app uses `Contents/MacOS/Geothite`, `Contents/Resources` and an
`Info.plist`, with no shell launcher. Resources and artwork resolve relative to
the executable, including when opened from Finder or a mounted, read-only DMG.
Only system dynamic libraries are accepted. Packaging uses Apple's `sips`,
`iconutil`, `codesign`, `otool` and `hdiutil`, without an external bundler dependency.
The DMG has an Applications shortcut for drag-and-drop installation; `just desktop`
installs to the current user's Applications folder without administrator access.
The bundled game starts through the production title flow, not a location preview.

Packaged native clients (`desktop-app`, without `location-tester` or `dev-tools`)
play music and sound effects by default. Native developer clients remain muted,
including release builds of preview tools; `CRYSTAL_PREVIEW_SOUND` no longer
enables them. `CRYSTAL_PREVIEW_MUTE=1` can explicitly silence a packaged client.
Opt-in `CRYSTAL_AUDIO_TRACE=1` records the native sink volume and playback timing.
The Mac window's title-bar Settings button and native Settings menu expose
volume (0–100% in 10% steps), mute, the 2D/3D toggle, a direct checkable First Person
toggle, default/close/wide orbit, camera reset, fullscreen and Reset Settings.
Turning First Person off returns to orbit; enabling it from 2D also enables 3D.
Orbit uses the interpolated trainer footing as its pivot, so clamped indoor
viewports no longer rotate around the room center. These controls edit
presentation resources, while the production controller retains gameplay input.
The master gain scales music and sound effects separately from authored music
fades and fanfare suppression. Developer mute remains authoritative.
Preferences persist atomically in
`~/Library/Application Support/Geothite/settings.json`; camera changes are
debounced, and normal exit flushes pending settings. F3, mouse, controller and
fullscreen changes are mirrored back to both menus. `CRYSTAL_SETTINGS_TRACE=1`
records effective presentation preferences for live checks.
After building, `python3 tools/test_desktop_settings.py` checks preference
restoration, real AppKit target/action dispatch and Metal-layer preservation
using an offscreen window that is never shown.
Normal web play enables sound on its first trusted click, touch or keypress.
Browser previews, `?development=1` and embedded training views keep their master
gain muted while preserving source audio timing. A new `crystal.audio.preference` storage key
records only deliberate user choices; the old forced-mute value is ignored.

Audio-default verification on 2026-10-18 passed all 107 browser regressions and
all 377 PCM hash/frame/loop comparisons in each native and browser pack. The
installed Mac client started both title music and sound effects at sink volume
`1.0`. Its executable matched the rebuilt DMG, whose signature and checksum passed.
The commands used for the pack and browser checks were:

```sh
target/desktop-build/wasm-dev/verify_crystalpack content-packs/core-modular.crystalpack --desktop --audio
target/desktop-build/wasm-dev/verify_crystalpack content-packs/core-modular.browser.crystalpack --desktop --audio
npm run test:browser
```

New desktop saves use `~/Library/Application Support/Geothite/saves` on Mac,
`%LOCALAPPDATA%\Geothite\saves` on Windows, and `$XDG_DATA_HOME/geothite/saves`
(or `~/.local/share/geothite/saves`) on Linux. Usable existing portable saves beside the
pack continue to use that location. Overlong legacy filenames are copied to the
bounded user-data filename without deleting originals or replacing newer saves. Explicit `--save-path` now selects both the
save target and the title menu's Continue lookup. Startup errors from the Mac
GUI show a native alert and are recorded in `~/Library/Logs/Geothite/startup-error.log`.
`--check-installation` suppresses alerts for automated checks.
`just desktop-run` also captures normal launch output in
`~/Library/Logs/Geothite/game.log` on macOS.

Local Mac packages are ad-hoc signed and are not notarized. For public
Gatekeeper distribution, install a Developer ID Application signing identity and
configure an existing `notarytool` keychain profile. Then run:

```sh
GEOTHITE_SIGN_IDENTITY='Developer ID Application: Your configured identity' \
GEOTHITE_NOTARY_PROFILE='your-existing-keychain-profile' just desktop-release
```

That command uploads the signed DMG to Apple's notary service, waits for an
accepted result, then staples and validates its ticket. Normal build/install/
package commands do not upload anything. See
[Apple's notarization documentation](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).
An Intel Mac must build its own native artifact; this checkout's local Apple
Silicon artifact does not establish Intel, Windows or Linux runtime acceptance.

## Initialize once

The single platform-selecting command is `just crystalpack`.

Install Git, Git LFS, and the Rust toolchain selected by `rust-toolchain.toml`.
The build environment also needs Bash, curl, tar, make, a C compiler, and
`sha1sum`/`sha256sum`. Windows uses the `Ubuntu-24.04` WSL distribution by default;
install these tools and Rust inside that distribution, plus Git LFS on Windows.
Native graphical play also needs the platform libraries listed in
[cold-start-3d.md](cold-start-3d.md).

On macOS, install Bison, pkgconf, and libpng with Homebrew. The initializer
builds the pinned RGBDS source outside the checkout, uses Homebrew's Bison
without changing shell configuration, and verifies the assembled ROM's SHA-1.

From the checkout, run on Windows:

```powershell
.\tools\build-crystalpack.ps1
```

On Linux x86_64:

```sh
bash tools/build-crystalpack.sh
```

The same Bash command runs on macOS. Paths and checksum verification do not
require GNU `realpath` or GNU checksum command options.

The external workspace defaults to `$HOME/.local/share/geothite/export-workspace`
inside the Linux build environment. The script selects the exact pret commit and
tree, downloads and checks RGBDS 1.0.4, and checks the assembled ROM identity.
It initializes Git LFS and verifies artwork, then runs the repository's Rust
exporters for game data, title and intro presentation, graphics, animation, and
audio. It generates the manifest and source provenance in the external workspace.

Both browser and TUI audio verification check every program's PCM hash, frame
count, and loop range. Controller tests exercise dialogue ownership, Mom, the
New Bark exit gate, production menus, and a battle turn with PP mutation. MCP
tests exercise the same input controller and require clean stderr. The local
packs are replaced only after these checks and the text client build pass.

The published files are:

| Client | Local pack |
| --- | --- |
| Native game | `content-packs/core-modular.crystalpack` |
| Browser | `content-packs/core-modular.browser.crystalpack` |
| Terminal and stdio MCP | `content-packs/text-tui.crystalpack` |

All three remain ignored by Git. The initializer applies the shared
[All Pokémon, solo](../modpacks/all-pokemon/README.md) mod to both source pack
formats before creating the TUI pack. All 251 species are catchable by default;
Celadon Department Store 4F sells all six evolution stones and a ¥2,100 Link
Cable. Use the cable from PACK on a trade-evolving Pokémon, holding its usual
trade item when required. Successful use consumes the cable and required held
item. This gives native, browser, TUI and MCP clients the same content policy.
The source-only exports in the external workspace retain vanilla tables.

To enhance an existing verified pack without rebuilding source exports:

```sh
cargo run --locked -p crystal-assets --bin pack_all_pokemon -- \
  content-packs/core-modular.crystalpack content-packs/all-pokemon.crystalpack
cargo run --locked -p crystal-assets --bin pack_all_pokemon -- \
  content-packs/core-modular.browser.crystalpack content-packs/all-pokemon.browser.crystalpack
cargo run --locked -p crystal-assets --bin pack_text_tui -- . \
  content-packs/all-pokemon.browser.crystalpack content-packs/all-pokemon.text-tui.crystalpack
```

These commands preserve the supplied pack's audio storage and leave its source
file intact. The additional manifest identities separate modded saves from
older saves; no save migration happens automatically. Rebuild clients from
this checkout for Link Cable held-item behavior.

### Targeted source-text repair on 2026-10-17

Battle Tower selects 120 greeting, win and loss messages from
`data/battle_tower/trainer_text.asm`, outside the usual source text directories.
The Rust exporter includes this file and rejects missing selected messages.
With the pinned source and an already prepared external workspace, the repair
used these `pack_core` stages (built with the `wasm-dev` profile):

```sh
workspace="$HOME/.local/share/geothite/export-workspace"
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_core -- \
  "$workspace" --export-asm-text
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_core -- \
  "$workspace" --pack-prepared-workspace
```

These stages produce vanilla source packs. Apply `pack_all_pokemon` and then
`pack_default_battles` to each format, derive the TUI pack with `pack_text_tui`,
and run the initializer's publication checks before replacing shared packs.
The verified repair added exactly 120 messages to each published pack; all old
texts, other game-data fields, runtime files and embedded audio were identical.
All 377 PCM hash/frame/loop checks passed for each pack. Twelve Tower controls,
early-game controller checks, earned Hall/Continue, MCP movement/menu and
clean-stderr checks passed. The new pack identities require matching saves;
existing saves were not rewritten.

### Additive native field-script repair on 2026-10-26

The pinned `engine/events/field_moves.asm` supplies the native Headbutt/Fly
animation routines and their local branches. The Rust global-script exporter
now retains these original bodies. In an **already prepared external
workspace**, the following stages successfully exported the global scripts and
compiled fresh native/browser source packs:

```sh
field_workspace="$HOME/.local/share/geothite/export-workspace"
target/runtime-fps/wasm-dev/pack_core "$field_workspace" --export-global-scripts
target/runtime-fps/wasm-dev/pack_core "$field_workspace" --pack-prepared-workspace
```

These are the built Rust tools from this checkout's warm `wasm-dev` target.
The first stage refreshes the external global-script catalogs; the second
compiles that workspace's existing prepared inputs into
`$field_workspace/content-packs/core-modular{,.browser}.crystalpack`. This is
targeted source creation, with the full initializer and its playable-core
publication checks remaining a separate workflow. The fresh external packs
retain vanilla source tables; existing modded packs remain the gameplay base.

The additive repair takes **BASE SOURCE OUTPUT**. The executed 450-map Boundary
Waters example used the existing local Minnesota pack as BASE and the fresh
external browser source pack as SOURCE:

```sh
target/runtime-fps/wasm-dev/pack_native_field_scripts \
  content-packs/minnesota-boundary-waters.crystalpack \
  "$field_workspace/content-packs/core-modular.browser.crystalpack" \
  content-packs/minnesota-boundary-waters-fields.crystalpack
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-boundary-waters-fields.crystalpack --audio --desktop
```

The builder verifies both inputs, requires equality for every overlapping
OverworldEvents body, and adds only the 40 missing native field definitions to
the **existing** catalog. All 627 original bodies are preserved. Existing global
root order is preserved, with Headbutt entries and `Script_UsedWaterfall` retained
when absent. Native source certificates and the compiled Waterfall object,
runtime and control-command slots are checked before publishing a distinct
manifest/identity. Reapplying the repair is idempotent. Maps, art, encounters,
items, event constants, HOME/recovery, runtime files and audio remain unchanged;
BASE and SOURCE files remain intact. Outputs stay ignored under `content-packs/`.
The canonical native/browser/TUI packs are separate publication targets of the
full initializer, rather than destinations for these fresh source-only packs.

Executed evidence: the pinned-source animation-closure/timing regression and all
four repair tests passed, including native Headbutt `callasm` execution, complete
Waterfall-loop retention, preservation/idempotence and valid-schema overlapping
source/local-tail rejection. The published Boundary repair passed all **377**
audio PCM hashes, frame counts and loop ranges. These source/component results
do not establish menu-origin Headbutt fallthrough or a whole playable-core
rebuild; field input, animations and regional gameplay require their production
controller regressions. Source export/pack logs and the repair results are the
ignored `.local/field-move-source-{export,pack}.log` and
`.local/native-field-repair-assets.log` from this run.

### Solo-completion pack verification on 2026-10-16

The shared Rust overlay builder generated native, browser and text-client packs
from the existing verified local content. The native pack passed production
presentation validation. The TUI pack passed all 377 audio PCM hashes, frame
counts and loop ranges. The real-pack item tests passed purchases, all ten solo
trade evolutions, failed-use inventory preservation, 251-species encounter
coverage and round-trip preservation of unrelated content and audio.

This run reused content and build caches. Pack tools used
`target/solo-completion` with the `wasm-dev` profile and an assets optimization
override of 0. Client regressions use the standard optimized `wasm-dev` profile.
The standard controller suite passed all seven renderer-neutral regressions,
including cable use with a complete evolution confirmation, Mom/New Bark,
battle command menus and PP mutation, department-store clerks, running shoes,
Research Center and trainer rematches/save. The native earned Hall-of-Fame
ceremony, save and Continue regression also passed. MCP movement/Start-menu
and byte-empty stderr checks passed. The production TUI was launched with
`target/solo-completion/wasm-dev/geothite play content-packs/all-pokemon.text-tui.crystalpack`;
Enter opened its production Start menu and Ctrl-C exited cleanly.

The three canonical files in `content-packs/` were then atomically replaced
with the verified native, browser and TUI candidates. Byte comparisons matched
all three published files to those tested candidates. The independent
`all-pokemon.*.crystalpack` files remain available for explicit selection; all
packs stay ignored by Git. Existing saves retain their original identities.

The executed pack commands were:

```sh
target/solo-completion/wasm-dev/pack_all_pokemon \
  content-packs/core-modular.browser.crystalpack content-packs/all-pokemon.browser.crystalpack
target/solo-completion/wasm-dev/pack_text_tui . \
  content-packs/all-pokemon.browser.crystalpack content-packs/all-pokemon.text-tui.crystalpack
target/solo-completion/wasm-dev/pack_all_pokemon \
  content-packs/core-modular.crystalpack content-packs/all-pokemon.crystalpack
target/solo-completion/wasm-dev/verify_crystalpack \
  content-packs/all-pokemon.crystalpack --desktop
target/solo-completion/wasm-dev/verify_crystalpack \
  content-packs/all-pokemon.text-tui.crystalpack --audio --desktop
```

The independent radio oracle tests use external fixtures at runtime rather than
embedding files from another checkout into the test executable. To run those
opt-in comparisons, set `CRYSTAL_ORACLE_FIXTURE_ROOT` to their external fixture
directory and `CRYSTAL_RENDER_TEST_PACK` to a completed pack, then use
`cargo test --locked -p crystal-assets --lib radio_ -- --ignored`. Missing
configured inputs fail those tests; normal test compilation needs no sibling
repository or game-content fixtures.

Independent PC and Mailbox LCD references can be captured from the completed
external ROM and symbols with the optional [menu QA tool](menu-oracle.md).
Those tests use `GEOTHITE_ORACLE_FIXTURES`, separately from the radio fixture
setting. The emulator environment and all generated reference content remain
external; no pack, audio exporter, or shipped dependency changes are required.

### Fresh-source verification on 2026-10-13

Two independent fresh remote checkouts at `a55d3ea`, each with empty source,
export, Cargo registry and build caches, completed the default `just crystalpack`
command with exit code 0 and no prompts on macOS. Both initialized the pinned
pret source and RGBDS 1.0.4 and assembled the reference ROM successfully.

Each run certified the continuous production intro against all 28 pinned ROM
scenes, verified all 377 audio programs' PCM hashes, frame counts and loop
ranges, and passed the battle, Mom/New Bark, MCP movement/Start-menu and
byte-empty stderr regressions. All three published pack formats matched
byte-for-byte between both runs and the shared checkout. Packs remain ignored;
verification logs are external to the repository.

### Source rebuild verification on 2026-10-14

The follow-up source rebuild completed with exit code 0 using the existing
pinned external workspace and an isolated cache:

```sh
CRYSTAL_PACK_BUILD_PROFILE=wasm-dev \
CRYSTAL_PACK_BUILD_DIR="$HOME/.cache/geothite/agent-source-timing" \
just crystalpack
```

All 28 intro scenes and 377 audio programs passed certification. The actual
candidate passed battle execution, Mom/New Bark progression, Research Center
startup, MCP movement/Start-menu, and byte-empty stderr regressions before all
three canonical packs were published. The updated production TUI was launched
successfully. This reused source and build caches; the independent cold-start
proof remains the two runs described above.

The packs now contain 159 additional authored scene constants and the authored
Surf executable closure. Strict comparisons preserved every existing script
record, all unrelated game data, and every runtime/audio byte. No generated
packs or verification output are tracked.

## Play

### Wild Challenge and standalone variants

[Wild Challenge](../modpacks/wild-challenge/README.md) scales wild encounters
with the strongest three non-egg party members and builds legal competitive
movesets from the supplied pack. Standard packs enable it together with trainer
profiles. It preserves encounter species, authored level floors, unrelated
content and audio. Build a separate ignored terminal variant with:

```sh
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_wild_challenge -- \
  content-packs/text-tui.crystalpack content-packs/wild-challenge-tui.crystalpack
```

Use `core-modular.crystalpack` or `core-modular.browser.crystalpack` as the input
for the corresponding graphical format. The builder preserves storage format,
verifies the input and output, and rejects overwriting the input. Rebuild the
Rust client to use the rules. Modded saves have a distinct pack identity; choose
a separate save path. The mod's README describes the level curve, move selection
and external-pack regression command.

The native `wild-challenge.crystalpack`, browser
`wild-challenge.browser.crystalpack`, and terminal
`wild-challenge-tui.crystalpack` outputs have been built and reloaded locally.
The verified terminal workflow uses the optimized `wasm-dev` profile:

```sh
export CARGO_TARGET_DIR=target/agent-wild-challenge
cargo build --locked --profile wasm-dev -p geothite
"$CARGO_TARGET_DIR/wasm-dev/geothite" play content-packs/wild-challenge-tui.crystalpack
```

The Wild Challenge integration suite, required battle and Mom/New Bark
controller regressions, MCP movement/Start test, and clean-stderr regression
passed against the generated terminal pack. Interactive launch, Start-menu
input, and a scratch save/reload were also verified.

### Launch a client

The initialization script prints the built text executable's absolute path and
its play command. For a native Windows terminal build:

```powershell
cargo build --release --locked -p geothite
.\target\release\geothite.exe play content-packs/text-tui.crystalpack
```

The same executable supports the stdio MCP session:

```powershell
.\target\release\geothite.exe mcp content-packs/text-tui.crystalpack
```

Build and launch the graphical client with the production title and save flow:

On Windows, `just desktop` builds and installs the graphical executable with
an embedded icon, the validated native pack, model assets, and a Desktop
shortcut. Install `just`, the pinned Rust toolchain, and the Visual Studio C++
tools with Windows SDK first. Keep Git LFS artwork checked out. Use
`just desktop-content` to run the initializer above when native content has not
yet been generated. To select another completed native pack, use
`just desktop 'content-packs/outdoor-base-visual.crystalpack'` or pass its path.
The install lives in `%USERPROFILE%\Games\Geothite`; launch `Geothite.lnk` from the
Desktop. Saves stay in the install's `saves` directory across recipe updates.
The recipe rejects packs that fail the production presentation consumers,
including uncertified intro timing, before changing the install.
The installed app resolves its pack and models beside the executable, opens
without a console, and shows startup errors in a Windows dialog.

For a direct Cargo launch:

```sh
cargo run --release --locked -p crystal-bevy --features voxel-view -- \
  --pack content-packs/core-modular.crystalpack \
  --save-path content-packs/player.crystalsave
```

Press F3 to switch between 2D and 3D. To deploy the generated browser pack:

```sh
docker compose -f docker-compose.production.yml up -d --build
```

## Serve a prepared browser copy

`just serve` starts the Rust server on `127.0.0.1:3003` using the existing
ignored `.local/desktop-3d-runtime/crystal-web-server` executable and `web/`
bundle, including its local browser pack, Wasm modules, and model assets.
Server data stays under that runtime's `data/` directory. Open
[localhost:3003](http://localhost:3003/?multiplayer=off); Ctrl-C stops serving.
Use `just serve 3004` for another port, or `just serve 3004 /absolute/runtime`
for another prepared directory with the same layout.

This command reuses a prepared build. For source rebuilding, run
`sh scripts/cold-start-3d.sh setup-web` once, then
`CRYSTAL_PACK=content-packs/core-modular.browser.crystalpack sh scripts/cold-start-3d.sh web`.
The source path is documented in [the host runbook](cold-start-3d.md).
On 2026-10-13, the source-built loading test client completed three 3D
house-exit/re-entry cycles with production movement and Start-menu input in
Chromium/WebGPU, without rendering errors or context loss. A separate
WebKit/WebGL bundle also completed an exit/re-entry, movement and menu check
without render errors or context loss. Its 2D bedroom/house
renderer remains available; the captured 2D house frame matched the WebGL frame
pixel for pixel. The controller, battle/PP, stdio-MCP movement/menu and clean
stderr regressions passed against the external TUI pack. Browser memory is
reported as an observation; no fixed memory budget is an acceptance requirement. See the
[loading diagnostics](cold-start-3d.md#loading-waterfall-diagnostics) for commands
and the measured limits.

## Optional trainer rematches

The [trainer-rematches overlay](../modpacks/trainer-rematches/README.md) adds
repeat conversations and scaled battles to ordinary and scripted trainers.
Their standard dialogue plays before the short rematch line and battle.
Build it from the completed local pack without rerunning source export:

```sh
cargo run --locked -p crystal-assets --bin pack_trainer_rematches -- \
  content-packs/text-tui.crystalpack content-packs/trainer-rematches-tui.crystalpack
cargo run --locked -p geothite -- play content-packs/trainer-rematches-tui.crystalpack
```

The native/browser inputs use the same builder. Keep outputs ignored and use
the overlay's own saves. See the mod guide for scaling rules and regressions.

## Share packs and isolate builds

The initializer composes [competitive battles](../modpacks/competitive-battles/README.md)
and [Wild Challenge](../modpacks/wild-challenge/README.md) into its native,
browser, and text candidates before publication gates. Hosted pack preparation
uses the same Rust composition. Build a separate local variant from an existing
completed text pack with:

```sh
cargo run --locked -p crystal-assets --bin pack_default_battles -- \
  content-packs/text-tui.crystalpack content-packs/default-battles.crystalpack
```

The builder enables trainer-class profiles, per-move categories, scaled wild
encounters, legal wild movesets, and the shared wild move selector. It preserves
other content and audio, verifies and reloads its candidate before publishing,
and is idempotent. Live wild selection and mutation replay use the same method;
tutorial and contest battles retain their original rules. Native/browser inputs
use the same command. Source and output paths must differ.

The local standard native, browser, and text packs were refreshed from their
completed predecessors with this Rust builder. Their previous versions remain
ignored as `core-modular.before-default-battles.crystalpack`,
`core-modular.browser.before-default-battles.crystalpack`, and
`text-tui.before-default-battles.crystalpack` for existing saves. Use a new save
with the upgraded packs. Content/audio preservation and pack round trips passed
for both native and browser storage, along with the trainer/wild integration,
controller battle and Mom/New Bark, MCP movement/Start, and clean-stderr gates.
The refreshed release text client was launched and accepts real keyboard input:

```sh
./target/competitive-battles-release/release/geothite play content-packs/text-tui.crystalpack
```

The prepared native `just play-3d` client was rebuilt against these rules and
checked with movement and matching 2D/3D captures. This refresh composed the
existing external packs; it did not rerun the full source exporter.

The matching prepared browser runtime is `.local/default-battles-runtime`.
Its Rust server, WebGL/WebGPU clients, and terrain/actor workers were rebuilt.
Relaunch it with:

```sh
just serve 3006 .local/default-battles-runtime
```

Open `http://localhost:3006/?multiplayer=off`. Browser 2D checks exercised Start,
keyboard movement, and complete trainer/wild turns through the production
controller, including PP consumption and an actual trainer switch. A separate
headless outdoor 3D performance probe rendered New Bark and accepted Start,
but exceeded its 240-second host deadline; it is not a passing performance
result. The older server on port 3003 was left running separately.

Use the generated file as `CRYSTAL_RENDER_TEST_PACK` for the controller and stdio gates.
The full source initializer must complete its gates for a new canonical pack
publication.

Run pack initialization once for the checkout. Agents use the published packs;
they do not regenerate exports independently or overwrite the shared workspace.
Use a distinct Cargo target directory for each agent's client builds:

```powershell
$env:CARGO_TARGET_DIR = Join-Path $PWD 'target/agent-title'
$env:CRYSTAL_RENDER_TEST_PACK = Join-Path $PWD 'content-packs/text-tui.crystalpack'
```

```sh
export CARGO_TARGET_DIR="$PWD/target/agent-title"
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack"
```

Choose a different target directory for every concurrent agent. The initializer's
persistent cache defaults to `$HOME/.cache/geothite/pack-builder`; override it
with `CRYSTAL_PACK_BUILD_DIR`. `CRYSTAL_PACK_BUILD_PROFILE` accepts `release`
(default) or `wasm-dev`, and applies consistently to builders and tests. Windows
forwards these two settings into WSL. Keep all generated packs and captures ignored.

### Hall-of-Fame publication gate

The initializer also requires
`kanto_lance_controller_completes_earned_ceremony_and_continues_home` against
the verified native candidate before publication. This earns Lance's victory
through the production controller, runs the original Hall dialogue/machine
approach and ceremony, waits for THE END, returns to title and applies the
production Continue transaction. It checks that credits own movement/Start
input while their presentation clock advances. The source exporter binds
credits metadata to the pinned external source; packs without that operation
fail this gate even when their early-game controller checks pass.

On 2026-10-16 the initializer completed with exit zero on macOS using the
existing pinned source workspace and this task's isolated cache:

```sh
CRYSTAL_PACK_BUILD_PROFILE=wasm-dev \
CRYSTAL_PACK_BUILD_DIR="$PWD/target/kanto-resume" \
CARGO_BUILD_JOBS=1 bash tools/build-crystalpack.sh --jobs 1
```

The normal optimized earned ceremony gate passed without a stack override,
alongside all 28 intro scenes, 377 audio hash/frame/loop checks and the existing
controller/MCP/clean-stderr gates. All three canonical packs were published.
Non-title game data, all 4,915 runtime assets and audio fingerprints matched the
prior pack. The resulting client was launched with
`target/kanto-resume/wasm-dev/geothite play content-packs/text-tui.crystalpack`;
Start opened its production menu and the client exited cleanly. This reused
source and build caches and is not another cold-start proof.

## Natural controller movement

The 3D client defaults to continuous movement for existing and new packs;
arrows, WASD, D-pad, and left-stick movement follow the camera in orbit and
first person, with equal-speed keyboard diagonals. Native Q/E and controller
shoulders rotate continuously while held. 2D and the text client retain tile
controls. Existing Natural Movement packs need no regeneration. The optional Rust
`pack_natural_movement` builder records that choice explicitly in an external
pack, preserving all content and audio. See
[the movement modpack](../modpacks/natural-movement/README.md) for controls,
build arguments, and the `natural_movement: false` opt-out.

The verified outputs are `content-packs/natural-movement.crystalpack` and
`content-packs/natural-movement.browser.crystalpack`, built from the matching
core modular packs. Both remain ignored. Native validation used the production
`render_at_location` example in headless image mode; controller, battle, MCP,
and clean-stdio regressions use the completed external text pack.

## Grand Marais town overlay

The town-only Minnesota prototype reads the completed TUI pack and an original
hand-authored layout. Its Rust overlay retains the base maps, art and audio,
adds one town and six interiors, and redirects this demo's HOME spawn. The
pack stays ignored; this is not the complete Minnesota region.

Verified local build and full-town source-art render commands:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --profile wasm-dev --locked -p crystal-assets --bin pack_grand_marais
./target/runtime-fps/wasm-dev/pack_grand_marais \
  content-packs/text-tui.crystalpack \
  planning/minnesota/playable/grand-marais.json \
  content-packs/minnesota-grand-marais.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_grand_marais_pack.py
```

The Python render requires Pillow and cbor2; it reads the published pack, checks
its layout and event anchors against the design, and renders the complete town.
It does not export packs. After building the production text client, run
`./target/runtime-fps/release/geothite play content-packs/minnesota-grand-marais.crystalpack --save .local/grand-marais.crystalsave`.
The [town runbook](../planning/minnesota/playable/README.md) contains the live
review loop, gameplay regressions, local story and current review scope.

## Historical standalone North Shore and Gunflint overlays

The standalone rectangular `pack_north_shore` and `pack_gunflint` installers
are retired. They reject publication rather than reconstruct the removed coast
connector or restore the longer Gunflint footprint. Their earlier layouts,
runbooks and render evidence remain available as history.

Use the compact Superior build and its verified descendants. The current
selection is `active_local_review_revision` in
[`world.json`](../planning/minnesota/world.json); preserve newer descendants
rather than selecting a pack by modification time or reverting to the compact
foundation. All native pack writers reject the retired connector, retired R12
chunks and obsolete coastal dimensions. Historical packs remain readable.

The legacy source-art review tools require an explicit `--pack`; their
rectangular canvases do not establish current connected native geography.
See the [compact revision evidence](../planning/minnesota/revisions/superior-gunflint-quality-v01/).

## League Approach native route overlay

The supplied ignored TUI pack is the input to the individually authored League
Approach overlay. No source checkout or exporter run is needed for this overlay.
The JSONs contain hand-selected terrain strokes and exact events; the Rust
publisher compiles all eighteen chunks and Cedar Shelter using unchanged Johto
art and ordinary production scripts.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_league_approach -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R13.layout.json \
  planning/minnesota/routes/MN_R13.json \
  content-packs/minnesota-league-approach.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_league_approach.py \
  --pack content-packs/minnesota-league-approach.crystalpack --verify-published
```

The executed build publishes a 407-map pack with original HOME/recovery, source
maps, art and audio preserved. All 13,824 route metatiles and native event/room
anchors match the editor. Desktop validation and all 377 audio PCM hashes,
frame counts and loop ranges pass. A composition regression preserves North
Shore and Gunflint when all three overlays are applied together.

The source-art loop uses `review_league_approach.py --serve` at port 8768; it
rerenders JSON edits without Cargo or publishing a pack. The separate native
controller fixture tests late-route movement, interactions, shelter, Cut,
battle PP and saved flags. Its placement, level-60 party and badges are explicit
test setup. Actual Boundary Waters/Ely ports, Q08 gates and earned progression
remain pending. See [the League Approach runbook](../planning/minnesota/routes/MN_R13.md)
for exact test, review-save and local client commands. Packs remain ignored.

## Headwaters native route overlay

Headwaters adds all nine individually authored chunks to the supplied ignored
TUI pack, retaining the original world, HOME/recovery destinations and audio.
The Rust publisher adds a bounded derived Johto tileset with three native pier
recipes. No glyphs are painted or recolored; original Johto water animation is
retained beneath the wooden rail. There are no buildings or extra interiors.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_headwaters -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R14.layout.json \
  planning/minnesota/routes/MN_R14.json \
  content-packs/minnesota-headwaters.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_headwaters.py \
  --pack content-packs/minnesota-headwaters.crystalpack --verify-published
```

The executed build publishes a 397-map pack. All 6,912 terrain metatiles,
sixteen objects, signs, parties, dialogue and all 131 source/derived block images
match the authoring/native PNG. Desktop validation and all 377 audio PCM hashes,
frame counts and loop ranges pass. All four authored overlays compose together
without replacing source/earlier maps, files, recovery or audio.

The live JSON review is `review_headwaters.py --serve` at port 8769; it needs no
Cargo build or pack publication for each edit. Native controller checks pass
the five circuits, both boardwalk rows both ways, interactions, Cut, trainer
battle/PP and saved flags; native Johto animation frames/timing and clean-stdio
MCP also pass. The fixture explicitly supplies placement, a level-45 party and
six badge bits. Real Bemidji/Itasca hub ports and the earned Q05 journey remain
pending. See [the Headwaters runbook](../planning/minnesota/routes/MN_R14.md)
for exact test, source-art recipe, review-save and interactive client commands.
Packs and generated renders remain ignored.

## Compact Central Lakes review overlay

The user rejected the giant route direction. Central Lakes now adds one
32×18-metatile exterior and one small cabin, rather than nine exterior chunks.
It uses unchanged Kanto source art, a short required grass crossing, a directional
meadow ledge, a railed bridge and one gated dock. It preserves original HOME,
source maps, recovery and audio. Visual acceptance and regional progression
remain open.

Build the ignored native overlay with the verified command:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_central_lakes -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R15.layout.json \
  planning/minnesota/routes/MN_R15.json \
  content-packs/minnesota-central-lakes.crystalpack
```

See [the compact map, exact connections and review/test commands](../planning/minnesota/routes/MN_R15.md).
The JSON review at port 8770 refreshes the whole map and player-scale views
without a pack build. The published overlay contains 390 maps; all 377 audio
programs retain their verified PCM hashes, frame counts and loop ranges.


## Compact Driftless review overlay

Driftless now adds one 32×20-metatile exterior and two interiors to the ignored
source pack. Its native Johto ramp climbs the limestone bench; a HOP_DOWN ledge
returns westbound traffic to the meadow and rejects reverse ascent. The central
grass road is the short alternative, and the shared eastern grass gate remains
mandatory. The dock uses actual directional rails and a closed end.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_driftless -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R01.layout.json \
  planning/minnesota/routes/MN_R01.json \
  content-packs/minnesota-driftless.crystalpack
```

The resulting pack contains 391 maps; exact publication and all 377 audio checks
pass. See [the map, physical contract and native test commands](../planning/minnesota/routes/MN_R01.md).
The geometry fixture uses explicit placement with an empty party. Real
Winona/Rochester ports, trainer/encounter balance and the earned Q01 journey
remain integration work. Do not describe the isolated pack as a playable region.


## Compact Prairie Farm review overlay

Prairie Farm Road adds one 30×22 exterior, Lind Farm and a native workshop.
Its west-facing ledge is a real HOP_LEFT transition: the nursery shortcut works
westbound, while the reverse trip uses the southern lane. A fenced drainage
channel has one railed bridge, and the western pasture grass is unavoidable.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_prairie_farm -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R02.layout.json \
  planning/minnesota/routes/MN_R02.json \
  content-packs/minnesota-prairie-farm.crystalpack
```

The isolated pack has 391 maps and passes exact block/event publication and
all 377 audio checks. See [the whole map, native captures and test commands](../planning/minnesota/routes/MN_R02.md).
The populated-party fixture uses source-created Cyndaquil 12/Mareep 9 and one
badge as explicit setup. Real Rochester/Mankato ports and earned Q02 progression
remain integration work. Packs and derived renders stay ignored.


## Compact Minnesota River Valley overlay

River Valley adds two 24-metatile-wide maps (heights 24 and 8) and an 8×6 cave.
The two timber bridges are both necessary, the lower-bank habitat is required,
and an optional ramp/ledge terrace supplies an Ether and battle. It reuses the
bounded native Headwaters rail extension and preserves every original Johto
metatile and audio program.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_river_valley -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R03.layout.json \
  planning/minnesota/routes/MN_R03.json \
  content-packs/minnesota-river-valley.crystalpack
```

The isolated pack has 391 maps. Geometry and populated-party controller
fixtures pass, including both bridges, native map seam, cave return, four
trainer battles/PP and save/reload. The party, two badges and replay clock are
explicit fixture setup. Real Mankato/Minneapolis ports and earned Q02 remain
integration work. See [the whole map, native captures and exact test commands](../planning/minnesota/routes/MN_R03.md).


The River Valley journey verifies the bridge's Super Potion through the
production Pack, including HP restoration, consumption and saved pickup flag.
The shared controller uses a compiled HP-effect capability from the runtime
item snapshot; the supplied item catalog is unchanged. The route runbook also
records a separate failing runtime battle-fixture setup test.

## Twin Cities park review overlay

The hand-authored crossing adds one 28×18 native exterior. It uses original
Park art plus four bounded source-row recipes for paved gold rails and straight
retaining banks. A dry trainer lane and a four-step grass alternative work in
both directions; a protected bridge and picnic habitat provide distinct
features. Map sizes remain individual design choices.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_twin_cities -- \
  content-packs/text-tui.crystalpack planning/minnesota/routes/MN_R04.layout.json \
  planning/minnesota/routes/MN_R04.json content-packs/minnesota-twin-cities.crystalpack
```

The isolated 389-map pack passes exact terrain/events, native geometry,
populated-party battles/PP, Ether use, save/reload and all 377 audio checks.
Explicit third-badge placement/party setup is separate from earned Q03 and real
Minneapolis/Saint Paul ports. See [the map, native captures and local review
commands](../planning/minnesota/routes/MN_R04.md). Generated packs stay ignored.

## Composed Minnesota map review

The verified local `content-packs/minnesota-region-review.crystalpack` contains
469 maps: all twenty current route layouts, Grand Marais and the preserve/cabin,
in addition to the original game. See the [executed Rust composition commands](../planning/minnesota/playable/region-review.md).
All previous450 map modules and runtime files, existing trainers, source
catalogs, audio and the full HOME/recovery table are preserved. The added town
uses `pack_grand_marais ... --preserve-home`; the isolated demo remains available
without that option. Native verification and all 377audio checks pass.

This is a review pack. Planned regional hub links are not playable connections,
Safari admission remains closed, and local party/badge fixtures do not establish
earned progression. The atlas can compare every current native slice against
this one artifact with `render_progress_atlas.py --region-pack`.

## Connected Boundary Waters basecamp

The additive 471-map `content-packs/minnesota-boundary-basecamp-connected.crystalpack`
publishes the hand-authored 32×24 free town, 8×6 Survey Shelter and three guarded
native walking links to Gunflint, League Approach and Voyageurs. The verified
content hash is `9cb8936dbd1afeeae3abe69e237aeddf12d5c8bccb3a8eb9f8b7ed53bdb45841`.
The publisher preserves existing content, art, audio and HOME/recovery while
adding the town/room and explicit checkpoint/connection bindings; no route
terrain changes or port Floor warps are required.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_boundary_basecamp -- \
  content-packs/minnesota-region-review.crystalpack \
  planning/minnesota/towns/MnBoundaryWaters.json \
  planning/minnesota/towns/MnBoundaryWaters.layout.json \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack --connect-routes
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack --audio --desktop
```

The CLI writes and strictly reads back the full CBOR file. Twenty asset/core
tests, all nine fresh production-controller cases and all 377 audio PCM
hash/frame-count/loop-range checks pass. Real inputs traverse both lanes of all
three seams in both directions, exercise28 prerequisite refusals, preserve
unqualified inward/transverse access to free healing, and verify Surf boundaries
and Continue. The Q07 case plays the chief victory, separately acknowledges the
receiver, walks the unlocked South seam to `MnR13C00R00(30,0)` and Continues.
No post-setup restoration injection is used.

The legal level 80 Mew review partner, eight badges, upstream lighthouse signal and
initial location remain explicit fixture setup. The complete earned regional
journey and optional Safari capture expeditions remain open; admission is closed
and charges nothing. Both bare-town and connected earned-save MCP/TUI input are
verified with empty stderr; connected MCP walks the South seam in both directions.
See the [current town runbook](../planning/minnesota/playable/boundary-basecamp.md)
for commands, local logs and exact coverage. All full-resolution renders,
preconversion originals, conversions, crops, previews and rejected passes remain
archived for reuse. Generated packs stay external and ignored.

### Revising installed authored terrain

The current `content-packs/minnesota-quality-review.crystalpack` retains the
471-map connected baseline and changes only R05's upper-ridge cleft and R08's
pine peninsula. The Rust `revise_authored_route` command requires the exact
previous layout, revised layout and existing dossier. It verifies installed
typed/base64 terrain, source/native anchors and full revised geometry, preserves
events/connections/art/catalogs/HOME, and strictly reads back its written pack.
It does not generate terrain or install dialogue/story edits. Source-art recipe
changes require the separate art/publication path.

All seven revision tests pass with explicit and default pack inputs. Three R05
and two R08 production-controller cases pass on the resulting pack, along with
both actual-save MCP checks and all 377 audio hashes/frame counts/loop ranges.
The complete before/after source inputs and every render stage remain saved.
See the [executed revision commands and exact preservation report](../planning/minnesota/playable/terrain-quality-revisions.md).
The full twenty-route visual and earned regional progression work remains open.

### Original Minnesota cry overlay

The [cry authoring and export runbook](../planning/minnesota/pokemon/CRIES.md)
documents the verified Rust gallery and `pack_minnesota_cries` commands. The
publisher reads the current active roster and original `cry-designs.json`, adds
224 finite MIDI cry programs and replaces those species' cry references in a
separate ignored pack. It preserves the input pack's original audio and all
unrelated content. WAV listening exports stay outside the checkout.

The earlier 223-species publication from `minnesota-city-life-fair-v3.crystalpack` produced
`content-packs/minnesota-cries.crystalpack` with 664 maps, 474 species, 270 moves
and 600 verified audio programs. The current 224-species revision is the separate
`minnesota-cries-v4.crystalpack` candidate described above, with 693 maps,
475 species and 601 verified audio programs. Reading authored programs requires the current
Rust audio implementation, which supports their opt-in output release envelope;
legacy programs retain their original PCM. This verifies audio publication and
runtime decoding, not a new gameplay or auditory-acceptance milestone.

## Northstar character-driven story candidate

The [character-story runbook](../planning/minnesota/playable/northstar-character-story.md)
records the executed Rust ICE, drilling and regional-cast publication commands
and their required frozen local parents. The final ignored pack is
`content-packs/minnesota-northstar-character-story-v5.crystalpack` (559 maps).
Keep its parents and scene saves attached to their original identities; do not
overwrite them or copy a save to another pack identity.

This candidate adds Reed's ICE recruitment and mine obstruction, two maze
chapters with separate controls, witnessed drilling and crew recovery, spoken
character quests and an optional Gillpup egg. The regional binder runs last,
uses active Minnesota species only, and validates native faction moves and
evolution levels. Original audio and source-region content remain preserved.
Nine native story/balance tests, the asset preservation/legality suites and two
final-pack stdio quest replays passed. The runbook identifies the declared
later-chapter fixtures and the limits of that acceptance.

The rebuilt production stdio client was exercised with the real quest saves:

```sh
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/geothite mcp \
  content-packs/minnesota-northstar-character-story-v5.crystalpack \
  --load .local/northstar-character-v5-final-drill-saves/ice-drill-stopped.crystalsave
```

Those saves are verification output, remain ignored, and are not distributed.
See the runbook for source inputs and chapter checks rather than treating a
pack dump as gameplay evidence.

### Cross-floor ICE headquarters successor

The [ICE headquarters runbook](../planning/minnesota/playable/ice-headquarters.md)
records the subsequent Silph-inspired navigation revision. Apply its Rust
`pack_northstar_ice_headquarters` publisher to the immutable unbound
`minnesota-northstar-drill-lair-v4.crystalpack`, then run the regional caster
last. The final local output is
`content-packs/minnesota-northstar-complex-story-v5.crystalpack` (563 maps).
Do not apply the overlay to an already regionally bound pack or migrate a save.

Four new floors bring the ICE dungeon to six floors plus the public office.
Eight native panel pairs and six individually persistent card doors create
cross-floor compartment access and useful returns. Optional staff recovery,
supplies and a second captive remain separate from Reed, holding shutdown,
the enforcer and Ellis's release. A local native-art variant adds real warp
collision without changing original facility behavior or source art files.
All original audio remains unchanged.

The full native course, strict preservation and final-parent regional audits,
actual clean-stdio card replay, and paired native scene review pass. The runbook
contains the exact executed publication/replay commands and declares the
prior-access/party fixtures; the larger gauntlet is not an unboosted full-game
balance or Hall of Fame acceptance claim.


### Interleaved Iron Range successor

The [mine-workings runbook](../planning/minnesota/playable/iron-workings.md)
records the later Rust `pack_northstar_iron_workings` overlay. Apply it to the
unbound `minnesota-northstar-ice-headquarters-v5.crystalpack`, publish
`minnesota-northstar-iron-workings-v3.crystalpack`, then run the regional caster
last to produce the ignored
`content-packs/minnesota-northstar-complex-story-v6.crystalpack` (566 maps).
All three commands were executed and both outputs runtime-verified. Preserve
the immutable inputs and use same-pack saves.

The mine now has seven levels including its public entry: three added shaft
levels interleave eight reciprocal ladder pairs. Two independent native valve
actions each open a local crosscut, and both are needed for the sump shutter.
They cannot grant the original feed, rival, drill, rescue or meter flags.
The native course follows the actual foreman win through all shafts, controls,
Continue, witnessed piston, Reed battle, drive stop, Finn quest and return.
The separate ICE intended-level course uses four level-22/23 Minnesota partners
and real recovery/resources, rather than the level-45 geometry fixture. See the
runbook for exact evidence and limits.

## Minnesota harvests and frozen lakes

The [berry and apricorn runbook](../planning/minnesota/fruit-groves/README.md)
documents `pack_minnesota_fruit_groves BASE SOURCE NEW_OUTPUT`, its current compact
parent, explicit placement contract and ignored output pack. The overlay retains
the two twenty-trainer Mississippi routes, adds native regional fruit harvests,
and pairs with the production daily fruit reset in `crystal-core`. Apply
`pack_frozen_water` afterward for the combined fruit-and-ice candidate; its
[runbook](../planning/minnesota/regions/ice/FROZEN-WATER.md) records the seven
inland winter map revisions, original cave-rock art and solid boundary checks.
North Shore stays warm. The obsolete shore-gate and icy-coast descendants are
retired and cannot be used for new publication.

## Minnesota fishing guru houses

The [fishing progression runbook](../planning/minnesota/playable/fishing-rods.md)
records the Rust `pack_minnesota_fishing_rods PACK OUTPUT` builder and the exact
three-house source contract. The earlier fishing and field-progression packs are historical: their
shore-gate/furnished-home lineage retained obsolete northern geometry. Do not
branch new packs from those outputs. New publication must preserve the
[canonical compact geography](../planning/minnesota/revisions/superior-gunflint-quality-v01/VERIFICATION.md).
The fishing stage adds Old/Good/Super Rod
houses on Driftless Trail, River Valley and Pine Country Road, plus guru
directions after all 27 installed Minnesota fisherman battles. All three rod
controller regressions passed, covering real doors, both exits, gifts, Continue,
pocket retry and an earned fisherman victory. Native 2D is the composition
reference. Additional paired exterior 3D captures passed after fixing native
house/sign model recognition for the Headwaters atlas.
The protected-lineage [fishing-home composition](../planning/minnesota/interior-life/FISHING-HOUSES.md)
uses gate v04, gives all three interiors distinct native layouts and six local
family details, and verifies services, furniture, preservation and native 2D
frames on `minnesota-interiors-fishing-life-v02.crystalpack`.

The [HM progression runbook](../planning/minnesota/playable/hm-progression.md)
records historical build receipts and the withdrawn v03 hash. Current
publication awaits a verified compact/furnished-home parent. Its seven handoffs
follow native badge permissions, with gym directions and 18 visit-unlocked Fly
hubs. All five HM controller tests passed, including the shared production Fly
clock and actual St Cloud-to-Winona menu-driven flight. Original native Fly
ordering and both rod/HM preservation regressions passed.


## Recurring rival and physical intake candidate

The [rival arc runbook](../planning/minnesota/playable/rival-arc.md) contains the
executed Rust publication commands, frozen inputs and validation status for
`content-packs/minnesota-northstar-story-arc-v8.crystalpack` (567 maps), retained
only as historical story evidence. Its outdoor lineage predates the compact
canonical coast; new story publication from that parent is now rejected.
Build `pack_northstar_rival_arc` against the unbound interleaved-mine parent
with reviewed native Kettlurk/Gillpup banks, then apply
`pack_northstar_regional_cast` last. Never apply the binder twice or migrate a
save between pack identities.

This revision has six Reed battles, a St Cloud worker intervention, personal
return dialogue and separate chief/link/rival/shutdown/escape/crew actions.
The physical intake is authored from native facility and cave blocks. The
source, topology and original-pixel checks pass; native scene acceptance and
an ordinary-party New Game through Hall of Fame review remain in progress.
Chapter fixtures do not count as an earned campaign.

## Ely snowfall, pines and the Split Rock back route

The [Ely back-route runbook](../planning/minnesota/revisions/ely-backroute-v01/README.md)
and [executed verification](../planning/minnesota/revisions/ely-backroute-v01/VERIFICATION.md)
record the Rust `build_ely_shore_pass BASE CONTRACT SHA256 NEW_OUTPUT` publisher.
Its compact-Superior child is the ignored
`content-packs/minnesota-ely-snow-shore-pass-v01.crystalpack` (691 maps).
Ely has native snowy pines and falling snow, a reciprocal wooded pass reaches
the North Shore, and a working Pokémon Center stands south of Split Rock.
Both production-controller tests, native-art preservation, byte-identical replay
and all 377 audio checks passed. Final headless 2D captures were inspected;
final 3D visual acceptance and an earned full campaign are unverified.

The [pine-knoll quality successor](../planning/minnesota/revisions/ely-trail-quality-v02/README.md)
is the ignored `content-packs/minnesota-ely-pine-knolls-v02.crystalpack`.
It replaces the straight clinic approach with a winding grass climb and real
westbound ledge return, retains snowy pine Ely and the compact coast, and uses
the native curated Minnesota species/move pipeline. The final two controller
checks, original pass seams/north exit, stdio clinic/ledge/encounter replays and
377 audio checks pass. Its [publication record](../planning/minnesota/revisions/ely-trail-quality-v02/PUBLICATION.md)
contains exact commands, fixture scope and reached captures. The actual 97-map
exterior atlas has no footprint overlaps and identifies remaining regional gaps;
it is not a completed Surf/campaign or full-map-quality claim.

## Minneapolis waterfront native frontages

The [waterfront runbook](../planning/minnesota/revisions/minneapolis-waterfront-v01/README.md)
and [executed publication](../planning/minnesota/revisions/minneapolis-waterfront-v01/PUBLICATION.md)
document `pack_minneapolis_waterfront BASE CONTRACT REVIEWED_SHA256 NEW_OUTPUT`.
The ignored `content-packs/minnesota-waterfront-frontages-v01.crystalpack` is a
691-map child of the pine-knoll candidate. Four complete waterfront fronts use
exact native art while retaining all city collision, doors, rooms, story,
species and audio. Paired native 2D captures, strict readback, the focused
preservation/mutation tests, waterfall/static-art animation tests and377 audio
checks pass. Source epochs and every render remain saved. This is a local
exterior revision; wider map quality and branch integration remain open.

The [current native river review](../planning/minnesota/connections/current-river-v01/README.md)
verifies that same waterfront candidate from Winona to St Cloud and back.
The1,247-action itinerary each way includes all seven bridge passages, native
Waterfall/current and Continue beneath a bridge; the ordinary road above stays
walkable. Five additional Falls/seam/badge checks pass. The explicit initial
fixture uses legal regional SLIDEWAKE; it is not earned campaign progression.
The whole26-map corridor, exact native itinerary, proof,45 reached checkpoints
and eight inspected headless 2D frames are retained. This review does not
republish terrain or audio and does not certify every other lake or map.

## R10 rock gallery on the compact-coast lineage

The [current Iron Range cave runbook](../planning/minnesota/caves/current-iron-v01/README.md)
composes the existing hand-authored R10 gallery into the waterfront parent with
`revise_authored_cave BASE PREVIOUS_ROOM REVISED_ROOM OUTPUT`. The ignored
`content-packs/minnesota-r10-rock-gallery-v01.crystalpack` retains691 maps and
all690 other modules, including the compact coast, snowy pine Ely and river
geometry. It changes139 cave metatiles and the rock border, retaining the
original reader, Revive, four stairs, two drop lanes and mouth pair.

The current Rust publisher's replay is byte-identical. Complete residual
preservation, four controller cases, nine full-frame source/native comparisons,
three reached native captures and377 audio checks pass. The
[publication record](../planning/minnesota/caves/current-iron-v01/PUBLICATION.md)
contains the exact inputs, commands, rejected mutations and repository check
limits. Only this cave is composed here; the other eight cave revisions and
full regional/campaign quality remain open.


### Selected Minnesota outdoor review — 2026-10-28

The [Minnesota script repair](../planning/minnesota/trainers/README.md#missing-battle-return-commands)
documents the verified Rust commands for restoring omitted map-command records
from an existing local pack. The ignored
`content-packs/minnesota-scripts-repaired-r15-v01.crystalpack` restores all 18
Minneapolis cyclist battle returns and retains all 377 verified audio programs.

The verified game build is
`content-packs/minnesota-progression-rewards-v04.crystalpack` (705 maps), recorded
as `progression_rewards_revision` in `planning/minnesota/world.json`. Its parent
is the [accepted R11 terrain](../planning/minnesota/revisions/route-grass-v01/current-r11-v01/README.md).
Follow `active_local_review_revision` for later terrain successors.

The Minneapolis tool workshop gives a rebuilt bicycle, with native PACK
registration and SELECT riding. Tamarack awards one Master Ball after Boundary
Waters restoration. The Winona aide awards Great Balls, Itemfinder, Exp. Share,
Lucky Egg and Rare Candies for caught Minnesota species milestones. Grand Marais
can recover a failed Machine Part delivery without giving another starter.

Five native reward cases, the earned opening, three byte-clean production stdio
replays and all 377 audio checks pass on this exact identity. All terrain,
actors, warps, teams and encounters are preserved; 701 parent map modules are
exact. The [full story](../planning/minnesota/revisions/current-story-v01/README.md),
complex lairs, six rival battles, 24 added grunts and written League remain.

```sh
./target/northstar-story/wasm-dev/geothite play content-packs/minnesota-progression-rewards-v04.crystalpack
```

Use this newly built client for the regional caught-count query. See the
[verified reward build/run receipt](../planning/minnesota/revisions/comprehensive-game-v01/README.md)
and [full completion requirements](../planning/minnesota/revisions/comprehensive-game-v01/requirements.json).
The comprehensive-game goal remains active: HM/fishing integration, missing
species acquisition paths and an uninterrupted full campaign still need work.
Packs, reached saves and renders remain ignored.


## Minnesota Town Map and Fly atlas

The [verified native atlas](../planning/minnesota/town-map/README.md) uses the
original Johto tile sheet and palettes in both Town Map and Fly. The Rust
`pack_minnesota_flight_atlas` stage registered 16 visit-unlocked dry landings on
the compact R13 parent and verified
`content-packs/minnesota-town-map-flights-v01.crystalpack` by readback. The runbook
records exact inputs, hashes, build commands, four passing atlas/controller
checks and inspected production renders. Source artwork/audio are preserved;
the pack and render history remain ignored.

## Tamarack and Team ICE trainer sprites

The [native sprite runbook](../planning/minnesota/npcs/tamarack-ice/README.md)
records the executed Rust encoding/publication commands, concrete Castorock/
Cedar R20 parent and combined pack, normal Crystal six-pose field bases, all
17 ICE variants and eight new trainer encounters. Use
`tools/build-tamarack-ice-sprites.sh` after building the listed Rust tools.
It preserves the parent and requires a fresh ignored output name. The runbook
also gives the concrete production play/MCP commands, preservation/audio
results, actual controller evidence and declared test-fixture limits.
