# Playable trainer cast

This is an art and local-encounter record for the named trainer-cast review
pack. It does not select the regional game or establish the current story arc.
Use the [character profiles](../../playable/northstar-character-story.md) for
story roles and the [current story record](../../revisions/current-story-v01/README.md)
for that publication's integration. Locations and encounter results below keep
their original pack scope. Only active Minnesota species belong in regional
teams; historical review fixtures do not define current parties.

**21 new native sprite sets, combined with the original 15 NPC sets: 36 distinct characters.** Each has a matched 56×56 battle sprite, six hand-authored 16×16 field poses, its own four-color palette, and an individually tested encounter.

Reed's corrected art direction is an overweight adult rival: stocky belly-forward build, scruffy hair and stubble, burgundy open shirt and a smug expression. The child design is rejected. See [his art profile](01-reed/DESIGN.md) for the binding boundaries.

New cast: Reed; Gym Leaders Varel, Torven, Avrin, Brannic, Saven, Sivren, Orlis and Norek; Elite Four Fern, Flint, Veil and Halden; Cycle Tourist Ren, Beekeeper Nell, Lumberjack Axel, Bird Watcher Pippa, Dairy Farmer Edda, Mushroom Forager Morris, Volunteer Tess and Ski Patroller Sloane.

Battle concept references used built-in ImageGen. Each new character folder retains its reference PNG and prompt text; accepted native JSON contains the hand-edited field grids and refined battle pixels.

## Play locally

From `/Users/ryanculligan/GitHub/geothite`:

```sh
./target/trainer-cast/tools/geothite play content-packs/minnesota-trainer-cast-36-v01.crystalpack
```

This is a separate local review pack descended from `minnesota-northstar-league-dialogue-v01.crystalpack`. No current-world pointer, deployment or unrelated source was promoted. Generated packs and native banks remain ignored under repository rules.

## Review

- [New cast gallery](../../../../target/trainer-cast/cast-progress.png)
- [All 36 characters on the final pack](../../../../target/trainer-cast/final-audit/combined-gallery.png)
- [Verification and limits](FINAL-VERIFICATION.md)
- [Character sources and individual evidence](STATUS.md)

## Encounter locations

| Character | Role | Map | Tile | Approach |
|---|---|---|---|---|
| REED | rival | MnWinona | [16, 13] | [16, 14] |
| VAREL | Gym Leader 1 | MnRochesterGym | [6, 3] | [6, 4] |
| TORVEN | Gym Leader 2 | MnMankatoGym | [11, 3] | [11, 4] |
| AVRIN | Gym Leader 3 | MnMinneapolisGym | [4, 3] | [3, 3] |
| BRANNIC | Gym Leader 4 | MnStPaulGym | [11, 2] | [11, 3] |
| SAVEN | Gym Leader 5 | MnStCloudGym | [3, 3] | [3, 4] |
| SIVREN | Gym Leader 6 | MnBrainerdGym | [5, 3] | [5, 4] |
| ORLIS | Gym Leader 7 | MnBemidjiGym | [5, 3] | [5, 4] |
| NOREK | Gym Leader 8 | MnDuluthGym | [8, 3] | [9, 3] |
| FERN | Elite Four 1 | MnLeagueRoom1 | [5, 7] | [5, 8] |
| FLINT | Elite Four 2 | MnLeagueRoom2 | [5, 7] | [5, 8] |
| VEIL | Elite Four 3 | MnLeagueRoom3 | [5, 7] | [5, 8] |
| HALDEN | Elite Four 4 | MnLeagueRoom4 | [5, 7] | [5, 8] |
| REN | Cycle Tourist | MnStCloud | [14, 13] | [14, 14] |
| NELL | Beekeeper | MnRochester | [23, 12] | [23, 13] |
| AXEL | Lumberjack | MnGrandRapids | [22, 24] | [22, 25] |
| PIPPA | Bird Watcher | MnBemidji | [28, 35] | [28, 36] |
| EDDA | Dairy Farmer | MnMankato | [32, 24] | [32, 25] |
| MORRIS | Mushroom Forager | MnEly | [23, 18] | [23, 19] |
| TESS | Volunteer | MnStPaul | [27, 20] | [27, 21] |
| SLOANE | Ski Patroller | MnGrandMarais | [16, 14] | [16, 15] |
| RANGER ROWAN | preserved original NPC | MnR01C00R00 | [7, 24] | [7, 25] |
| CANOE GUIDE OREN | preserved original NPC | MnBrainerd | [42, 21] | [42, 22] |
| MINER MARA | preserved original NPC | MnGrandRapids | [21, 24] | [21, 25] |
| BOTANIST ELDEN | preserved original NPC | MnMankato | [31, 24] | [31, 25] |
| ANGLER NETTA | preserved original NPC | MnBemidji | [27, 35] | [27, 36] |
| COURIER IVAR | preserved original NPC | MnEly | [16, 18] | [16, 19] |
| CAPTAIN LEIF | preserved original NPC | MnStCloud | [13, 13] | [13, 14] |
| ORCHARD KEEPER POMONA | preserved original NPC | MnRochester | [22, 12] | [22, 13] |
| RICE HARVESTER SEDGE | preserved original NPC | MnLeechLake | [24, 19] | [24, 20] |
| MECHANIC TOBIN | preserved original NPC | MnDuluth | [54, 59] | [54, 60] |
| PHOTOGRAPHER LARK | preserved original NPC | MnEly | [22, 18] | [22, 19] |
| BAKER DOTTIE | preserved original NPC | MnStPaul | [26, 20] | [26, 21] |
| STORM WATCHER GALE | preserved original NPC | MnStCloud | [16, 13] | [16, 14] |
| FOSSIL RESEARCHER FARA | preserved original NPC | MnWinona | [15, 13] | [15, 14] |
| NIGHT SKY OBSERVER ASTER | preserved original NPC | MnGrandMarais | [26, 27] | [26, 28] |

Boss art is bound to the existing actors and battle IDs; their teams, AI, rewards, prerequisites and script flow are preserved. Halden is the current fourth Elite Four member. Champion Casey was outside the requested art scope.

Reed has one playable Winona review encounter. Six older recurring rival scenes are absent from the compact parent; this pack does not claim to restore that story arc. Regular trainers elsewhere named Reed keep their own identities.

The original fifteen retain their exact accepted art, palettes, parties and dialogue. Placement changes avoid compact terrain and newer actors. Ivar moved from absent International Falls to Ely; Gale moved from absent Moorhead to St Cloud. No retired maps were transplanted.
