# Northwoods starter signatures

Each starter family has one move exclusive to its final evolution at level36.
Cyprielle retains its already implemented Slipper Waltz. The two additional
signatures are authored in [starter-signature-moves.json](starter-signature-moves.json).
They are compiled into the pack and real learnsets; the website reads those
packed values rather than inserting display-only moves.

| Final evolution | Move | Type / category | Power | Accuracy | PP | Effect |
| --- | --- | --- | ---: | ---: | ---: | --- |
| Cyprielle | Slipper Waltz | Fairy / Special | 75 | 100 | 15 | 20% chance to confuse; no dance lock. |
| Borealynx | Rimebrand | Fire / Physical | 85 | 100 | 10 | 20% chance to burn. |
| Voyagear | Paddle Sweep | Water / Physical | 80 | 100 | 15 | 30% chance to lower the target's Attack by one stage. |

Slipper Waltz follows the lady-slipper dancer's paired petal steps. Rimebrand
turns the winter lynx's ambush into an ember-paw strike. Paddle Sweep uses the
voyageur beaver's broad paddle-tail to drive a crescent of water through a guard.
All secondary effects use the existing native battle rules. The animation shows
the attack, without promising a burn, confusion or stat drop on every hit.

The Grass move remains in the original Northwoods catalog at presentation index
263. Rimebrand and Paddle Sweep append indices271 and272 after the completed
Fairy extension. The original native move pool, all existing move definitions,
normal/shiny creature art, type chart and audio remain unchanged.

The website is published as `20261027-starter-signatures-19` in the
[live Pokédex](https://geothite.ryanculligan.com/pokedex/moves.html).
All three signatures have native normal/shiny previews for both battle sides.
All 1,040 captured frames pass native rendering, OAM limits, mirroring, palette,
APNG parity and cleanup checks. Desktop/mobile browser checks and all 17 exporter
tests pass. Three focused overlay tests preserve every existing runtime file,
all 224 walking banks and all audio; the fresh pack was written and reopened:
`content-packs/northwoods-starter-signatures-v1.crystalpack` (ignored).

Existing art, 448 walking atlases and all 19 previous move animations remain
byte-identical. Captures, independent preservation receipts and publication logs
are retained under `planning/minnesota/renders/signatures-starters-20261027-v1/`.

A production learning test exposed an existing item/evolution adapter limitation:
with four known moves and multiple pending moves at level 36, only the first is
queued, so Rimebrand can be omitted after Ice Beam. The adapter locations are
`game_data_items_and_battles.rs:1949` and `:2020`. The role/learnset metadata test
passes; the secondary-effect controller test remains unverified because its
fixture was rejected by real battle-state validation. No production adapter fix
was made: further engine work stopped at the user's explicit direction to publish
now. The website displays the actual compiled learnsets; the live game's pack
mounts were preserved. Required full workspace formatting and all-target Clippy
retain their documented pre-existing failures.
