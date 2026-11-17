# Minneapolis — two independent feeder circuits

`minneapolis.json` uses the full `minneapolis-electrical-station-v01.json` schema as its template. Overlay its geometry and authored interactions on selected refinement-v02; retain installed trainer identities/parties, rewards, rematches, native AVRIN portrait/class and all exterior warp indices. Party fields copied into this planning source are templates, not an instruction to replace current balancing.

The 7×11-metatile footprint, arrival `(6,19)` and exits `(6,21)` / `(7,21)` are unchanged. Accepted AVRIN sprite is `SPRITE_MN_LEADER_AVRIN`. A central generator bank separates an entrance feeder alcove from the middle gallery. Opening the west feeder changes the route and reveals JOSSA and the east console. Powering east opens a return shortcut and, with west still on, the leader bridge. The two electrical states remain independently reversible; the old six-console guessing/reset puzzle is removed. Each console and its bridge can be inspected together in a 160×144 viewport.

## Native encounters and controls

| Feature | Position | Contract |
| --- | --- | --- |
| `MN_GYM_3_TRAINER_1` / LENVI | `(3,15)` | Facing left, radius 1. Body and sightline cover the two-tile west approach. |
| `MN_GYM_3_TRAINER_2` / JOSSA | `(10,8)` | Facing left, radius 2. Body and sightline watch the east control stance `(8,8)`; physically reached after the west bridge opens. |
| WEST_FEEDER | `(5,14)` | Stand `(5,13)`, face down. Native solid `facility:02` console. Requires LENVI's existing win. |
| EAST_FEEDER | `(8,7)` | Stand `(8,8)`, face up. Same native console; requires JOSSA's existing win. |
| AVRIN | `(6,3)` | Both independent feeder ON flags qualify. |
| Guide | `(9,19)` | Explains the west-to-middle progression, both-on court condition and permanent drop out. |

Sightlines and `all` / `any` qualification are machine-readable in `minneapolis-contract.json`. Source encodes two existing-schema `routers` with `toggle:true`, two modes each, and three existing-schema gates. ON is the first mode, because the native toggle selects the first mode when neither flag is set; its first activation therefore opens the bridge. Each activation changes one feeder only. No inactive console, guessed neighbor, hidden order or incorrect-choice reset remains. Both unset flags represent OFF initially; ON/OFF modes are mutually exclusive within their own router. Do not clear the other feeder.

Flags:

- West: `EVENT_MN_GYM_3_V2_WEST_FEED_OFF` / `EVENT_MN_GYM_3_V2_WEST_FEED_ON`.
- East: `EVENT_MN_GYM_3_V2_EAST_FEED_OFF` / `EVENT_MN_GYM_3_V2_EAST_FEED_ON`.

Changed blocks `[metatile x,y,off glyph,on glyph]`:

- `[2,5,"A","f"]`: west ON; receipt `EVENT_MN_GYM_3_V2_WEST_BRIDGE_OPEN`.
- `[4,5,"B","f"]`: east ON; receipt `EVENT_MN_GYM_3_V2_EAST_BRIDGE_OPEN`.
- `[2,2,"C","f"]`: west ON AND east ON; separate terrain receipt `EVENT_MN_GYM_3_V2_COURT_OPEN`.
- A/B/C use existing native `facility:0b` electric-grid art with the schema's explicit `barriers` override to four WALL quadrants. The source block's default FLOOR collision must not leak through. `f` is native `game_corner:01` with four FLOOR quadrants.
- The root computes canonical `EVENT_MN_GYM_3_COURT_OPEN` for leader scripts. The V2 receipt is distinct and must not overwrite that predicate.

| West | East | West bridge | East bridge | Court bridge |
| --- | --- | --- | --- | --- |
| off | off | closed | closed | closed |
| on | off | open | closed | closed |
| off | on | closed | open | closed |
| on | on | open | open | open |

Before victory, both switches are reversible. The west control is below its bridge; the east control is in the middle gallery. A native one-way service drop at metatile `(5,5)` uses `tower:3d`: `[WALL,HOP_DOWN,WALL,WALL]`. The player drops from `(11,10)` to FLOOR `(11,12)` in the entrance's east lane. It cannot be climbed backward and always provides an exit, including the otherwise trapping case of turning EAST off when WEST is already off. Ordinary uphill returns cross one of the powered bridges; with both open the shortest return is eight geometric steps. Re-entering applies saved feeder state. Contract `victory_opens_gates:true` requests permanent access after the badge, with completion text rather than further mutation.

All source text is ASCII and <=18 characters per line. Suggested additional root-installed conditional feedback after a toggle:

- Both on: `Both feeders hum.` / `Court bridge open.`
- After badge: `AVRIN left both` / `feeders running.`

These are dialogue drafts, not unknown JSON fields. The source prompt deliberately does not ask a yes/no question: the existing toggle API changes state immediately when the player activates the console.

## Static review

Final native original/2× whole rooms and arrival/choice/leader/control crops are ignored under `planning/minnesota/renders/gyms/overhaul-20261117/first-three/final-static-review-07/minneapolis-*`. All earlier material trials remain retained. Rejected native `facility:00` and `facility:03` floors read as void/electrical floor; the final scene uses a normal metal-tile floor and distinct yellow-topped console faces.

The Python static check covers all four feeder combinations: WEST is initially approachable while EAST is initially sealed in the gallery; either powered bridge subsequently reaches that gallery. Each control stance can return to the arrival even with both feeders off, using the service drop. Before victory, the court is reachable exactly when both feeders are on, actors stand on FLOOR, and both exits remain reachable. These are source/token graph checks and source-art visual review, not controller/gameplay, native-engine verification, a build or a published result.
