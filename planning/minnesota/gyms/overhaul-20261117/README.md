# Minnesota Gym and cave overhaul

The eight Gyms now have distinct routes, readable physical rules and revised
battle plans. All 18 supporting Gym trainers use native battles on sight; the
eight Gym Leaders wait for an explicit conversation. Optional fights are
optional because their lanes can be avoided, not because a trainer ignores a
player who enters the lane.

This overlay builds on the exact refinement-v02 raw and hosted pack identities.
It preserves the established campaign, leader identities, badge/TM rewards,
Champion rematches and Gym circuit. It does not replay the historical regional
builders. The combined revision also includes the five-cave overlay described
below.

## Mainline comparisons

The comparison uses the actual mainline event and trainer data published by the
pret projects, plus static native-art renders from the locally supplied Crystal
pack. These are design references; the Minnesota rooms have their own layouts.

| Reference | Applied lesson |
| --- | --- |
| [Crystal Violet Gym](https://github.com/pret/pokecrystal/blob/master/maps/VioletGym.asm) and [Azalea Gym](https://github.com/pret/pokecrystal/blob/master/maps/AzaleaGym.asm) | Early Gyms use a small supporting cast and a clear leader encounter. Rochester now teaches one required battle, then offers a short optional fight or a quiet loop instead of extending the introductory checklist. |
| [Crystal Cianwood Gym](https://github.com/pret/pokecrystal/blob/master/maps/CianwoodGym.asm) | Strength boulders make the room's physical obstacle part of its identity. Mankato uses actual ledges and stairs, while Bemidji makes native Cut trees and woodland clearings the course itself. These rooms do not add an imitation Strength puzzle. |
| [Emerald Mauville Gym](https://github.com/pret/pokeemerald/blob/master/data/maps/MauvilleCity_Gym/scripts.inc) | Switches alter real barriers; load scripts restore their state and victory deactivates the puzzle. Minneapolis applies that cause-and-effect relationship to two visible feeder circuits and leaves the course open after Avrin's defeat. |
| [Emerald Petalburg Gym](https://github.com/pret/pokeemerald/blob/master/data/maps/PetalburgCity_Gym/scripts.inc) | Named branches, battle-dependent doors and restored victories make route choice understandable. St. Paul offers archive and workshop routes with distinct opponents; either win qualifies the leader. Brainerd similarly offers a battle pier or an alternate bridge route. |
| [Crystal Blackthorn Gym 2F](https://github.com/pret/pokecrystal/blob/master/maps/BlackthornGym2F.asm) and [1F](https://github.com/pret/pokecrystal/blob/master/maps/BlackthornGym1F.asm) | An action changes traversable terrain and saved events reconstruct that terrain on return. St. Cloud's wind crossings and Brainerd's floating walk use the same principle of persistent physical state, with controls accessible from both banks. They use reversible native routes, not falling boulders. |
| [Crystal Mahogany Gym](https://github.com/pret/pokecrystal/blob/master/maps/MahoganyGym.asm) and its supplied native room art | Ice makes stopping positions and turns the puzzle. Duluth teaches a stop, then a turn, then a combined approach, with visible rough footing and recovery routes instead of unrelated floor materials or an early upper-course bypass. |

Mainline battle identity also comes from move combinations: Bugsy's Fury Cutter,
Whitney's Rollout and recovery, and Morty's sleep/Dream Eater pairing are present
in [Crystal's trainer parties](https://github.com/pret/pokecrystal/blob/master/data/trainers/parties.asm).
The Minnesota revision applies that lesson through legal regional moves,
coverage, support and evolution choices. It retains the campaign's level curve
except for the two reviewed changes listed below.

No ROM, disassembly, exported map catalog or generated pack is added to tracked
source. Room art remains native 2D, assembled from the supplied pack's pixels and
collision definitions.

## The eight rooms

Dimensions below are metatiles; each metatile contains four walking tiles.
The linked notes retain detailed coordinates, flag contracts and authoring
history. Their earlier candidate-status statements describe the authoring stage.

| Gym | Revised course and qualification |
| --- | --- |
| [Rochester — Varel](rochester_notes.md), 6×9 | A compact timber training court replaces the larger hall. Tavel's sightline covers the entry rail; his defeat opens it. Elvik guards the shorter practice lane, while the west loop reaches Varel without a second battle. The existing parcel prerequisite remains. |
| [Mankato — Torven](mankato_notes.md), 8×10 | A rock spine separates connected quarry terraces. Caldren guards the first stair and Jorel the eastern ascent; both wins qualify Torven. Three native one-way drops change the return route, with ordinary stairs available to climb back. The former northwest void becomes part of the course. |
| [Minneapolis — Avrin](minneapolis_notes.md), 7×11 | Lenvi guards the west feeder. Powering it opens the middle gallery containing Jossa and the east control; both feeders must be on to open the court. Each control changes only its own circuit. A permanent one-way service drop returns to the lobby even with both feeders off. After victory, the gates remain open and controls give completion text. |
| [St. Paul — Brannic](st-paul_notes.md), 9×12 | The public lobby forks into Belva's wooden archive and Orven's steel workshop. Four reciprocal pad pairs connect the chambers; either trainer victory opens the shared council grille. The other route remains optional. Distinct room materials and guide text explain the choice. |
| [St. Cloud — Saven](st-cloud_notes.md), 9×11 | Wyrel unlocks two reversible wind crossings. The lower crossing carries the player east; the upper crossing must carry them west to Saven. Each crossing has a console on both banks, with arrows and forced-walk collision changing together. Hallen occupies an avoidable side lane. |
| [Brainerd — Sivren](brainerd_notes.md), 5×8 | Lorla is the required introduction to a compact floating dock course. The winch explicitly offers CENTER, toward Bravel's optional battle, or LEFT, the alternate court approach. Either Bravel's win or LEFT qualifies the court after Lorla. North and south winches share the same state; visible native buoys prevent Surf/canoe shortcuts around the walking course. |
| [Bemidji — Orlis](bemidji_notes.md), 6×10 | Pine boundaries enclose two independent Cut branches. Moslen's west clearing and Elvren's north clearing can be completed in either order; both wins qualify Orlis. Brindel's side nook remains optional. The guide's retryable Cut HM gift is retained, and victory opens a shorter central return. |
| [Duluth — Norek](duluth_notes.md), 7×10 | Connected ice basins teach stopping, turning and combining those skills. Livra opens the upper course; Tordel supplies the second required victory. Anver occupies an optional lap. The last approach still requires ice navigation; Norek's defeat opens the dry west recovery link. Earlier wrong turns retain an eastward return. |

## Sight encounters and stronger parties

Every supporting trainer has an explicit cardinal facing and a one-to-three-tile
sight radius in its companion `*-contract.json`. The installer writes native
`OBJECTTYPE_TRAINER` dispatch and direction overrides, so the wider regional
sightline default cannot erase an optional route. Existing trainer identifiers,
defeat receipts and after-battle conversations remain intact. Leaders retain
their conversation-driven qualification and reward scripts.

The canonical party section covers **26 teams and 50 Pokémon**: eight leaders
and 18 supporting trainers. All members must belong to the active Minnesota
roster, be legal at their authored evolution level, and use distinct moves
available from their level learnset or native TM/HM compatibility.

| Leader | Authored battle plan |
| --- | --- |
| Varel | Nibbin and Crookit establish defense pressure and priority; Crookit's Bite adds a second attacking type. The opening team stays at levels 10/12. |
| Torven | Sodcalf applies physical pressure, Burrbud provides Grass coverage and Growth, and Mudnip combines Dig with Water Gun and Bite at levels 14–16. |
| Avrin | Pipulse opens with Spark and priority, Sparkveil adds Light Screen and paralysis, and Tapacitor combines Thunderbolt with Grass coverage and Growth. |
| Brannic | Patchebee threatens escalating Fury Cutter, Skatuff adds Ice priority, and Trollure combines Steel/Water attacks with rain. The archive/workshop opponents preview different parts of that coverage. |
| Saven | Deepulse brings Psychic coverage and Light Screen, Bankwing adds Dig, and Silohawk combines Flying/Dark attacks with defense pressure and Roar. |
| Sivren | Torrout supplies Fighting coverage and Water priority, Bankmaw adds Ground pressure, and Hatchrush pairs Surf with Signal Beam and rain. |
| Orlis | Zizalith, Burroot, Shedstag and Cyprielle combine Grass attacks with Water, Ground, Ghost and Psychic coverage, plus seed pressure, Growth and recovery. |
| Norek | Four distinct Ice partners add Water, Steel, Ground, Fighting, Psychic and Grass coverage, with priority, Haze, Counter and defensive support. The final team remains levels 42–45. |

Most levels, party sizes, held items and DVs are preserved. The explicit
exceptions are Avrin's ace moving from level 21 to **22**, enabling the reviewed
Tapacitor evolution, and Orlis gaining a **fourth partner**, giving levels
38/38/39/40. Existing Orlis slots retain their held items and DVs; the additional
partner inherits the native lead's values. The builder rejects any other change
to the established level slots.

These are authored tactical improvements, not measured win rates. No ordinary
party, starter matchup or full-campaign balance playthrough was performed.

## Source authority and preservation

- Each room's `.json` provides geometry, native-art references, controls and
  physical gates. Its `*-contract.json` provides sightlines and the leader's
  `all`/`any` qualification predicate. The older full Gym schema contains copied
  party and dialogue fields as template context; those copies do not override
  the canonical revisions.
- [story.json](../../story.json), especially `gym_overhaul.dialogue`, is the
  canonical prose authority. The dialogue installer consumes it while retaining
  the existing leader, gift, reward and rematch wrappers.
- [story-pokemon.json](../../story-pokemon.json), `gym_overhaul_teams`, is the
  canonical authority for all 26 revised initial Gym teams. The full party
  definitions must not be reconstructed from older room templates.
- The [overlay builder](../../../../crates/crystal-assets/src/northstar_gym_overhaul.rs)
  checks the exact parent identities, retains numeric event/trainer constants
  and indexed warp destinations, and checks unrelated game data and runtime
  files against the parent. The combined scope also explicitly repairs two
  retired shore creature actors to active regional species.
- Save compatibility names six approved source identities and exact safe
  arrivals for the eight Gyms and five caves. Runtime migration accepts only
  supported safe states, clears obsolete snapshots only for the declared
  redesigned rooms, and relocates an in-room player to that room's approved
  arrival. Earned victories, badge/TM receipts, inventory and rematch progress
  are retained; this is not an unrestricted pack-hash bypass.

## Combined cave scope

The [five-cave revision](../../caves/overhaul-20261117/README.md) adds a fossil-ridge
branch in Driftless hollow, a connected ore/spring circuit on Route 10, an upper
rim and optional angler at Gunflint grotto, and deliberate rock-stop routes on
both Voyageurs ice floors. Three new sightline trainers and six hidden items
reward exploration while existing story entrances, healing, visible rewards and
encounters remain in place. The cave README records its mainline comparisons,
per-map preservation contracts and static route checks. The combined pack
requires both `northstar-gym-overhaul-v1` and `northstar-cave-overhaul-v1` markers.

## Verification boundary

The [Gym publication validator](../../../../crates/crystal-assets/src/northstar_gym_validation.rs)
checks the native collision graph across reachable control states and retained
victory/Cut receipt combinations. It models solid actors, reciprocal pads,
directional ledges, settled ice slides and forced wind movement. Every trainer
must have a reachable sight encounter; entering an undefeated trainer's ray
forces the abstract battle transition before further movement or control use.
Leader battles remain explicit adjacent actions. The graph requires reachable
controls, a qualified leader route and a return from every reachable state,
rejects exposed walking-to-water bypasses, and checks that operating a control
does not remove the player's footing.

Separate source-art reviews inspect whole rooms and 160×144 player views using
the supplied native pixels. Builds, data checks and pure migration tests provide
additional evidence, but neither the graph nor an image executes a battle,
controller input, Cut menu or live save continuation. Battle wins and Cut use
are abstract transitions in the geometry model.

Following the user's explicit instruction, this work uses no gameplay replay or
game/client launch for verification. It therefore does not claim a played
campaign, demonstrated live input behavior or measured combat difficulty. This
document records the authored scope and validation contracts; selected-pack,
native/browser build and deployment identities belong to their release records.
