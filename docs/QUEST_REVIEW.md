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
| Medicine | Pharmacy prerequisite, SecretPotion gift, refusal, Amphy healing, Jasmine departure and gym availability | Request, gift, refusal, consumption, healing and departure passed; pharmacy negative case and save/reload pending |
| Suicune | Burned Tower release, Raikou/Entei initialization, Cianwood/Eusine, Route 36/42 sightings, Radio Tower Clear Bell, sages, Tin Tower battle and aftermath | Burned Tower release and Clear Bell gate passed; Tin Tower victory aftermath, route sightings, and Eusine departure/non-repeat passed; save/load also passed; capture and complete Radio Tower/sage chain pending |
| Lake of Rage | Access, Red Gyarados battle, Red Scale, Lance, Mr. Pokémon exchange | Defeat aftermath, Red Scale, Lance refusal/acceptance and Mart scene unlock, Exp. Share trade/refusal/repeat, full-pocket retry and save/load pass; access/traversal and capture/loss aftermath remain pending |
| Rocket Hideout | Statue alarms, traps, passwords, locked doors, rival/Lance, generator battles, reward and exit | Camera 1a two-grunt continuation, switch disabling all eight trigger positions, three trap species/repeats, both password doors and persisted floor changes, all three Electrode pairs, Whirlpool/town unlocks and save/load pass. Full traversal, other active cameras/traps, remaining approach variants, Lance's initial password scene, capture/loss and visual/audio timing remain pending |
| Radio Tower | Disguise/director battles, Basement Key, switch maze, Card Key, rescue, Clear Bell | Fake director/Basement Key, warehouse Card Key/repeats, key-gated shutter, final executive/Clear Bell, town restoration and save/load pass. Switch ordering/refusal/emergency/reset and door tiles pass after fixing stale map overrides. Disguise, basement entry, rival, full traversal and visual timing remain pending |
| Slowpoke | Tail seller responses, well rescue, Kurt return, repeat state | Rescue callback, town flags, party healing and Kurt warp passed; seller accept/refuse, unchanged money/inventory and save/load pass; full battle approach pending |
| Bicycle | Gift, refusal, riding restrictions, shop callback, persistence | Gift/refusal/repeat and three runtime riding/restriction checks passed; real gift through mileage threshold, visible shop call, no repeat and save/load pass after fixing the enable flag; full 1,024-step traversal remains pending |
| Fishing | Old/Good/Super Rod gifts, repeats, water targeting, no bite, hooked encounter, restored control | All three gifts/refusal/repeat and eight runtime casting/encounter checks passed; full visual sequence pending |
| Gyms | Every leader's after-talk, badges/TMs, delayed Whitney/Clair awards, repeats | Falkner, Bugsy, Morty, Chuck, Jasmine and Pryce terminal-battle reward continuations, repeat dialogue and save/load pass; Whitney post-victory crying state, Bridget scene, delayed badge/Attract, repeat and save/load pass; Clair quiz, delayed badge/TM, return-visit Dratini, repeat dialogue and save/load pass; Kanto leaders remain pending |
| Evolution | Completion, cancellation, Pokédex registration, move learning | Targeted checks passed; visual review pending |
| Hall of Fame | Record, save, credits, title/Continue, postgame unlocks | Record/save/Continue verified; full ceremony and postgame pending |
| Early story | Starter, Mystery Egg delivery, rival, Togepi/Everstone, Sprout Tower | Existing checks identified; coverage review pending |
| Ilex Forest | Farfetch'd chase directions, wrong approaches, return to apprentice, Cut, Charcoal, persistence, shrine event | Chase including backward branch, Cut, Charcoal/repeat and save/load passed; full traversal and shrine event pending |
| Goldenrod | Radio Card, SquirtBottle/Sudowoodo, Kenya mail, flower shop repeats | Pending |
| Ecruteak/Cianwood | Kimono Surf reward, Burned Tower rival/Morty gate, Chuck/Fly, Shuckle | Pending |
| Dragon's Den | Entrance gate, quiz, badge/TM timing, Dratini, Elm/Master Ball | Perfect/corrected quiz, delayed badge/TM, Dratini move reward, full-party retry, repeats and save/load pass; entrance/traversal and Elm/Master Ball pending |
| Kanto | S.S. Ticket/ship rescue, Power Plant/Machine Part, EXPN Card, Copycat/Lost Item/Pass, Snorlax, Mt. Silver | Pending |
| Every TM/HM | Every acquisition, compatibility, teaching/replacement/cancel, TM consumption, HM reuse/deletion, field-move badge and location gates | Overworld first: all pickups/gifts/shops/rewards and Cut/Fly/Surf/Strength/Flash/Whirlpool/Waterfall plus Headbutt/Rock Smash/Dig, badge gates, obstacles, map transitions and restored control. Teaching/battle checks remain in scope. Cut/Flash/Surf/Waterfall commit timing and Surf/Whirlpool prompt checks passed; comprehensive audit pending |
| Other item/side quests | Moomoo healing, Itemfinder, Apricorns, Ruins puzzles, trades, Silver/Rainbow Wings and legendary gates | Pending |

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
