# Geothite

A Rust implementation of Pokémon Crystal with browser and desktop clients,
multiplayer, and an optional 2.5D overworld renderer. Game logic, rendering,
audio, and networking are implemented in this repository.

The [Tamarack and Team ICE sprite revision](planning/minnesota/npcs/tamarack-ice/README.md)
uses normal Crystal overworld proportions, woman Tamarack and eight additional
trainer types, with private native art and local encounter checks.

The [Minnesota canoe](planning/minnesota/canoe/README.md) adds a reusable key
item, a St. Paul canoe-shop handoff after the Capitol Badge, and native Chris/Kris
paddling sprites. Carry it and walk into open water to launch; step onto land to
portage. Surf remains available. The separate local candidate is
`content-packs/minnesota-canoe-v04.crystalpack`; see its runbook for verification.

The new [Voyageurs Ice Cavern](planning/minnesota/ice-caves/README.md) adds a
Minnesota two-floor ice dungeon off R20, using native sliding puzzles, persistent
rewards and paired return ladders. Its local pack is
`content-packs/minnesota-voyageurs-ice-v02.crystalpack`; the runbook records the
executed Rust build and current verification results.

The [Minnesota TM/HM distribution](planning/minnesota/playable/tmhm.md) stocks
all 50 TMs and seven reusable HMs at Minneapolis Department Store 5F and adds
20 hidden route/cave TMs. Its Rust publisher uses the supplied Minnesota cries pack
and writes `content-packs/minnesota-all-tmhm.crystalpack`.

The default content builds enable [All Pokémon, solo](modpacks/all-pokemon/README.md):
all 251 Pokémon have renewable wild encounters, and Celadon Department Store 4F
sells a Link Cable for solo trade evolutions alongside all six evolution stones.

