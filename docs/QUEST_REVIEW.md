# Quest and presentation review

This is the working review checklist, not a claim that every route has passed.
The regression fixtures position a player at each quest leg; dialogue, script
continuations, and rewards then run through the real runtime and visible shell.
Full normal-input traversal and visual review remain separate checks.

## Shipped fixes: 44a31298

- Whiteout: normal battle loss cleanup, one recovery sequence, restored input.
- Battle text catch-up, presentation snapshot reuse, HP handoff, EXP visibility.
- Trainer defeat portrait: full width, centered in the opponent slot, enemy HUD hidden.
- Trainer approach facing and phone-number acceptance/refusal.
- Gym badge state synchronized with script flags; repair old flag-only saves.
- Falkner badge, TM, repeat dialogue, and save/load regression.
- Hall of Fame record saved before credits; title and Continue progression.
- Evolution completion, cancellation, registration and move-learning checks passed.

44a31298 is deployed: rebuilt container is healthy and a real browser battle interaction passed. Opponent-name bounds pass in the native renderer; the mobile browser top row matches all 9,216 compared native pixels. No clipping fix was needed.
Hall of Fame currently uses the existing abbreviated presentation; a full visual
team-induction ceremony has not been established by these tests.

## Quest coverage still in progress

For each chain, check prerequisites, refusal/cancellation where applicable,
rewards and item consumption, continuation after battles, repeat interactions,
map reload, and saved progress. Check visual placement and restored control at
animated boundaries. A successful isolated opcode is insufficient evidence for
a complete quest.

| Chain | Required checks | Status |
| --- | --- | --- |
| Medicine | Pharmacy prerequisite, SecretPotion gift, refusal, Amphy healing, Jasmine departure and gym availability | Request, pharmacy prerequisite/shop exit, gift, refusal, consumption, healing, departure and save/reload pass; post-healing pharmacy revisit gives no duplicate; full traversal and visual/audio review pending |
| Suicune | Burned Tower release, Raikou/Entei initialization, Cianwood/Eusine, Route 36/42 sightings, Radio Tower Clear Bell, sages, Tin Tower battle and aftermath | Burned Tower release and Clear Bell gate passed; Tin Tower victory aftermath, route sightings, and Eusine departure/non-repeat passed; save/load also passed; capture and complete Radio Tower/sage chain pending |
| Lake of Rage | Access, Red Gyarados battle, Red Scale, Lance, Mr. Pokémon exchange | Defeat aftermath, Red Scale, Lance refusal/acceptance and Mart scene unlock, Exp. Share trade/refusal/repeat, full-pocket retry and save/load pass; access/traversal and capture/loss aftermath remain pending |
| Rocket Hideout | Statue alarms, traps, passwords, locked doors, rival/Lance, generator battles, reward and exit | Camera 1a two-grunt continuation, switch disabling all eight trigger positions, three trap species/repeats, both password doors and persisted floor changes, all three Electrode pairs, Whirlpool/town unlocks and save/load pass. Full traversal, other active cameras/traps, remaining approach variants, Lance's initial password scene, capture/loss and visual/audio timing remain pending |
| Radio Tower | Disguise/director battles, Basement Key, switch maze, Card Key, rescue, Clear Bell | Fake director/Basement Key, warehouse Card Key/repeats, key-gated shutter, final executive/Clear Bell, town restoration and save/load pass. Switch ordering/refusal/emergency/reset and door tiles pass after fixing stale map overrides. Disguise, basement entry, rival, full traversal and visual timing remain pending |
| Slowpoke | Tail seller responses, well rescue, Kurt return, repeat state | Rescue callback, town flags, party healing and Kurt warp passed; seller accept/refuse, unchanged money/inventory and save/load pass; full battle approach pending |
| Bicycle | Gift, refusal, riding restrictions, shop callback, persistence | Gift/refusal/repeat and three runtime riding/restriction checks passed; real gift through mileage threshold, visible shop call, no repeat and save/load pass after fixing the enable flag; full 1,024-step traversal remains pending |
| Fishing | Old/Good/Super Rod gifts, repeats, water targeting, no bite, hooked encounter, restored control | All three gifts/refusal/repeat and eight runtime casting/encounter checks passed; full visual sequence pending |
| Gyms | Every leader's after-talk, badges/TMs, delayed Whitney/Clair awards, repeats | Falkner, Bugsy, Morty, Chuck, Jasmine and Pryce terminal-battle reward continuations, repeat dialogue and save/load pass; Whitney post-victory crying state, Bridget scene, delayed badge/Attract, repeat and save/load pass; Clair quiz, delayed badge/TM, return-visit Dratini, repeat dialogue and save/load pass; all eight Kanto leader reward/repeat/save continuations pass; full access/traversal, combat and remaining presentation checks are pending |
| Evolution | Completion, cancellation, Pokédex registration, move learning | Targeted checks passed; visual review pending |
| Hall of Fame | Record, save, credits, title/Continue, postgame unlocks | Record/save/Continue verified; full ceremony and postgame pending |
| Early story | Starter, Mystery Egg delivery, rival, Togepi/Everstone, Sprout Tower | Existing checks identified; coverage review pending |
| Ilex Forest | Farfetch'd chase directions, wrong approaches, return to apprentice, Cut, Charcoal, persistence, shrine event | Chase including backward branch, Cut, Charcoal/repeat and save/load passed; full traversal and shrine event pending |
| Goldenrod | Radio Card, SquirtBottle/Sudowoodo, Kenya mail, flower shop repeats | Radio Card refusal, all five wrong answers, success, repeat, save/load and Radio-tab unlock pass; Floria prerequisites, SquirtBottle badge gate/gift/refusal, Sudowoodo victory continuation, Rock Smash/repeats and save/load pass; full traversal, remaining outcomes and visual/audio review pending; Kenya mail pending |
| Ecruteak/Cianwood | Kimono Surf reward, Burned Tower rival/Morty gate, Chuck/Fly, Shuckle | All five Kimono trainer battle continuations, Surf prerequisite/reward/repeat and save/load pass; Chuck dialogue/battle continuation, badge/TM, wife's Fly gift/repeat and save/load pass; full combat/traversal and visual review pending; other chains pending |
| Dragon's Den | Entrance gate, quiz, badge/TM timing, Dratini, Elm/Master Ball | Perfect/corrected quiz, delayed badge/TM, Dratini move reward, full-party retry, repeats and save/load pass; entrance/traversal and Elm/Master Ball pending |
| Kanto | S.S. Ticket/ship rescue, Power Plant/Machine Part, EXPN Card, Copycat/Lost Item/Pass, Snorlax, Mt. Silver | Oak/Mt. Silver 8/15/16-badge gate, repeat and save/load pass; live Oak assessment/goodbye/movement passes. Other Kanto chains and Route 28 traversal remain pending |
| Every TM/HM | Every acquisition, compatibility, teaching/replacement/cancel, TM consumption, HM reuse/deletion, field-move badge and location gates | Overworld first: all pickups/gifts/shops/rewards and Cut/Fly/Surf/Strength/Flash/Whirlpool/Waterfall plus Headbutt/Rock Smash/Dig, badge gates, obstacles, map transitions and restored control. Teaching/battle checks remain in scope. Cut/Flash/Surf/Waterfall commit timing and Surf/Whirlpool prompt checks passed; comprehensive audit pending |
| Other item/side quests | Moomoo healing, Itemfinder, Apricorns, Ruins puzzles, trades, Silver/Rainbow Wings and legendary gates | Moomoo prerequisite/refusal, seven berries, Snore/repeats, milk sale/rejection/retry and save/load pass after two script fixes; Moomoo slow-cry worker PCM, dialogue close and movement verified in production; remaining side quests pending; milk price verified in production |

