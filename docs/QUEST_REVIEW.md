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
| Lake of Rage | Access, Red Gyarados battle, Red Scale, Lance, Mr. Pokémon exchange | Pending |
| Rocket Hideout | Statue alarms, traps, passwords, locked doors, rival/Lance, generator battles, reward and exit | Pending |
| Radio Tower | Disguise/director battles, Basement Key, switch maze, Card Key, rescue, Clear Bell | Pending |
| Slowpoke | Tail seller responses, well rescue, Kurt return, repeat state | Rescue callback, town flags, party healing and Kurt warp passed; seller and full battle approach pending |
| Bicycle | Gift, refusal, riding restrictions, shop callback, persistence | Gift/refusal/repeat and three runtime riding/restriction checks passed; callback and persistence pending |
| Fishing | Old/Good/Super Rod gifts, repeats, water targeting, no bite, hooked encounter, restored control | All three gifts/refusal/repeat and eight runtime casting/encounter checks passed; full visual sequence pending |
| Gyms | Every leader's after-talk, badges/TMs, delayed Whitney/Clair awards, repeats | Falkner verified; remaining leaders pending |
| Evolution | Completion, cancellation, Pokédex registration, move learning | Targeted checks passed; visual review pending |
| Hall of Fame | Record, save, credits, title/Continue, postgame unlocks | Record/save/Continue verified; full ceremony and postgame pending |
| Early story | Starter, Mystery Egg delivery, rival, Togepi/Everstone, Sprout Tower | Existing checks identified; coverage review pending |
| Ilex Forest | Farfetch'd chase directions, wrong approaches, return to apprentice, Cut, Charcoal, persistence, shrine event | Chase including backward branch, Cut, Charcoal/repeat and save/load passed; full traversal and shrine event pending |
| Goldenrod | Radio Card, SquirtBottle/Sudowoodo, Kenya mail, flower shop repeats | Pending |
| Ecruteak/Cianwood | Kimono Surf reward, Burned Tower rival/Morty gate, Chuck/Fly, Shuckle | Pending |
| Dragon's Den | Entrance gate, quiz, badge/TM timing, Dratini, Elm/Master Ball | Pending |
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

Shiny follow-up found during source review: `begin_visible_wild_entrance_animation` starts only the frontpic animation; wild encounter sparkle scheduling is missing. Treat this as an open presentation defect, including Battle Scene on/off behavior, sound timing and input release.