The source exporter also includes Battle Tower's dynamically selected trainer
dialogue. See the verified [source-text repair runbook](docs/game-content.md#targeted-source-text-repair-on-2026-10-17)
for refreshing a prepared external workspace and preserving the default mods.

The [additive native field-script repair runbook](docs/game-content.md#additive-native-field-script-repair-on-2026-10-26)
covers importing pinned field-animation source while preserving an existing
modded pack and its audio.

The [original Minnesota Pokédex](planning/minnesota/pokemon/README.md) contains 224 active original species with individual GBC palettes and native front/back sprites. The [quadrupedal lynx starter revision](planning/minnesota/pokemon/sprites/mn-f125/DESIGN.md) adds Kindlit → Rimepelt → Borealynx while retaining the furnace family as ordinary Iron Range Pokémon. The [Rust species-pack publisher](modpacks/minnesota-pokemon/README.md) compiles full entries, learnsets, stats and complete evolution families against a supplied local base. The [roster balance review](planning/minnesota/pokemon/BALANCE.md) covers all 223 active species against the campaign level curve, preserves strong Bugs, and requires the modern physical/special split. Generated packs stay ignored; regional encounter placement remains separate authoring. The [Fairy ruleset](docs/northwoods-fairy.md) introduces thirteen regional Fairies, earned attacks and practical Poison/Steel counterplay while preserving its frozen 220-species baseline. The [sticky-note restoration](docs/northwoods-sticky-notes.md) restores Jotlet → Memofold → Recallisk, with a verified 474-species/270-move pack, native evolution/battle checks and the published 223-species Pokédex (`20261027-sticky-13`).

All **224 active Northwoods Pokémon** now have individually authored six-pose
16×16 overworld sprites with their own normal/shiny palettes. The
[verified complete overlay](planning/minnesota/pokemon/overworld/20261027-v1/README.md)
builds `content-packs/northwoods-overworld-v1.crystalpack` and supplies the field
and battle PKMN menus. The [live Pokédex](https://geothite.ryanculligan.com/pokedex/)
includes all 224 animated walkers and cries in release `20261027-overworld-224-18`.

The [Northwoods animation overlay](docs/northwoods-move-animations.md) gives all
nineteen added moves native Gen II attack programs and exports their battle
previews through the production renderer.

The [Minnesota cry performances](planning/minnesota/pokemon/CRIES.md) cover
224 active Pokémon in 101 lines, with explicit replacement scores for all evolved
forms. Revision 8 develops all six active legendaries, both mythicals and the
three-stage pseudo-legendary line after the full-roster revision 7. Rare adult
calls now run 2.71–3.48 seconds, with independently authored openings, climaxes
and resolutions. The [current listening gallery](http://127.0.0.1:8937/crafted-v8/listen/)
starts with these comparisons and includes every species, evolution sequences,
Crystal legendary references and revision-7 previous versions. Generated WAVs
and the separate `minnesota-cries-v8.crystalpack` candidate stay ignored or
outside tracked source. The runbook distinguishes playback checks from listening
judgment. All 224 revision-8 cries are now live in the
[Pokédex](https://geothite.ryanculligan.com/pokedex/), preserving release 19’s
profiles, sprites and starter signature moves.

The [State Fair Chaos tab](https://geothite.ryanculligan.com/pokedex/state-fair.html)
plays a 58-second normal-speed native showcase with five crowd close-ups, scene buttons and a
full-resolution fairgrounds map. Release `20261027-state-fair-realtime-22` preserves the
latest 224 cries and all existing species and battle-animation content.

Current [Minnesota tile art](planning/minnesota/tiles/campfire-animation-v1/README.md)
focuses on a transparent 32×32 campfire with a gray stone ring and flickering warm flames.
Rust imports its native per-glyph palettes and preserves the original map art.
The earlier six-prop batch remains rejected history.

The [Minnesota map review pack](planning/minnesota/playable/region-review.md)
composes all twenty authored routes, Grand Marais and the Boundary Waters
preserve/cabin. The [Boundary Waters basecamp milestone](planning/minnesota/playable/boundary-basecamp.md)
adds a real town, Survey Shelter and three guarded walking links to Gunflint,
League Approach and Voyageurs. The connected 471-map pack passes native file
read-back,377 audio checks,20 asset/core tests and all nine production-controller
cases, including earned local chief victory, receiver restoration and South
departure with Continue. Review party, badges and upstream signal are explicit
fixture setup; the full regional journey and Safari capture expeditions remain
open. All original renders, conversions and rejected passes are retained.

The [Minnesota city population overlay](planning/minnesota/playable/city-life.md)
adds local residents, bounded walking NPCs and one native Pokémon trade per
installed town. Its Rust publisher preserves the compact-city, Ely/League and
State Fair pack lineages; exact inputs, trades and verification are in the runbook.

The [Minnesota interior-life pass](planning/minnesota/interior-life/README.md)
authors room purposes, local conversations, residents and optional neighborhood
stories for the northern-winter pack's 187 non-gym interiors. Its Rust publisher
rejects missing coverage and preserves existing quests and services. The runbook
tracks verification and the registered rooms still absent from that pack branch.
The [physical room pass](planning/minnesota/interior-life/LAYOUTS.md) adds
24 distinct room compositions and 25 native furniture interactions over that
story pack, including the hostel dormitory, workshop floors and public halls.
The [St. Paul facade and river repair](planning/minnesota/towns/st-paul-repair/README.md)
replaces mixed house/clinic assemblies with complete native buildings and
rebuilds the waterfront banks, quay and working basin while retaining its rooms.
The [Duluth club pass](planning/minnesota/interior-life/CLUBS.md) adds working
sail/letter rooms, a timber kitchen, lodge spaces and 16 readable local details.
The [preserved interior composition](planning/minnesota/interior-life/RAIL-STOP-CORRECTION.md)
uses `content-packs/minnesota-interiors-no-red-wing-v01.crystalpack`
(719 maps, 217 non-gym interiors). Red Wing's station has been withdrawn by
user direction and removed from the active sources and rail-stop plan.
The two native timetables now list the five approved stops. This retains the
St. Paul facade/waterfront repair, Staples' supplies, Split Rock's clinic/PC,
fishing homes, shore gate and Winona depot lineage.
The current visual direction is native 2D only.
The 719-map interior branch is now **preserved history, not an approved new
publication parent**: it retains the obsolete coast/Gunflint geometry. Current
interior publishers refuse it. See [the compact rebase](planning/minnesota/interior-life/COMPACT-REBASE.md) for the protected room work, geometry conflicts
and pending physical entrance integration. No new stale-geometry pack was published.
 Canonical coast/Cedar and Ely
back-trail composition, full five-stop service, the Staples ledger repair and
regional room-quality review remain open. Counts do not complete that scope.
The historical
[city renderer notes](planning/minnesota/interior-life/CITY-COAST-FOLLOW-UP.md)
record the facade, snowy-tree, bridge/depot and rocky waterfront corrections.

The [smaller, tighter major cities](planning/minnesota/playable/major-cities.md)
publish whole64×48 Minneapolis, St Paul and Duluth in a660-map review pack,
with117 new rooms, native shops and local activities. Canonical planning files,
whole-town/player-camera reviews and a measured2.11-second warm street-render
loop are available in the runbook. Local controller evidence and final visual
acceptance remain separate from source and pixel checks.

The [Northstar story review](planning/minnesota/playable/minneapolis.md) now
combines Minneapolis's ICE battle, separate captive release/dispatch reading
and canonical Avrin gym with the northern story and Ryan's waterfall cabin.
The 515-map `content-packs/minnesota-northstar-city-story.crystalpack` retains
the revised mine/hollows, expanded Northwoods and guarded Grand Rapids/Duluth
links. Twenty targeted chapter/travel/cabin cases, actual MCP and all 377 audio
checks pass on the combined pack.

The [Rochester successor](planning/minnesota/playable/rochester.md) connects
Winona through R01 to the original-readings delivery and Lode-stamped shipment
clue. The 526-map `content-packs/minnesota-northstar-rochester.crystalpack`
passes an earned New Game delivery/return, both starter-checkpoint lanes,
separate native Gym1 battles/reward, actual stdio and all 377 audio checks.
One complete earned Cyndaquil-to-Care-Badge run and its stdio Continue now pass;
balance/retry coverage and the remaining earned regional journey stay open.
Start with `target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-rochester.crystalpack`.

The [538-map regional gym successor](planning/minnesota/playable/eight-gyms.md)
now combines all eight current gym rooms and actual city facades with this
opening/story lineage. One production suite passes all eight local gym courses,
including trainer wins, puzzles, badges/TMs, saves and both returns. The earned
Winona/R01/Rochester chapter checks still pass. The merged native cities retain
the verified care/PC/mart and R04/R05/R06 seams. Missing regional links, the
courier, rail and the continuous earned badge journey remain unfinished.
The linked runbook contains exact Rust inputs, build commands and fixture limits.

The [Rochester two-switch revision](planning/minnesota/gyms/rochester-gates.md)
replaces the rejected planter maze with a smaller room: open the left lane,
reach the upper switch, then return and swap to the right lane for the leader.
Its separate541-map pack passes a literal control course, gate reversals,
saving and both city exits with Totodile16/Hoothoot14. The runbook records the
executed Rust publication and native state views.

The cumulative [Mankato terrace revision](planning/minnesota/gyms/mankato-terraces.md)
adds clear sand levels, visible stairs behind wooden gates and native one-way
drops. Its fixed Bayleef/Pidgey control run proves the required hop, retreat,
leader, saving and both city exits while preserving Rochester's gate course.
The [Minneapolis circuit revision](planning/minnesota/gyms/minneapolis-circuit.md)
alternates trainer battles and switches, with visible electrical barriers.
Its Geodude/Bayleef literal course proves both locks, resets, Continue and both
city exits on the cumulative541-map pack. The [St. Paul pad-loop revision](planning/minnesota/gyms/st-paul-loop.md)
connects both trainer rooms directly to the leader landing and a short return.
Its Quilava/Geodude course verifies every reciprocal pad, saving, rewards and
both city exits. The [St. Cloud arrow revision](planning/minnesota/gyms/st-cloud-arrows.md)
makes wind direction visible on every lane. Its Ampharos/Graveler course proves
all four forced crossings, wrong-way entries, saving and both city exits.
The [Brainerd dock-control revision](planning/minnesota/gyms/brainerd-winch.md)
replaces a trainer's position-choice menu with one blue fixture that toggles the
piers on A. Its ordinary-team course verifies both trainers, reversals, saving,
rewards and city exits. The [Bemidji garden revision](planning/minnesota/gyms/bemidji-return.md)
keeps the two native Cut hedges and required trainers, then opens a gold gate
after Orlis for a direct return. Its ordinary-team literal course, Cut denials,
saving and both city returns pass. The [Duluth ice revision](planning/minnesota/gyms/duluth-gold-gates.md)
distinguishes moving gold gates from permanent silver stops. Native slides,
required trainer order, losses, badge permission, saving and both city returns
pass. A fixed four-partner run also earns all three trainers and Norek through
native slides, PKMN switches and Pack healing. All eight revisions now also
rebuild through [the Rust regional gym publisher](planning/minnesota/gyms/current-regional.md)
in the 554-map story and 660-map compact-city branches. Original city doors,
story data, parties and audio remain intact; all sixteen native room-camera
comparisons and both branches' paired city returns/Continue checks pass.

The [prairie story successor](planning/minnesota/playable/prairie-story.md)
connects Rochester through R02 to Mankato. Care permits departure; return is
free. A paper notebook beside the garden exposes copied readings after both
pages are acknowledged, without becoming a gym prerequisite. Two preservation
cases, three local controller cases, actual stdio round-trip and all 377 audio
checks pass. The local setup supplies Care and an empty party; a continuous
earned trip remains unproven. Run
`target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-prairie-story.crystalpack`.

The [River Valley successor](planning/minnesota/playable/river-cities.md) connects
Mankato through R03 to Minneapolis, with a native second-badge departure gate and
free inbound return. A continuous local test earns Torven's badge, walks the
valley, earns Avrin's badge and returns to Mankato. Its first badge, initial
placement and strong party are fixtures; an earned starter journey remains open.
The second continuous fixture follows the road into the ICE rescue/dispatch
chapter and returns without badge3. Five controller cases, two preservation
cases, actual stdio round trips and all377 audio checks pass on the v2
`content-packs/minnesota-northstar-river-story.crystalpack`; six native forest
caps close the partial candidate's stray south-bank exits.
The runbook records the exact Rust build, preservation and native-view checks.

The [courier story successor](planning/minnesota/playable/courier-story.md)
connects Minneapolis through R04 to St Paul. Current opens the road; the actual
ICE dispatch copy separately permits the courier fight. Victory and reading
the DEEP BORE manifest are saved independently. Five local controller cases,
two preservation cases, actual stdio and all377 audio checks pass. Prior badges,
dispatch and review party remain explicit setup; the full earned journey is
unfinished. Run
`target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-courier-story.crystalpack`.

The [lake-country successor](planning/minnesota/playable/lake-country-story.md)
connects Minneapolis/R05/St Cloud/R06/Brainerd. Current and Capitol open R05;
Wind opens R06, with free inbound return and no courier requirement. A continuous
local course earns Wind and Lake, reads the ranger's conflicting report and
receives Surf once. Five controller cases, two preservation cases, actual stdio,
four matched native cameras and all377 audio checks pass. Initial prior badges
and review party are setup; Brainerd's later branches and the full earned
campaign remain unfinished. Run
`target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-lake-country.crystalpack`.

The [Leech Lake shore successor](planning/minnesota/playable/leech-shore.md)
adds the actual shore settlement, two usable cabins, free care and local
testimony. Lake-only R07/R08 connections now reach Bemidji. A continuous local
course earns Pine after a played loss, native clinic recovery and Fire-move
retry, then walks home. Six controller/two preservation cases, three actual
stdio replays, matched native views and all377 audio checks pass. Prior badges,
partner and RNG are fixtures; this parent precedes the Itasca successor below.
The full earned campaign remains unfinished. Run
`target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-leech-shore.crystalpack`.

The latest [short coast and compact Gunflint revision](planning/minnesota/revisions/superior-gunflint-quality-v01/VERIFICATION.md)
publishes `content-packs/minnesota-superior-compact-v03.crystalpack` with689maps.
It removes twenty metatiles of the straight Grand Marais approach, retains a
short warm coastal bend into town's west side, shortens Gunflint to32×32 and
retires the long inland Neck. Thirteen production controller cases, native
source parity, all377 audio checks and byte-identical replay pass. The runbook
includes the actual connected render, headless player views and exact replay
inputs. Earlier designs, packs and rejected renders remain history.

The earlier [Superior coast and inland winter correction](planning/minnesota/revisions/north-shore-ely-geography-v01/VERIFICATION.md)
publishes the 694-map `content-packs/minnesota-superior-inland-winter-v01.crystalpack`
from the supplied native-art parent through three guarded Rust stages. North
Shore enters Grand Marais from the west over shared Superior water; both stay
warm, and the separate north exit leads inland to winter. The runbook records
exact pack hashes, replay commands, the actual connected render, passing local
controller checks and canonical Rust source parity. It supersedes the older
coastal geography; earned statewide progression remains separate.

The historical [544-map North Shore successor](planning/minnesota/playable/north-shore-story.md)
connects Grand Marais through the actual Ilex–Azalea gatehouse and the two-lane
Gunflint walking seam. Both town departures check eight badges and the Duluth
signal; inbound care is free. The optional local lamp repair grants no starter
or regional signal and cannot block either road. Six local controller cases,
two preservation cases, three actual stdio replays, the existing full-bag keeper
regression and all377 audio checks pass. Both clinics now register real blackout
recovery, proved through played losses and Continue. The complete earned course
from Brainerd through Harbor to Boundary Waters and back also passes, with no
downstream flags or reserve partner inserted. The client builds. Run `target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-north-shore-story.crystalpack`.

The [554-map Ely/League successor](planning/minnesota/playable/ely-league.md)
adds a complete final rest camp, the authentic Ilex Forest gatehouse lobby,
independent eight-badge/survey registration, five native battles, Hall of Fame
recording/credits, Ely postgame Continue and rematch entry. Five controller cases,
source/native views, preservation and all377 audio checks pass. Prior access,
Mew100 and carried medicine are explicit review fixtures; the fresh Winona
campaign and Kettlurk's visible set piece remain open.
Run `target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-ely-league.crystalpack --save .local/ely-campaign.crystalsave`.

The same story pack now has a [fresh Winona-to-ICE controller course](planning/minnesota/playable/minneapolis.md#new-game-through-two-badges-and-ice--2026-10-27):
earned starter/capture, delivery, Care, Prairie, native evolution/TM teaching,
ICE victory, separate captive release and dispatch reading, and return home.
Native blackout/Continue and actual stdio are checked; the later full earned
campaign remains open. The runtime also fixes native damaging stat effects such
as Mud-Slap when optional pack annotations are absent, preserving pack identity.

That earned party now also wins Current, follows the ICE dispatch to the
[St. Paul courier and separate DEEP BORE record](planning/minnesota/playable/courier-story.md#earned-courier-capitol-and-minneapolis-return--2026-10-27),
earns Capitol and returns to Minneapolis with four badges. Native controller
and actual stdio record checks pass. These saves belong to the Ely story parent;
the selected-gym/species successor's own compatible course is recorded below.

The [Kettlurk relay candidate](planning/minnesota/playable/northstar-relay.md)
combines those selected gyms and the existing regional species catalog with a
native remote-feed picture and a visible console shutdown. The separate chief
battle and acknowledged receiver action remain required. Local controller,
preservation and all 377 audio checks pass. A fresh New Game and actual saved
continuations now earn four badges, ICE rescue/evidence, the courier's separate
manifest and the Minneapolis return on this new identity. Both evidence scenes
also pass actual stdio replay. Wind onward and the complete earned campaign
remain open.

The [local sleep-recovery successor](planning/minnesota/playable/northstar-relay.md#published-local-sleep-recovery-successor--2026-10-27)
adds the existing purchasable Awakening at St Cloud before Saven. Its Rust
publication/readback, full preservation, all 377 audio checks and affected-crate
Clippy pass. Its fresh party earns two badges before four ICE losses. At the
user's request, an explicit level-75 story review now continues from that real
checkpoint and passes five badges, both evidence chapters and Brainerd arrival.
Lake, the separate ranger report/Surf, Pine and the independent Itasca report
also pass. The full Itasca/Iron Range road stdio replay passes after correcting
compact status for native escape notices. The continued party now clears the
mine, separate custody/depot handovers, Harbor, the coast, chief and native
relay shutdown. After raising the review minimum to 100 for the last three
League matches, Hall of Fame and postgame Continue also pass. Earlier saves
retain their original pack; these reviews do not establish unboosted balance.

The [comprehensive dialogue overhaul](docs/dialogue-overhaul.md) rewrites the
required story from Winona through the Boundary Waters and Hall of Fame:
279 conversations across 36 maps, including refusals, reminders and returns.
It stages the opening and courier encounter, preserves authored pages and
native text timing, and gives each handoff a clear speaker, reason and next
destination. Its 2D runbook records candidate publication and acceptance
separately from the historical text-only revision.

The [dialogue quality revision](planning/minnesota/playable/northstar-relay.md#dialogue-quality-revision--2026-10-27)
is published as the ignored
`content-packs/minnesota-northstar-dialogue.crystalpack`. Its opening, rescue,
copy trail and finale use the revised [Northstar dialogue](planning/minnesota/STORY.md#dialogue-revision--2026-10-27):
ELLIS and JO give the rescue a personal aftermath, the villains have distinct
unrepentant voices, and the harbor crew prepares for the shutdown. The runbook
records the new identity and verification scope; earlier saves retain their
original pack identity.

The [character-driven story revision](planning/minnesota/playable/northstar-character-story.md)
adds Reed's ICE recruitment and drill obstruction, two playable maze chapters,
motivated crew quests, and Tamarack's optional Gillpup egg. The ignored
`content-packs/minnesota-northstar-character-story-v5.crystalpack` uses only
active Minnesota species in its regional story. ICE mixes parasites and road
slush with Cindore/Cartick coverage and Wrigglet-line boss aces. The runbook
records the executed Rust publication commands, native and stdio checks,
paired scene renders, and the separate limits of battle and geometry fixtures.

The [cross-floor ICE headquarters](planning/minnesota/playable/ice-headquarters.md)
has six dungeon floors, eight paired transfer routes, six persistent card doors,
isolated offices and optional recovery/supply/captive rooms. The subsequent
[Iron Range workings](planning/minnesota/playable/iron-workings.md) adds three
interleaved mine levels, eight ladder pairs, two independent pressure controls,
and physical crosscuts before the existing working drill. The combined ignored
`content-packs/minnesota-northstar-complex-story-v6.crystalpack` runtime-verifies
with 566 maps. Native story courses, Minnesota-only binding, preservation and
clean-stdio valve replay are documented in the runbooks. Prior packs/saves stay
unchanged.

The [recurring rival revision](planning/minnesota/playable/rival-arc.md) adds
Rochester, St Paul and Boundary battles for six Reed battles in total, an acted
St Cloud intervention, changed return conversations and a physical Kettlurk
release. Its ignored candidate is
`content-packs/minnesota-northstar-story-arc-v8.crystalpack` (567 maps), retained
only as historical story evidence. Its outdoor lineage predates the compact
canonical coast; new story publication from that parent is now rejected.
Native acceptance and the ordinary-party full campaign review are in progress;
the runbook separates completed checks from remaining work.

The [earned northern finale](planning/minnesota/playable/northstar-finale.md)
now continues that actual arrival save through the chief battle, separate
receiver decline/accept/acknowledgment and the full reopened R13 road. The chief
win alone does not open the League gate. Two actual stdio replays also pass.
Ely and its League remain unbuilt in this frozen pack; no full campaign is claimed.

The earlier [Itasca story successor](planning/minnesota/playable/itasca-story.md) adds the
actual Ilex Forest–Azalea gatehouse, the headwaters marker and the separately
acknowledged Bemidji comparison. Lake permits an early R14 visit; Pine plus the
completed report opens R09. Five controller/two preservation cases, three stdio
replays and all 377 audio checks pass on the 543-map pack. Prior badges/partner
remain fixtures; the full earned campaign and remaining links are unfinished.
Run `target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-itasca-story.crystalpack`.

A [continuous Pine-to-harbor course](planning/minnesota/playable/northstar-iron-course.md)
now earns the report, mine victory/recovery, depot handover, Harbor and the full
drainage reveal in one local session. Played blackout preserves earned custody;
four actual stdio replays pass. Six prior badges/Mew45 remain setup. That frozen
parent has no Grand Marais regional links; the successor above adds them.
The Ely/League successor above is separate; the full earned campaign remains unfinished.

The separate [fourth-gym approach](planning/minnesota/playable/gym-four-approach.md) opens
Minneapolis's east road through R04 to St Paul. A Current Badge
checkpoint preserves free inbound return. One local level45/two-prior-badge
course earns the rescue, third and fourth badges through their trainers and
puzzles, uses native clinic care, and walks home. Asset, controller, actual
stdio facade/return, matched native views and all377 audio checks pass.
This remains explicit local setup; courier, later links and the full earned
regional journey remain unfinished.

The [Winona opening successor](planning/minnesota/playable/winona.md) adds real
New Game, the guardian scene, three starter choices with native naming, and
the original survey readings/five-ball handover. The 521-map
`content-packs/minnesota-northstar-winona.crystalpack` passes three earned local
opening journeys, a separate full-pocket retry case, actual stdio MCP and all 377
audio checks. R01/Rochester, remaining regional links, St. Paul courier, League
finish and Kettlurk's encounter remain unfinished. Start locally with
`target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-winona.crystalpack`.

The cabin's actual door, Ryan dialogue, sign, both exits and Continue pass
production input and MCP; original content/art and all 377 audio programs are
preserved. The [cabin runbook](planning/minnesota/playable/ryans-house.md) records
exact sources and proof. This local fixture starts beside the falls with an
explicit review partner and no badges; walk up to enter the house:

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-city-story.crystalpack \
  --load .local/minneapolis-integrated-north-saves/ms10-ryans-house-porch-54327-1793073003729602000.crystalsave
```

The [current Iron Range and North Shore revision](planning/minnesota/playable/geography-revisions.md)
adds rock ribs, ore aprons, two rivers, new bridges and real native cascades to
the471-map review pack. Thirteen targeted controller cases, both actual journey
MCP sessions, exact unrelated-content preservation and all 377 audio checks pass.
The [State Fairgrounds](planning/minnesota/playable/state-fairgrounds.md) now spans
one48×36 map. The latest local maze candidate has255 outdoor characters,
including35 trainers,10 Pokémon exhibits with judges,32 shops and large native
counter pavilions. Distributed groups include slower walkers, brisk runners,
turning spectators and standing visitors, all with dialogue. Its32 shops sell16 fair foods, including sweet corn, cookie pails,
cheese curds, hotdish, pronto pups and beer. Beer restores100 HP and causes four
turns of confusion. The grass revision adds
14 native tall-grass patches with all29 active food/carnival species, including
every evolution at all times. Its engine supports255 NPC records on expanded
maps. The [fair runbook](planning/minnesota/playable/state-fairgrounds.md) records
current gameplay, preservation and native visual verification. The candidate
pack stays separate from later regional integration lineages. All source/native
render generations are retained for reuse.

The [preceding terrain quality pass](planning/minnesota/playable/terrain-quality-revisions.md)
adds a hand-authored limestone cleft in River Bluffs and a connected pine
peninsula in Northern Lakes. That preserved471-map review pack retains all other
content, passes five focused route-controller cases and retains every render
and its complete authoring inputs for reuse. The twenty-route quality goal
remains in progress.

## Features

- Classic 2D presentation with a switchable 2.5D overworld view.
- Keyboard, touch, and gamepad controls with responsive browser layouts.
- Shared-world multiplayer, player chat, battle invitations, and trading.
- Browser saves tied to a persistent player identity.
- A shared server clock for the hosted game's day and night cycle.
- A Docker build that packages the server and browser client together.

This is an actively developed project. See the [fidelity audit](FIDELITY_AUDIT.md)
for implementation status and remaining differences from the original game.

## Setup

### Terminal install

Install the prebuilt Rust client and start playing:

```sh
curl -fsSL https://geothite.ryanculligan.com/install.sh | sh
```

Downloads are published for Apple Silicon macOS and Linux x86_64. The installer
checks executable, launcher and content SHA-256 hashes. It installs under
`~/.local/share/geothite`; rerunning it preserves `saves/`. Use
`sh -s -- --no-play` at the end of the pipeline to install without starting.
Run `~/.local/share/geothite/bin/geothite` later, or append `mcp` for the stdio
server. See the [text-client guide](modpacks/text-tui/README.md).

### Native desktop app

With the local content pack prepared, the same commands select your current OS:

```sh
just desktop             # Build, validate and install
just desktop-run         # Open the installed app immediately
just desktop-package     # Build a distributable in .local/desktop-dist/
just desktop-repack      # Package the last build without compiling
```

On macOS, `just desktop` installs **Geothite.app** in `~/Applications` with a
crystal icon, the production title/new-game/Continue flow, the supplied native
pack and 2.5D artwork. `just desktop-package` creates an architecture-specific
DMG with an Applications shortcut and SHA-256 checksum. The app runs independently
of this checkout; F3 switches between 2D and 3D. Saves live in
`~/Library/Application Support/Geothite/saves`, so replacing the app preserves them.
The desktop app plays music and sound effects by default. Normal web play enables
sound on the first click, touch or keypress, subject to browser autoplay rules;
the sound button remembers deliberate mute choices. Native developer clients,
browser location previews (`?preview=...`), embedded training views and
`?development=1` sessions always stay muted.

The 3D view uses simpler meshes for distant pines while keeping nearby trees at
full detail. See the [rendering performance notes](docs/johto-3d-performance.md#pine-distance-lod)
for measurements and the full-detail comparison control.

On Mac, **Settings** appears in the window title bar and the Mac menu bar.
It controls master volume, mute, 2D/3D view, a checkable First Person toggle,
camera presets and fullscreen. Orbit rotation follows the character, including
inside rooms where the classic viewport stops scrolling.
Settings follow keyboard/controller changes and persist in
`~/Library/Application Support/Geothite/settings.json`. Camera reset and
Reset Settings are available in the same menu.

On Windows, installation goes to `%USERPROFILE%\Games\Geothite` and creates a
Desktop shortcut; packaging creates a ZIP. Linux installation goes to
`~/.local/opt/geothite` with an application-menu entry; packaging creates a native
`tar.gz`. Build on each target OS. Windows and Linux packaging are prepared;
the macOS path is the focus of local verification. These are native host builds,
not cross-compilation or a universal Mac binary.

Install Rust 1.99 (selected by `rust-toolchain.toml`), `just`, and Git LFS.
macOS also needs Xcode command line tools and Python 3.9+; Windows needs the Visual
Studio C++ tools/Windows SDK; Linux needs the libraries in
[the native runbook](docs/cold-start-3d.md). On a fresh checkout, run
`git lfs pull` and `just desktop-content` once. Preparing source content and a cold
Bevy build can take substantially longer than two minutes. Desktop commands use
fully optimized release builds by default and share a persistent build cache.
`desktop-repack`, `desktop-install` and `desktop-run` skip compilation.

Earlier Apple Silicon checks using the `wasm-dev` development profile produced
a DMG in **2m 13s** and installed it in **52s** with a warm cache. The initial
build took substantially longer. That installed app was opened through Launch
Services and reached the interactive New Game flow.
`just desktop-run` captures its launch log in `~/Library/Logs/Geothite/game.log`.

`just desktop-install` installs the last build without compiling, and
`just desktop-check` validates the last build's content/art without opening a
window. Pass a different local pack as a recipe argument, for example
`just desktop-package content-packs/all-pokemon.crystalpack`.
Use `GEOTHITE_DESKTOP_PROFILE=wasm-dev` for shorter development builds with
lighter game-code optimization. Keep the same profile when building and installing.
Close the installed app before updating it. All build products, copied packs,
and distributables remain ignored; they are never source deliverables to commit.

Mac packages use ad-hoc signing for local use by default. For public distribution,
configure `GEOTHITE_SIGN_IDENTITY` with a Developer ID Application certificate
and `GEOTHITE_NOTARY_PROFILE` with an existing `notarytool` keychain profile, then
run `just desktop-release`. This signs with hardened runtime, submits the DMG to
Apple, and staples/validates the notarization ticket. It requires your Apple
Developer credentials; the local package is not notarized. See
[Apple's distribution guide](https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution).

Generate local game content with the repository's Rust builders, then use Docker
with Compose for your own server. [Game content setup](docs/game-content.md)
contains the required tools, pinned source inputs, validation, and play commands.

### 1. Clone and initialize game content

```sh
git clone https://github.com/TheCulliganMan/geothite.git
cd geothite
```

With Git LFS and Rust installed, initialize from Windows using WSL Ubuntu-24.04:

```powershell
.\tools\build-crystalpack.ps1
```

With `just` installed, the same entry point selects the platform initializer:

```sh
just crystalpack
```

On macOS, source assembly also requires Homebrew Bison, pkgconf, and libpng.
RGBDS is built from its pinned source commit in the external workspace. The
direct Bash entry point works on macOS and Linux x86_64:

```sh
bash tools/build-crystalpack.sh
```

The script assembles the pinned pret source outside the checkout, exports game
content with Rust, verifies audio and gameplay, and publishes ignored native,
browser, and TUI packs under `content-packs/`. Concurrent agents use these packs
and separate Cargo target directories; see the [build runbook](docs/game-content.md#share-packs-and-isolate-builds).
Publication also checks an earned Lance victory through the Hall of Fame,
credits, title return and the production Continue transaction to New Bark.

### 2. Create your server secret

Create a signing secret once for a new installation:

```sh
umask 077
printf 'CRYSTAL_AUTH_SECRET=%s\n' "$(openssl rand -hex 32)" > .env
```

### 3. Start the server

```sh
docker compose -f docker-compose.production.yml up -d --build
```

Open [localhost:3003](http://localhost:3003). The first build compiles Rust and
WebAssembly dependencies and can take several minutes. Docker includes the
required toolchains and copies your locally supplied game pack into the image;
a separate Rust installation is unnecessary for the Docker build.

Keep `.env` across updates. For remote access, put the server behind HTTPS.
The web **2D / 3D / TUI** chooser swaps renderers within the same game session.
Use `/?view=tui` or `/tui/` to open the original painted terminal surface;
`/?view=2d` and `/?view=3d` select the graphical views. Switching keeps the
current trainer, position, dialogue, battle, menu, and save state.

See the [deployment guide](docs/deployment.md) for configuration, reverse
proxying, upgrades, health checks, persistence, and cleanup.

## Playing

For the prepared browser copy in this checkout, run `just serve` and open
[localhost:3003](http://localhost:3003/?multiplayer=off). Use `just serve 3004`
to select another port; Ctrl-C stops the server. This quick command reuses the
ignored `.local/desktop-3d-runtime` server and browser bundle without rebuilding.
See [local browser serving](docs/game-content.md#serve-a-prepared-browser-copy)
for input requirements and rebuilding from source.

`just play-3d` launches the prepared Bevy desktop client directly in the
bedroom with 3D enabled. For the same entry in the browser, open
[the 3D bedroom](http://localhost:3003/?preview=bedroom&multiplayer=off).
The prepared browser build selects an available WebGPU adapter and falls back
to WebGL when one is unavailable. `just trace-3d` records the native loading
waterfall in `.local/loading-trace.json`; browser tracing uses
`&loading-trace=on`. See [loading diagnostics](docs/cold-start-3d.md#loading-waterfall-diagnostics).


| Action | Keyboard |
| --- | --- |
| Move | Arrow keys or WASD |
| Confirm / interact (A) | `Z` |
| Back / cancel (B) | `X` |
| Game menu (Start) | `Space` |
| Select | `Backspace` |
| Open chat / send message | `Enter` |
| Close chat | `Esc` |

Rebind Chat, Start, and Select in **Personalization → Keyboard bindings**;
**Save key bindings** keeps the choice in this browser across reloads. Chat, Start, and
Select require different keys. Enter still sends messages while typing in chat.
Nearby Say messages appear briefly above the speaking trainer in 2D and 2.5D.
Whispers and global channels stay in the chat panel.
Phones show a D-pad, A/B, Start and Select below the game. Hold or slide on the
D-pad to move; Start opens the game menu and skips the opening movie. Rotate the
phone for controls beside the game. The same controls work in 2D, 3D and TUI;
standard gamepads are also supported.
Click or press a key in the game to enable browser audio.

Use the **2.5D** control to switch the overworld renderer. Battles and menus
keep their 2D presentation. Camera zoom and rotation are available in 2.5D.

Browser identities are created automatically. Save through the game's normal
menu; saves remain in that browser. Clearing browser storage removes the local
identity and saves.

Multiplayer ratings, the user directory, and leaderboard totals persist in a named Docker volume across container rebuilds
and restarts. Keep the signing secret in `.env` unchanged so returning players
can reuse their identities. Active connections, chat history, and in-progress
matches are temporary; players reconnect after a server restart. See
[persistence and cleanup](docs/deployment.md#persistence-and-cleanup).

### Multiplayer

Face a nearby player and press A, or click their name in chat, to choose
Battle, Trade, or Whisper. Invitations can be accepted or declined in the game.
Players must use compatible game packs to share a world.

Chat supports nearby Say, map-wide General, Trade, Looking for Group,
private whispers, and custom channels. Type `/help` for available commands.

The compact community window has **Chat**, **Social**, and **Leaderboard** tabs.
Social lists every trainer known to this server, including offline users, with
search and live connection status. Click an online name to whisper; battle and
trade invitations require the trainer to be nearby.

Leaderboards rank PvP wins and completed battles, PvE wins and completed battles,
completed player trades, and highest combined level of the six non-egg party
members. Ties share a rank. PvP results and trades require confirmation from both
participants; cancelled sessions do not count. PvE follows the game's encounter
results (including captures as wins), excludes tutorials and link battles, and
persists in normal saves. Save restores cannot reduce or double-count a trainer's
recorded totals. Tracking and the user directory begin as players use this update;
historical battles and trades were not stored.

### Desktop play

Build the native client with the 3D overworld renderer enabled:

```sh
git lfs install --local
git lfs pull
cargo build --release --locked -p crystal-bevy --features voxel-view
```

The executable is `target/release/crystal-bevy` (`crystal-bevy.exe` on Windows).
Press `F3` during play to switch between 2D and 3D. Game content is compiled
with the Rust `pack_core` command in [Game content setup](docs/game-content.md).

The 3D renderer includes material-specific highlights, sky reflections, filtered
detail textures and camera-fitted shadows. Native hardware defaults to the high
quality preset with SMAA Ultra, supported SSAO and HDR; browsers use balanced
quality. Character paintings use filtered mipmaps, and authored 3D textures
use anisotropic filtering for clearer oblique views and less texture shimmer.
See [render quality settings](docs/render-quality.md) for presets and fallbacks.

In the 3D overworld, the first Pokémon in the party follows the trainer's
trail. Reordering the party changes the companion, and it stays with the
trainer across map transitions. Surf shows the trainer seated on the Pokémon
that used the move. That Pokémon remains the mount when the party is reordered;
when it is the lead, it accompanies the trainer by carrying them.
The rider follows the Pokémon's animated back through turns and swimming bob,
with a seated pose fitted to its body width.
This is the sole 3D Surf presentation: the Pokémon recorded by the field move
carries the trainer. Pokémon retain their pack's Pokédex dimensions in every
overworld role; mounting never enlarges or shrinks them to fit the rider.

In 3D, stop on a chair, bench, or stool, or face an empty seat directly ahead.
After a short pause, the trainer sits automatically. Moving away makes them stand.
Human NPCs placed on recognized seats use the same seated pose. The shared animation
blends the hips, legs, hands, and body position smoothly, including interrupted transitions.
This animation preserves the game’s movement and collision rules.

For native play, install the Rust toolchain specified in `rust-toolchain.toml`
and your platform's Bevy build dependencies. Launch with the generated pack:

```sh
cargo run --release --locked -p crystal-bevy --features voxel-view -- \
  --pack content-packs/core-modular.crystalpack \
  --save-path content-packs/player.crystalsave
```

Add `--load-save content-packs/player.crystalsave` to load that save.
3D uses natural movement by default for every input device: walk diagonally,
stop between tiles, and use partial left-stick tilt for slower movement.
Arrow keys, WASD, the D-pad, and the left stick follow the camera, including
while it rotates and in first person. Combine two directions for diagonals
at the same walking speed. Collision, encounters, doors, and
scripts still use the shared production controller. 2D and the text client
retain their normal tile controls. See [natural movement](modpacks/natural-movement/README.md)
for the optional pack builder and opt-out setting.

Native controls use `Enter` for Start, alongside the arrow keys, `Z`, `X`,
and Right Shift. Hold B (`X`) with a direction to run at twice walking speed;
[running shoes](modpacks/running-shoes/README.md) are enabled by default.

The [competitive battle mod](modpacks/competitive-battles/README.md) adds a
physical/special category for each move, a move inspection board, and a
trainer AI that plans attacks, recovery, and switches through the Rust battle
engine. Six trainer-class profiles share the same search strength, with distinct
styles for pressure, setup, disruption, endurance, switching, and balanced play.
The pack initializer and hosted pack preparation enable it by default.
Its builder can also create a separate ignored pack from existing local content.

Standard packs also enable [Wild Challenge](modpacks/wild-challenge/README.md):
wild Pokémon scale with your team, receive legal complementary movesets, and
use the competitive planner. Native, browser, and terminal/MCP builds share
these defaults. Existing external content can be refreshed with the Rust
`pack_default_battles` builder documented in the content guide. These rules
change pack identity; use a new save for a newly upgraded pack.

The refreshed local browser game is available at
<http://localhost:3006/?multiplayer=off>. Its prepared runtime can be restarted
with `just serve 3006 .local/default-battles-runtime`.

Native Bevy enables gamepads and responsive window sizing by default, starting
with an initial window up to 1280×800, fitted to the display’s DPI and size. Steam Deck and other handhelds can use their standard gamepad or
Steam Input's gamepad output; keyboard mappings remain available alongside it.

| Native controller action | Standard / Xbox button |
| --- | --- |
| Move | D-pad or left stick |
| Confirm / interact | A (south) |
| Cancel / run while moving | B (east) |
| Game menu / Select | Start / Back |
| Toggle borderless fullscreen | X (west), or keyboard `F11` |
| Toggle optional 2.5D view | Y (north) |
| Steer 3D camera | Right stick (yaw and pitch), or hold either mouse button and drag |
| Cycle close orbit / wide orbit / first person | Select / Back, or keyboard right Shift |
| Rotate 3D camera continuously | Hold LB / RB, or keyboard `Q` / `E` |
| Zoom optional 2.5D camera out / in | LT / RT |

Drag the game to rotate and tilt: use either mouse button on desktop, or the
left button in the browser. Two-finger trackpad scrolling rotates horizontally
and zooms vertically; pinch zooms. Gamepad Select / Back cycles **close orbit
(150%) → wide orbit (75%) → first person**, aligning first person with the
trainer's facing direction. The right stick looks around in first person.
Touch dragging rotates horizontally and zooms vertically in orbit view, and
looks vertically in first person; two fingers pinch to zoom.

In the 3D overworld, forward follows where the camera looks and left/right
strafe across the view with keyboard, D-pad, or stick. The first-person preset switches to the trainer’s eye level and hides
the local player model. Menus and 2D retain the original Select action.

Sticks use a deadzone; menus and tile movement keep stable cardinal selection. Disconnecting a controller
releases its inputs; a docked controller can take over from an idle built-in pad.
After focus loss or a native overlay, release the controls before resuming.
Linux source builds also need the development package for `libudev` (for example,
`libudev-dev` on Debian/Ubuntu), used by the native gamepad backend.

Wide screens dock field dialogue and panels in a right-hand column. Portrait
screens place dialogue at the bottom center. Both 2D and 2.5D views use this
responsive layout, with smaller insets to keep panels on small screens.

### Browser agent play

In browsers that implement the current WebMCP `document.modelContext` API,
the game registers tools to observe the current presentation, read player and
party status, inspect visible surroundings and input flow, press original Game
Boy buttons, and initiate the existing multiplayer interactions. Open **How to
play → Agent tools** to see whether registration succeeded.

The WebMCP surface follows the same information boundary as a human player. It
returns on-screen text and menus, inspectable trainer/party state, and relative
positions of people visible in the viewport. It does not expose walkthroughs,
objectives, event or engine flags, hidden objects, absolute world coordinates,
map dimensions, collision data, or pathfinding. Every game action still goes
through the normal input system, and a human keypress cancels agent input.

## Development

The game uses Bevy 0.20 and Rust 1.99.0, pinned in `rust-toolchain.toml`.
Shared dependency versions and local crate paths live in the workspace
`Cargo.toml`; each renderer selects its own Bevy features.

Cargo defaults to two build jobs to bound memory use. Development builds keep
incremental game-crate caches, while tests compile game code without optimization
and reuse the optimized engine dependencies. The local allocator patch keeps the
same settings in both profiles so switching to tests does not rebuild Bevy.
Release optimization is unchanged. Use `-j` or `CARGO_BUILD_JOBS` to override
parallelism for the build machine; keep a stable target directory and feature
set for each ongoing task instead of creating a fresh cache per command.
Serialize memory-intensive Cargo builds on shared hosts: separate target
directories do not isolate compiler memory or prevent swap thrashing.
Use the default test profile for regressions: passing `--profile wasm-dev`
selects the playable renderer's optimization settings instead.
Radio oracle replay cases are opt-in with `--ignored` and
`CRYSTAL_RADIO_ORACLE_ROOT` pointing to the external source/capture workspace.
Normal builds and self-contained radio tests require no oracle captures.

For optional independent PC and Mailbox LCD comparisons, see the
[menu reference capture guide](docs/menu-oracle.md). Its emulator environment
and generated references remain external to the repository and game bundle.

Run the graphical client with the optional Bevy development tools and a local pack:

```sh
cargo bevy-dev -- \
  --pack content-packs/realtime-clock.browser.crystalpack
```

`F8` shows or hides FPS diagnostics, `F9` toggles the frame-time graph within
that overlay on native builds, and `F10` toggles UI layout outlines.
The browser supports the FPS counter; its WebGL2 renderer cannot display the graph.
Bevy's 3D render-buffer overlay
uses `F1` to cycle modes and `F2` to cycle opacity. Overlays start hidden.
The feature also enables Bevy's development assertions. Compiled game packs
require a restart after rebuilding. These tools are
opt-in for the graphical client and work with both renderer modes. Normal
desktop, browser, and terminal builds do not enable them. Use `cargo bevy-check`
to check the production graphical client, or `cargo bevy-test` to run its library
tests. These checks enable the 3D renderer without optional development overlays.
Use `cargo bevy-tools-check` to also check the development tools and location
capture examples.
Existing `F3` view,
`F6` capture, and `F11` fullscreen controls remain available.

Native state and schedule diagrams are included in `dev-tools`. Enable file
export explicitly when launching the actual graphical client:

```sh
CRYSTAL_BEVY_DIAGRAMS=target/bevy-diagrams cargo bevy-dev -- \
  --pack content-packs/realtime-clock.browser.crystalpack
```

Open `target/bevy-diagrams/main-index.md` or `render-index.md` for schedule
diagrams with Mermaid previews. Each schedule also has Graphviz ordering and
component/resource-access DOT files; the JSON contains query filters,
set run conditions, conflicts, and dependencies inserted by schedule build
passes. Solid arrows represent
ordering, dashed arrows represent set membership; access graphs label reads,
writes, and conflicts. The currently executing collector schedules (`Main`
and `ExtractSchedule`) are excluded. Render diagrams refresh as Bevy builds
additional schedules, including per-view schedules; `F7` refreshes the main
schedule export. No Graphviz installation is required for Mermaid previews.

`observed-state-flow.md` and `.mmd` record the controller's visible phases and
menus as you play. They show only transitions observed during that session,
not every possible game transition. The controller remains authoritative;
these tools read its presentation cache and never dispatch input. File export
is native-only and disabled unless `CRYSTAL_BEVY_DIAGRAMS` is set. All output
under `target/` is ignored and should be removed after use.

The workspace is organized around focused Rust crates:

| Path | Responsibility |
| --- | --- |
| `crates/crystal-core` | Game state, timing, battles, and world rules. |
| `crates/crystal-assets` | Compiled game data and content packs. |
| `crates/crystal-audio` | Native Rust synthesis, source linking, and audio auditing. |
| `crates/crystal-net` | Multiplayer protocol and transport. |
| `crates/crystal-bevy` | Desktop and WASM game client. |
| `crates/crystal-render-api` | Shared presentation snapshots. |
| `crates/crystal-tui` | Native text snapshots and terminal UI. |
| `crates/crystal-voxel-view` | Optional 2.5D renderer. |
| `crates/crystal-web-server` | HTTP hosting and multiplayer relay. |
| `web-client` | Browser page, controls, thin WASM audio worker, and chat UI. |

Check the server and run the browser session tests (the latter require Node.js):

```sh
cargo check --locked -p crystal-web-server
npm ci
npm run test:browser
```

The production Dockerfile builds the server and WASM client from the same
revision. Keep protocol changes coordinated across both. Generated game packs
should be regenerated using the canonical exporter, not edited manually.
Audio programs come from your separately supplied pack; the Rust exporter
is in this repository. See [Native Crystal audio](audio/README.md) for export and regression checks.
No TypeScript or JavaScript build tool is required to synthesize or export audio.

## Contributing

Bug reports should include the revision, platform or browser, reproduction
steps, and any relevant logs. For gameplay differences, describe the expected
Pokémon Crystal behavior and what happened instead.

Keep changes focused and add regression coverage for behavior fixes. Include
screenshots for visual changes and the checks you ran in pull requests.

## Further reading

- [Meshtastic battle/trade profile](modpacks/meshtastic/README.md)
- [Flygon modpack](modpacks/flygon/README.md)
- [Native Rust text renderer and TUI](modpacks/text-tui/README.md)
- [Minnesota caves: Crystal reference layouts and native verification](planning/minnesota/caves/johto-quality-v01/REVIEW.md)
- [Grand Marais town prototype and fast 2D review loop](planning/minnesota/playable/README.md)
- [Hand-authored North Shore route, native overlay and review evidence](planning/minnesota/routes/MN_R11.md)
- [Hand-authored Gunflint Trail, native overlay and production 2D review](planning/minnesota/routes/MN_R12.md)
- [Hand-authored League Approach, eighteen native chunks and Cedar Shelter](planning/minnesota/routes/MN_R13.md)
- [Hand-authored Headwaters Trail, nine native chunks and wooden-rail crossing](planning/minnesota/routes/MN_R14.md)
- [Compact Minnesota River Valley: two timber bridges, optional terrace and cave pocket](planning/minnesota/routes/MN_R03.md)
- [Twin Cities park crossing: protected bridge, battle-or-grass lane and picnic spur](planning/minnesota/routes/MN_R04.md)
- [Compact Prairie Farm Road: westward nursery ledge, drainage bridge and two farm rooms](planning/minnesota/routes/MN_R02.md)
- [Compact Driftless Trail: native uphill ramp, one-way bluff descent and two rooms](planning/minnesota/routes/MN_R01.md)
- [Compact Central Lakes review: one map, one-way ledge, gated dock and lake cabin](planning/minnesota/routes/MN_R15.md)
- [Wild Challenge: player-relative levels and competitive wild movesets](modpacks/wild-challenge/README.md)
- [Trainer rematches with level and moveset scaling](modpacks/trainer-rematches/README.md)
- [Northwoods Pokédex: cries and walking sprites](https://geothite.ryanculligan.com/pokedex/) · [build and deployment notes](docs/pokedex-web.md)
- [Deployment and maintenance](docs/deployment.md)
- [Game fidelity audit](FIDELITY_AUDIT.md)
- [Renderer inspection](RENDER_AT_LOCATION.md)
- [Repeatable CPU/GPU gameplay audit](tools/PERFORMANCE_AUDIT.md)
- [Operation profiling](tools/OPERATION_PROFILING.md)

## Alternative frontends

The renderer-independent `crystal-runtime` crate owns game sessions, commands,
snapshots, saves and audio cue resolution. `crystal-audio::pcm` decodes the same
packed sounds for custom audio devices. See [the frontend assessment and integration
guide](docs/FRONTEND_ARCHITECTURE.md) for the headless example and remaining UI
controller work.

### Minnesota harvests and frozen lakes

The [berry and apricorn runbook](planning/minnesota/fruit-groves/README.md)
records the renewable fruit-tree overlay, including twenty berry stops across
the two twenty-trainer Mississippi routes and harvests throughout Minnesota. The
[frozen-water overlay](planning/minnesota/regions/ice/FROZEN-WATER.md) carries
those onto the current compact geography with slippery inland lakes and
solid Crystal Ice Path rock rims. North Shore remains unfrozen.

### Minnesota fishing guru route houses

The [fishing-guru runbook](planning/minnesota/playable/fishing-rods.md) records
the executed Rust build of the ignored 716-map route-house pack, its
Old/Good/Super Rod progression and fishermen's post-battle directions. All rod
controller regressions passed. The [HM runbook](planning/minnesota/playable/hm-progression.md)
records seven badge-paced handoffs and 18 visit-unlocked Fly hubs. The former
`minnesota-field-progression-v03.crystalpack` lineage is withdrawn: it retains
obsolete northern geometry. New publication must use the canonical compact
coast/Gunflint lineage; the old pack is historical, not a current build parent.


### Selected Minnesota outdoor review — 2026-10-28

For missing runtime commands after Minneapolis cyclist battles, the
[Minnesota script repair](planning/minnesota/trainers/README.md#missing-battle-return-commands)
provides the verified Rust build/repair commands and the ignored
`content-packs/minnesota-scripts-repaired-r15-v01.crystalpack` successor. It restores
all 18 battle-return handlers while preserving the parent content and audio.

The verified game build is
`content-packs/minnesota-progression-rewards-v04.crystalpack` (705 maps), recorded
as `progression_rewards_revision` in `planning/minnesota/world.json`. Its parent
is the [accepted R11 terrain](planning/minnesota/revisions/route-grass-v01/current-r11-v01/README.md).
Follow `active_local_review_revision` for later terrain successors.

The Minneapolis tool workshop gives a rebuilt bicycle, with native PACK
registration and SELECT riding. Tamarack awards one Master Ball after Boundary
Waters restoration. The Winona aide awards Great Balls, Itemfinder, Exp. Share,
Lucky Egg and Rare Candies for caught Minnesota species milestones. Grand Marais
can recover a failed Machine Part delivery without giving another starter.

Five native reward cases, the earned opening, three byte-clean production stdio
replays and all 377 audio checks pass on this exact identity. All terrain,
actors, warps, teams and encounters are preserved; 701 parent map modules are
exact. The [full story](planning/minnesota/revisions/current-story-v01/README.md),
complex lairs, six rival battles, 24 added grunts and written League remain.

```sh
./target/northstar-story/wasm-dev/geothite play content-packs/minnesota-progression-rewards-v04.crystalpack
```

Use this newly built client for the regional caught-count query. See the
[verified reward build/run receipt](planning/minnesota/revisions/comprehensive-game-v01/README.md)
and [full completion requirements](planning/minnesota/revisions/comprehensive-game-v01/requirements.json).
The comprehensive-game goal remains active: HM/fishing integration, missing
species acquisition paths and an uninterrupted full campaign still need work.
Packs, reached saves and renders remain ignored.

## Minnesota Town Map and Fly

The [native atlas runbook](planning/minnesota/town-map/README.md) records the
one-screen 160×144 Minnesota Town Map, hand-composed from the original Johto
tiles and palettes and shared with Fly. It includes the verified ignored flight
pack, exact build/test commands, production UI renders and a controller flight
to Grand Marais. Existing artwork and audio remain unchanged.