## Renderer work

See FRONTEND_ARCHITECTURE.md for the shared runtime/audio assessment and remaining
frontend controller extraction. The modern 3D experiment remains on the separate
`feat/new-bark-3d` branch. It is not part of the production quest fixes.

TM/HM source coverage identifies 56 machines with a mapped pickup, gift or shop.
TM09 Psych Up is the original Time Capsule exception ([TM/HM reference](https://www.serebii.net/crystal/tmhm.shtml)); its definition exists, but normal overworld acquisition is not expected. Time Capsule transfer verification remains separate.

The runtime field-move tests initially could not read obsolete loose JSON paths.
Their fixtures now load the shipped pack. All 14 field-move checks and seven
Strength/Flash/Headbutt/Rock Smash/Sweet Scent checks pass. These are targeted
runtime checks, not proof of every map and obstacle. Three bicycle, eight fishing,
and four TM/HM teaching/consumption checks also pass.

Overworld animation review explicitly includes walking/running, turning, ledge
jumps, trainer spotting and approach, Farfetch'd and Suicune movement, bicycle,
fishing, field moves, doors, warps and fades. Check frame timing, sprite placement,
audio synchronization, and restored control; scripted-state tests alone do not
establish visual fidelity.

Animation checks passed for alternating walking feet, consecutive high-refresh
steps, Fly retaining map objects without stale frames, and Fly/Rock Smash sound
waits. The visual review and remaining animation cases are still open.

Shiny coverage explicitly includes encounter/gift DV generation and odds, palettes, sparkle animation, Red Gyarados, capture, evolution, storage and save/load preservation. The exhaustive 65,536-DV predicate check and actual Red Gyarados script pass. All 251 species now load normal/shiny front and back artwork: the shared lookup incorrectly changed underscores to hyphens, breaking Farfetch'd, Mr. Mime, Nidoran and Ho-Oh. Unown now selects DV-specific battle art and shares its species shiny palette; species-only catalog art defaults to A. Hatch reveal now uses the party Pokémon's shiny state. Red Gyarados capture, PC deposit/withdrawal and save/load preserve its shiny DVs; shiny Pikachu stone evolution into Raichu and save/load also pass. Other evolution paths, gift/egg generation distribution, Transform and sparkle timing verification remain pending. Seven targeted shiny checks pass; the hatch sprite and Stats screenshot were visually reviewed. These findings do not establish that all shinies work.

Wild shiny entrance now schedules the sparkle-only sequence before the frontpic and encounter text, matching BattleStartMessage ordering. New tests verify eight shine events, no ball-poof effect, input ownership/release and Battle Scene on/off behavior. A rendered sparkle frame was visually reviewed. Existing trainer/wild sliding regressions are also checked; full frame-by-frame comparison with original hardware remains open.

Artwork/hatch fix 80245f29 is deployed; container is healthy and the production browser battle interaction passes. Sparkle fix 01161c00 is also deployed; its container is healthy and a production browser battle interaction passes.

Lake of Rage review exposed three shared progression defects: the auto-runner
spun during sound waits, sound waits required a rendered page even in an empty
text window, and completed item notices could repeatedly consume A without
advancing. The Red Gyarados continuation also attempted to persist a scene on a
map without scene storage. Fixes pass the real-pack defeat/reward/Lance/trade/save sequence and full-pocket retry. Core scene tests (13), sound-wait tests (3), item-notice tests (3), medicine (1), and visible evolution tests (3) also pass. Item notice dismissal now resumes the script before stale dialogue can reappear. Gameplay fixes 1e40a90e are deployed; the rebuilt container is healthy and a production browser battle interaction passes.
The scope still includes access/traversal, capture/loss aftermath, the hideout,
and every other pending quest in the matrix above.

Dragon's Den scripted reward coverage now passes both perfect and corrected quiz
answers, the delayed Rising Badge, the exit TM scene, return-visit Dratini moves,
repeat dialogue/rewards, and save/load. Full-party refusal/retry also passes.
Fixtures stage each map leg and Clair's terminal battle; complete gym/Den walking,
water traversal, and every full-bag reward order are still open.

The quiz fixes initialize the active compiled menu cursor, render only its exact
call-site alias at authored coordinates, give menus A/B ownership, honor disabled
B, and release the active menu on closewindow. The rendered three-answer menu and
underlying question were visually reviewed. New map entry clears the eight
temporary flags before callbacks; battle reload, submenu return, and Continue
preserve them. This matches HandleNewMap in
[warp_connection.asm](https://github.com/pret/pokecrystal/blob/master/engine/overworld/warp_connection.asm)
and its [setup paths](https://github.com/pret/pokecrystal/blob/master/data/maps/setup_scripts.asm).
Flag checks also synchronize the script-value alias read by specials; previously
a wrong quiz answer still awarded the perfect-answer Dratini moves.

Verification: complete Clair reward sequence (both branches), full-party retry,
all twelve map setup paths with permanent-flag preservation, closewindow core
regression, five elevator checks, two vertical-menu checks, three sound-wait
checks, item notice, Whitney, medicine, and map load/refresh checks pass.
Four elevator fixtures were updated to locate the shipped pack. Nine existing
shiny checks also pass; the unverified shiny scope above remains open.
Dragon's Den fix 27412562 is deployed. The rebuilt container is healthy and the production browser battle interaction passes; its screenshot was reviewed. The complete quiz/reward sequence is verified in the native real-pack fixture, not yet through normal browser traversal.

Transform review found the core copied target DVs but the runtime snapshot exposed
only its species. The renderer could revert to the original shiny palette after
the move animation and choose the wrong Unown form. The snapshot now carries both
sides' transformed DVs, and rendering uses them consistently for form and palette.
Tests cover both sides, shiny/non-shiny targets, frames before/at the picture swap,
and the settled state. Runtime snapshot tests verify copied values clear with the
transform state without changing party DVs. The existing core Transform-copy test
passes; ten shiny checks pass, along with the picture-swap timing regression.
Both rendered shiny Unown sides were visually reviewed. This does not establish
all Transform/capture/switch combinations or every gift/egg/evolution path.
Transform rendering fix 5b8f36a1 is deployed via eaf20677. The rebuilt container is healthy, the production browser battle interaction passes, and its screenshot was reviewed. This browser smoke check does not replace the native Transform-specific rendering tests above.

Rocket Hideout coverage now exercises the first camera's two successive trainer
battles, verifies the secret switch disables all eight camera coordinate scripts,
and preserves that state through save/load. Trainer fixtures stage terminal HP;
full combat and the seven other active-camera approach animations remain open.
Three floor traps start their authored Koffing/Voltorb/Geodude encounters, complete
a real final turn, set their individual flags, and do not retrigger. The other
nineteen trap tiles and capture/loss outcomes remain open.

Password tests start from defeated-grunt fixtures, collect both passwords through
actual dialogue, reject the office door with zero or one password, and open it
with both. Murkrow's password gates the transmitter door. Both authored floor
blocks persist through save/load; full normal-input door traversal remains open.
Three Electrode battles clear their matching object pairs, then award Whirlpool,
stop the radio signal, unlock town progression, reset the scene, and expose the
deactivated-transmitter dialogue. The reward and flags survive save/load.
Four real-pack visible-shell tests pass. NPC fixtures now select an adjacent
walkable tile and face toward the target; blindly choosing the tile below an
Electrode placed the player inside a wall. No production hideout changes were
needed for these tested continuations after the preceding shared script fixes.

Hideout scene coverage now passes Lance's B2F healing, the B3F rival shove/exit,
both executive battles' left approach variants and retreats, and Lance's follow,
pacing and generator handoff. Final trainer HP is staged; authored movement,
dialogue, battle continuations and save/load execute through the visible shell.
Lance's initial B3F password exposition, right-side variants and full traversal
remain open.

Radio Tower checks pass the fake director's Basement Key reward/repeat dialogue,
the warehouse director's Card Key/repeat, negative Card Key slot behavior, shutter
opening and persisted map changes. The final executive continuation awards Clear
Bell, restores Rocket/civilian/town flags, sets the Tin Tower gate scene and saves.
Fixtures explicitly enter the Rocket takeover phase; the initial-game shutter
flag is otherwise already set. These are staged quest legs, not full-tower walks.

The underground maze test distinguishes wrong 1-2-3 order from the successful
3-2-1 door history, checks refusal, emergency activation/deactivation, all eleven
door flags, critical floor tiles, save/load and warehouse reset. It exposed stale
changeblock overrides surviving the warehouse's flag reset. Map setup now reloads
base metatiles before MAPCALLBACK_TILES, discarding that map's stale overrides;
callbacks reopen persistent doors from their flags. This matches
[LoadBlockData](https://github.com/pret/pokecrystal/blob/master/home/map.asm).
The reload regression covers warp, battle reload, Continue, submenu and connection
setup. Nine checks pass: reload semantics, the maze, both password-door chains,
room decorations, Cut, Fly, temporary flags and medicine. Earlier scene/reward
checks above also pass. Full normal-input maze traversal and visual timing remain
open. Map-block reload fix f3685a28 is deployed; the container is healthy and the production browser battle interaction passes. Its screenshot was reviewed. This smoke check does not establish full maze traversal.

The borrowed bicycle callback was broken: step counting checked the physical-bit
name STATUSFLAGS2_BIKE_SHOP_CALL_F, while authored shop and phone scripts use
ENGINE_BIKE_SHOP_CALL_ENABLED. The step system now reads and clears that same
engine flag. The real-pack visible-shell regression failed at 1,023 steps before
the fix and now passes gift, actual riding across the threshold, incoming shop
dialogue, dismissal, save/load and no repeat. The fixture stages mileage at 1,023;
it does not ride all 1,024 steps. Five core mileage/poison/service checks pass.
Slowpoke Tail offer checks also pass both answers, unchanged maximum money and
inventory, scene advancement and save/load; the authored seller never sells an
item even when answering yes. No seller production change was needed.

Breeding shiny audit found a production defect: InitEgg discarded the random
high Special DV bit during inheritance, preventing inherited eggs from being
shiny. It also retained the pre-inheritance HP DV, which could fail persisted DV
validation. The implementation now preserves the random Special bit, inherits
the donor's low three bits and Defense, and recomputes the derived HP DV. This
matches [InitEgg](https://github.com/pret/pokecrystal/blob/master/engine/events/daycare.asm).
The new regression failed before the fix and passes all 512 attack/speed/Special
bit outcomes for both Ditto slots and donor Special 2/10 (2,048 generated eggs).
Each setup yields eight shiny outcomes, and all generated DVs serialize and
validate on read. All 24 Day Care checks pass. This proves the tested inheritance
distribution, not random-source uniformity, every parent pairing, or a complete
visible hatch playthrough. Existing eggs are not rerolled by this change.

Bicycle callback fix aff7b40f is deployed. The container is healthy, the production browser battle interaction passes, and its screenshot was reviewed. Breeding fix ed4e7270 deployment has started; production completion remains pending.

The core bred-shiny lifecycle now passes generation, collection, Pokémon JSON
round-trip, the actual overworld hatch boundary, full health, preserved shiny
DVs and post-hatch Pokémon JSON round-trip. This stages egg readiness and the
last hatch cycle; it does not prove normal traversal or the complete save file.
The check exposed core egg initialization clearing the stat modifier map, making
it fail Pokémon validation. Eggs now receive all eight neutral modifiers. The
asset/runtime normalization already rebuilt that map, so this is a shared-core
validity fix rather than evidence of a deployed hatch freeze.
The existing visible hatch test now loads the selected current pack. Two timing
checks pass (hold, wobble/cracks, shell, frontpic, hatch text), and the separate
shiny Unown/hatch art plus save check passes. Its regenerated shiny hatch sprite
was visually reviewed; full-screen normal-input hatch and nickname completion
remain open.

Breeding DV fix ed4e7270 is deployed. The container is healthy, production browser battle interaction passes, and its screenshot was reviewed. Core egg-validity fix 1c5e39e7 is now building for deployment; completion is still pending.

Visible shiny hatching now passes both nickname paths through the native real-pack
input controller. A staged final-cycle shiny Togepi egg hatches after real walking;
A advances the hatch text/animation, B declines naming, or A opens the keyboard
and A/START/A enters and confirms a nickname. Both paths release the controller
back to overworld movement and preserve shiny DVs, egg completion and the chosen
name through a complete save/load. Two tests pass (126.79 seconds). The fixture
sets a valid player identity, as normal new-game setup would; hatching assigns
that identity to the Pokémon. Typewriter completion is accelerated by the test
helper, while animation timing advances through host updates. These checks do
not replace browser/mobile hatch screenshots or the full preceding breeding walk.

The production browser hatch flow now passes at a 390x844 viewport with DPR 2:
walking, hatch animation, nickname decline and restored movement. The isolated
save uses the combined production pack identity; the base-pack save correctly
was not resumed by that build. This is Chromium at phone dimensions, not iOS
Safari or touch-input verification. Native replay of the combined-pack fixture
also passes. Core egg-validity fix 1c5e39e7 is deployed, healthy, and passes the
same browser flow.

Screenshot review exposed the hatchling disappearing during the nickname Yes/No
question, despite functional input checks passing. The original
[HatchEggs sequence](https://github.com/pret/pokecrystal/blob/master/engine/pokemon/breeding.asm)
keeps its frontpic until naming or returning to the map. The renderer now draws
the pending hatchling while the nickname choice is active, preserving its shiny
palette. A regression checks an actual sprite entity with that artwork; the
corrected native composition was visually reviewed. Retention fix 13d8fa3f is
deployed. A fresh production Chromium run at phone dimensions passes hatch,
nickname decline and restored movement, and its nickname screenshot was
visually reviewed: the shiny hatchling stays visible beside the Yes/No choice.
This remains distinct from iOS Safari and touch-input verification.

All eight Kanto leader continuations pass from staged terminal battles: Brock,
Misty, Lt. Surge, Erika, Janine, Sabrina, Blaine and Blue. Each sets its correct
Kanto badge and defeat flag, reaches authored repeat dialogue without starting
another battle, and preserves rewards through save/load. Erika grants Giga Drain
and Janine grants Toxic exactly once; the other six leave the bag unchanged,
matching their source scripts. One parameterized real-pack visible-shell test
covers all eight (70.25 seconds). Leader availability is staged for Misty/Blue;
gym access quests, mazes, full combat, guide/statue branches and visual/audio
comparisons remain separate work. No Kanto production change was needed.

Oak/Mt. Silver review exposed a fatal missing-rating lookup during the lab
conversation. The rating table refers to OakRating01–19 command labels; each
[source text command](https://github.com/pret/pokecrystal/blob/master/engine/events/prof_oaks_pc.asm)
points to its matching _OakRating text already in the pack. The renderer now
resolves that target. All 19 texts pass a real-pack resolution check. A visible
quest regression stages 8, 15 and 16 badges, confirms the corresponding Oak
branches, checks the nearby Mt. Silver blocker before/after unlocking, and
verifies the unlock and repeat dialogue after save/load. All eight selected Oak
checks pass (46.44 seconds), including the title/intro regression after correcting
its obsolete test pack path. This proves the tested quest continuation, not full
Route 28 traversal or fidelity of the assessment's layout, paging and audio.
Rating lookup fix 2c505392 is deployed and its container is running healthy.
Browser verification of that build remains separate from the native checks.

Further visual review found that the single assessment list did not preserve the
original dialogue sequence. Oak's introduction, seen/owned counts, rating header
and rating paragraphs now use the existing source text-page renderer and the
field textbox, including two-line scroll steps. The all-19-ratings regression
checks complete page queues, substituted counts and return to the PC hub only
after the final page. Mt. Silver and the other selected Oak regressions continue
to pass. A screenshot test then exposed a blank-text refresh defect: unchanged
core state could bypass a shell-owned field text update. The retained-dialog
path now includes these PC/Oak boundaries, and the unchanged-frame check also
compares the dialogue key. The existing integrated Pokémon Center PC interaction
passes. The rendered keyboard-input test passes (6.30 seconds), requiring the
complete final congratulations page, visible text entries and live glyph
entities for every page. Counts, heading and final-page native dialogue-layer
captures were inspected; the final text fits within the box. Deployment and
browser review of these additional changes remain pending; assessment fanfare
timing is still unverified.

Oak assessment audio now uses each rating table entry's existing fanfare. It is
queued once when the final source text page finishes printing. An acknowledgement
while that sound is playing retains the final text and resumes automatically
when playback ends, matching JoyWaitAorB followed by WaitSFX. A regression covers
all 19 rating entries, no early playback, no repeat playback, and both retained
text and automatic return to the PC hub after an early acknowledgement. All ten
selected Oak checks pass (55.75 seconds). A separate real-pack audio check passes
(5.12 seconds): every distinct rating fanfare decodes against its canonical PCM
hash and byte/frame count, has no loop and contains non-silent samples. No DSP,
PCM payload or pack data changed. Audible browser verification and deployment of
this fanfare change remain pending. Full scripted assessment handoff timing still
needs a browser check in addition to the direct input and continuation tests.

The full Oak lab conversation exposed an additional handoff defect: automatic
cleanup treated the special's rowless core menu as noninteractive and removed
its shell-owned assessment in the same update. Cleanup now preserves an active
special display until its acknowledgement path releases it. A real scripted
conversation test fails before the fix and passes afterward: the assessment
survives updates without input, then reaches the authored goodbye only after
acknowledgement. All 12 selected Oak checks pass (67.31 seconds), including
Mt. Silver, all rating pages, fanfare ordering and PCM integrity. The prior
paging build 5e35786b is deployed and healthy; this handoff fix and fanfare commit
360d573b still await the next deployment and live browser replay.

The same scripted Oak handoff passes with the combined production pack and
exports an isolated browser save beside Oak (9.83 seconds). Pokémon Center PC
boot/access/shutdown and Day Care intro/party-selection regressions also pass
(6.49 and 6.44 seconds). These checks exercise neighboring special surfaces;
they do not establish completion of every remaining quest.

Medicine review now covers the pharmacy before Jasmine's request: it opens
MART_CIANWOOD, leaves the bag and potion-gift flag unchanged, and returns from
the shop through B input. The real medicine chain now saves/reloads after refusal
and again after healing. The potion remains after refusal, is consumed after
healing, Jasmine's gym/departure flags persist, and a pharmacy revisit opens the
shop without granting another potion. The extended chain passes; the separate
prerequisite/shop-exit test passes in 5.86 seconds. No production change was
needed. Fixtures position the player at each quest leg; this does not claim full
lighthouse travel or animation/audio fidelity.

Deployment c8133496 (including fanfare commit 360d573b) is running healthy with
image sha256:80ec54fd93110dc50e5778ca862fde15dbad7cea966b932745a4f1eeaba97e56.
The live production Oak replay passes using the isolated combined-pack save and
real keyboard input. It turns toward Oak after Continue, holds the assessment
introduction without advancing to goodbye, reads seen/owned counts and the
rating, reaches the goodbye, dismisses the conversation and walks away. No page
errors or reported runtime errors occurred. Introduction, counts, rating and
post-dialogue screenshots were inspected; the rating fits within the textbox and
the final overlay clears. The goodbye capture was taken during its typewriter
reveal, so it is not evidence that every goodbye glyph was displayed at once.
This was headed desktop Chromium at 390x844 CSS pixels with device scale 2;
iOS Safari/touch input and perceptual audio comparison remain unverified.
Medicine test-only commit 87ee0678 is pushed separately and needs no gameplay
redeployment.


## Moomoo Farm script fixes and bounded review

The real Moomoo interaction reproduced a crash at `PlaySlowCry`: `checkevent`
left `_value=0`, and the following `setval MILTANK` updated only `script_value`.
Accumulator-writing variable commands now synchronize both fields. Twelve
script-variable checks, 68 script-runtime checks, and the existing exact-species
cry lookup check pass. The authored checkevent/setval/cry regression now passes.

The healing sequence subsequently reached seven berries, retained progress
across reloads, and awarded Snore once. Milk purchases then exposed missing
`HAVE_LESS` comparison resolution. Script numeric constants now include the
three values from the existing `AmountComparison` implementation; all three Moomoo regressions now pass (75.97 seconds), including the complete
feeding/reward chain and insufficient-money/full-pocket/retry branches.
Milk purchases charge exactly ¥500 only after successful delivery; repeat
interactions preserve the existing bottle. Deployment and the initial sick-cow
browser interaction are verified below.

Presentation gaps remain: the milk dialogue displays an unresolved decimal
price placeholder, and `PlaySlowCry` uses the ordinary visible cry playback
path. Neither slowed-cry audio fidelity nor complete visual traversal has been
verified by these state tests.


The same Moomoo healing/reward chain passes with the production combined pack
(65.12 seconds). An optional ignored save fixture supports checking the initial
sick-cow interaction through browser inputs.

All 13 targeted Bevy shiny regressions pass again after these script changes
(180.53 seconds): all-species art, Red Gyarados script, exhaustive DV predicate,
both walking hatch nickname paths and persistence, retained hatch portrait,
stone evolution/save, Unown forms/palettes, Stats layout/palette, Transform,
and wild sparkle sequencing/input release. This does not cover every evolution
method or every capture/switch/Transform combination.


Moomoo fixes `796b342e` are deployed. The container is running and healthy with
image `sha256:6ab1a471a056ddecbcfae7d40f863281b24002d65f86938d2dc98e5aaf4f97ba`.
A fresh production browser session passed the sick-cow greeting, weak-cry text,
dialogue close and walking into the open aisle, with no page/runtime errors.
The initial movement assertion aimed down into the barn fence; the corrected
run uses the open aisle and verifies changed coordinates. Reviewed screenshots
show complete weak-cry text within its box and the player walking after closure.
This is desktop Chromium at 390×844 CSS pixels/DPR 2 with keyboard inputs;
iPhone Safari, touch controls and perceptual cry fidelity remain unverified.


## Shared text decimal substitution

Runtime text snapshots now resolve RGBDS `{d:CONSTANT}` expressions from the
pack's local/global/currency constants before any frontend renders them. Missing
TM-count and Bug-Catching Contest operands derive from the existing item and
contest catalogs. Both regression checks pass: all 12 affected map dialogue
bodies resolve, and the milk offer renders `fer just ¥500.` (7.96 seconds).
Production screenshot/deployment verification passed as recorded below.

The remaining sick-cow audio difference has been traced to the source
[PlaySlowCry routine](https://raw.githubusercontent.com/pret/pokecrystal/master/engine/events/play_slow_cry.asm):
subtract `0x140` from cry pitch, add `0x60` to cry length, then wait for completion.
The visible audio command currently carries neither override; implementing the
exact synthesis and wait behavior remains open. No audio program was changed
as part of the decimal text fix.


Decimal-text fix `c03b7277` is deployed. The container is running and healthy,
image `sha256:8a4c254c4e53e17ae0e9662e86d0cf35b0316a416d8f7d2e61eafb656776e966`.
A fresh production Chromium session renders the complete milk offer, including
`fer just ¥500.`, and the Yes/No prompt without page/runtime errors. The actual
screenshot was reviewed for legibility and textbox bounds. The first harness
run faced the television; correcting the input to face the farmer on the right
passed. This is keyboard interaction at a mobile-sized viewport, not iPhone
Safari/touch verification.

Slow-cry implementation review additionally found that
`play_cry_for_species` clears `waiting_for_sound_effect` and the visible special
handler combines `PlaySlowCry` with ordinary cries. Exact support must retain
the source sound wait, carry pitch/length through audio preparation and cache
identity, and validate the original bundled MIDI/PCM metadata before deriving
a new synthesis. The existing MIDI decoder retains the cartridge program;
no additional bundled program or generated audio file is required. This work
remains unimplemented.


## Source-parameter slow cry implementation

`PlaySlowCry` now derives the species cry from the existing bundled MIDI
program with the source's 16-bit pitch subtraction and length addition. The
shared Rust decoder validates the ordinary cry's PCM hash/frame metadata
before synthesizing the derived result. Native and browser workers use that
same decoder; synthesis stays off the UI thread. Cache identity includes cry
parameters, and the generic core cry event is replaced rather than played a
second time. The core sound-wait flag and visible sound-wait boundary hold the
script until queued/playing audio completes. No bundled audio inventory or
compiled pack changed.

Three worker tests and both audio/game WASM checks pass. Five Moomoo tests pass
(88.62 seconds), covering healing/reward/purchase continuations plus cry state.
The final two cry regressions pass (8.08 seconds): Miltank's ordinary source
validates at 15,506 frames, hash `c3913779`; the derived cry is 19,198 frames,
hash `db796201`, pitch word `64755`, length `512`, with no loop. Ordinary output
is unchanged, corrupt source hashes fail, cache entries differ, and queued or
playing audio holds the wait. Production worker PCM and browser interaction
verification are still pending. Parameterized playback requires the existing
species MIDI program, available in the shipped browser pack; it does not
approximate a cry from an opaque PCM-only asset.


## Radio Card quest review

The real Radio Tower quiz passes refusal, a wrong answer at each of the five
questions, retry after save/load, the authored correct-answer sequence, award
and follow-up dialogue, repeat suppression, and saved card ownership. Pokégear
page selection excludes Radio before the reward and includes it after reload;
closing the panel leaves dialogue idle. The final regression passes in 26.59
seconds. This positions the fixture beside the NPC and tests the visible shell
and source scripts; full route traversal, screenshots, and quiz audio timing
remain separate review items. No production change was needed for these paths.


Slow-cry fix `c8fd5ff3` is deployed. The container is running and healthy with
image `sha256:d254a3bcc37bad83eedf068746b131e08d90ea9886f8bb3a29b6a56955bca433`.
Two fresh production browser runs pass the Moomoo greeting, weak-cry text,
dialogue close and walking into the open aisle, with no page/runtime errors.
Instrumentation observed one parameterized worker result: pitch `64755`, length
`512`, 19,198 stereo frames, PCM hash `db796201`, exactly matching native output.
The worker received the unchanged base-source metadata (62,024 bytes, hash
`c3913779`) for validation. Screenshots of settled text and the player after
movement were reviewed; final punctuation fits inside the dialogue box.
This is desktop Chromium at a mobile-sized viewport with keyboard input;
iPhone Safari/touch and perceptual comparison on original hardware remain
unverified. The Radio Card regression above is also committed on main.

## SquirtBottle, Sudowoodo and Rock Smash

The real-pack visible-shell quest regression follows the authored
[flower-shop prerequisites](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/GoldenrodFlowerShop.asm)
and [Route 36 encounter/reward scripts](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/Route36.asm).
It checks touching the tree without a bottle, the pre-encounter Rock Smash
conversation, both Floria conversations, missing-badge refusal, the bottle gift
and repeat, declining the water prompt, the actual level-20 Sudowoodo battle,
victory continuation, tree disappearance and walking onto its former tile after
save/load, the one-time Rock Smash
TM and flower-shop after-talk. The bottle remains in the bag. Save/load is checked
before the badge, after refusal, after the battle and after the reward.

The test stages Whitney's badge only after checking its gate (Whitney's own
handoff has separate coverage), positions the player beside each authored NPC,
and stages the wild battle's final turn. It does not establish complete route
traversal, capture/escape/loss outcomes, both Floria approach animations, bag-menu
bottle use, full TM-pocket retry or browser visual/audio fidelity.

The final expanded regression passes (65.29 seconds); `git diff --check` passes.
This change adds regression coverage and documentation; it changes no deployed gameplay.

### SquirtBottle bag dispatch correction

A new real-pack regression reproduced an extra Yes/No question when using the
bottle from the bag. The shared backend dispatched `SudowoodoScript`, whereas the
[original bag effect](https://raw.githubusercontent.com/pret/pokecrystal/master/engine/events/squirtbottle.asm)
enters `WateredWeirdTreeScript` directly. The backend now selects that exact entry
for Route 36's standard tree script, validates it before mutating state, and
preserves custom tree-script dispatch. No audio or content-pack data changed.
Both real-pack regressions pass (120.70 seconds): bag use enters watering without
a second prompt, the visible battle introduction completes, the Run action
returns through the escape aftermath, the tree disappears, and save/load preserves
that state. The bottle remains. The earlier Floria/badge/refusal/victory/TM and
cleared-tile walking regression also passes. Commit c2afcdd4 is deployed; its
container is healthy (image 127c06d8affb). A fresh production Chromium session
loaded the saved Route 36 fixture, opened Pack/Key Items/SquirtBottle/Use with
keyboard input, and showed the complete watering message without another prompt.
The watering screenshot was reviewed; the following dialogue capture shows its
first page, not the entire attack message. No page/runtime errors occurred on
that successful run. Desktop Chromium at a mobile viewport is not Safari/touch
verification. This review exposed the separate invisible-Pack defect below.

### Pack display retention

Production screenshots of Pack → Key Items → SquirtBottle → Use showed the
world while the menu remained interactive; waiting three seconds did not restore
it. Pack was missing from `retained_field_fullscreen_active`, allowing the
world renderer to reuse its display. A native visible-entity regression reproduced
the failure (no full-screen presenter), then passed after retaining Pack alongside
other full-screen field menus. It checks opening, pocket changes, the action menu,
idle updates and closing. Existing Pack Cancel and party/Stats retention tests
also pass (three targeted tests total). The resulting native Pack image was
reviewed for the item list, description and action menu. Browser rollout of this
retention fix 453a6460 is deployed and healthy (image 34e82b2d010e). A fresh
production Chromium session at 390×844 CSS pixels verified the action menu,
return to the item list, closing Pack and walking from (35,10) to (35,12).
All three screenshots were reviewed: the full menu remains visible and centered,
Use/Quit disappears on returning to the list, and the overworld returns on close.
No page/runtime errors occurred. These checks do not establish every Pack flow
or physical Safari/touch behavior.

### Empty Balls pocket Cancel

The browser's earlier `bag:balls has no valid cursor` error was reproduced with
normal Start/Pack/pocket/Cancel input. The empty-Balls branch cleared only the
cursor and left Pack open, causing a renderer/observation error on the next
update. Removing that branch routes its Cancel row through the normal complete
Pack cleanup. A regression now passes for all four empty pockets, checking the
Cancel row before input, closing, idle updates, absence of runtime errors and
successful observation after closing (17.76 seconds). Production rollout remains
in progress in the subsequent deployment; Pack display rollout is now complete.

### Kimono Girls and Surf acquisition

The real-pack visible-shell regression passes all five authored trainer starts
and victory continuations, their individual defeat flags, save/load and repeat
after-talk. The gentleman withholds HM03 before each victory, awards one HM_SURF
after all five, explains it, and gives only repeat dialogue after saving/loading.
The [Dance Theater script](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/DanceTheater.asm)
is the reference. The check passes in 39.36 seconds. It positions the player
beside each NPC and stages terminal trainer victories; it does not establish full
combat, walking through the theater, the female prerequisite wording, visual
placement or Surf teaching/use. Existing field Surf checks remain separate.
The Pack retention regression also passes after the empty-Balls Cancel fix
(6.16 seconds).

### Chuck and Fly acquisition

The real-pack visible-shell check passes the wife's pre-victory dialogue with no
HM, Chuck's actual interaction into battle, terminal-victory continuation,
Storm Badge and DynamicPunch, save/load before visiting his wife, Fly gift and
explanation, and save/load/repeat with exactly one HM_FLY (25.67 seconds).
The [Cianwood script](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/CianwoodCity.asm)
is the reference. This test stages the trainer battle's terminal victory and
positions the player beside each NPC. It does not establish full gym traversal,
combat, HM-pocket capacity failure, Fly teaching/use or visual/audio fidelity.

### Strength gift and teaching

The real-pack visible-shell check passes the Olivine Café sailor's HM_STRENGTH
gift, event flag, save/load, visible HM boot/teach flow with an open move slot,
non-consumption of the HM, persisted learned Strength and repeat conversation
without a duplicate (8.62 seconds). The fixture selects the HM pocket/cursor,
then advances the visible prompt and party selection with the normal input helper.
The [sailor script](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/OlivineCafe.asm)
and [Machop learnset](https://raw.githubusercontent.com/pret/pokecrystal/master/data/pokemon/base_stats/machop.asm)
are the references. The initial Totodile fixture was corrected: its original
Crystal learnset excludes Strength, and the implementation correctly refused it.
Replacement/cancel, a second recipient, boulder movement and browser visual/audio
verification are not established by this acquisition/teaching check; existing
field Strength tests are separate.
