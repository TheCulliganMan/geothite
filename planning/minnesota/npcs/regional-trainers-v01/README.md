# Minnesota regional trainer revision

Ten additional trainer types join the 31 existing regional types. The 41-type
pool covers 273 ordinary opponents: 263 existing encounters and ten new ones.
Existing names, parties, AI, levels, positions, scripts and rewards are retained.
Every regular type appears at least four times. Eighteen Minneapolis cyclists
receive Cycle Tourist battles while retaining their original bicycle field art
and patrols.

Thirty-six older overworlds were repaired against authentic Crystal six-pose
banks: 23 regular types, Reed, eight Gym Leaders and four Elite Four members.
The head size and compact body stay consistent at 16×16; detailed equipment is
reserved for battle art. The eight already-normal regular types and the accepted
woman Tamarack/mean paramilitary ICE sets retain their pixels. Leadership keeps
the Greg Bovino olive-greatcoat reference in the
[original acceptance record](../tamarack-ice/04-ice-enforcer/ACCEPTANCE.md).

| New type | Trainer | New encounter |
| --- | --- | --- |
| Snowplow Driver | ROWENA | Detroit Lakes (15,18) |
| Rail Worker | HOBB | Grand Rapids (22,25) |
| Lake Diver | NAIA | Leech Lake (30,24) |
| Disc Golfer | FINN | St. Cloud (25,25) |
| Prairie Painter | ALDEN | Bemidji (25,25) |
| Conservation Officer | WREN | Boundary Waters (21,28) |
| Transit Driver | OTIS | St. Paul (40,40) |
| Sauna Goer | HILDA | Grand Marais (18,24) |
| Rock Hound | KIT | Duluth (40,44) |
| Snowmobile Rider | INGA | Ely (26,22) |

`asset-plan-v1.json` selects all 58 authoritative native grid sources, their
integration files, Crystal field references and controller fixtures. These are
41 regular sets, 13 named sets and four Tamarack/ICE sets. Tamarack keeps her
field-only professor role. Reed binds all six existing rival variants and nine
field appearances; the three ICE classes bind all 17 existing opponents. Team
Lode, Champion Casey and Johto/Kanto remain intact.

`distribution-v2.json` supplies 255 exact field/battle assignments. The additional
18 bicycle battles are in the Cycle Tourist binding, with no field replacement.
The publisher rejects stale classes/sprites, missing actors and protected story
opponents. Both typed and raw event rows receive matching sprite operands.
Private leader/Elite classes retain leader victory music and Gym happiness.

## Rebuild and run

All image encoding and pack publication use repository Rust tools. The shell
script uses Python only to read the source selection and write its temporary
publication plan. Native PNG/2bpp/palette banks, verification output and packs
stay ignored. The script refuses existing output and evidence directories.

```sh
CARGO_TARGET_DIR=target/regional-trainers-build cargo build --locked \
  -p crystal-assets --bin pack_trainer_art_batch
CARGO_TARGET_DIR=target/regional-trainers-build cargo build --locked \
  -p crystal-assets --example npc_sprite_candidate
CARGO_TARGET_DIR=target/regional-trainers-build cargo build --locked -p geothite
CARGO_TARGET_DIR=target/regional-trainers-build \
  bash tools/build-regional-trainer-sprites.sh \
  content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack \
  content-packs/minnesota-regional-current-v03.crystalpack
./target/regional-trainers-build/debug/geothite play \
  content-packs/minnesota-regional-current-v03.crystalpack
./target/regional-trainers-build/debug/geothite mcp \
  content-packs/minnesota-regional-current-v03.crystalpack
```

Use a fresh ignored output name to repeat a publication. The input is the
selected Duluth Cliff Stairs v03 descendant, SHA256
`6878059cebf9826b319b840bbcb2a8f243a73556c68d00cc0d135ad4a597265f`.
Check `planning/minnesota/world.json` before rebasing onto later map work.

## Verification scope

Individual `ACCEPTANCE.md` files record native/reference inspection and real
intermediate encounters for each new or repaired character. The generic trainer
fixtures supply an adjacent position and three legal level-60 Minnesota starters;
boss fixtures explicitly declare prerequisite badges or flags. Actual controller
inputs must open dialogue, reject movement during it, spend move PP, win legally,
record defeat, preserve rewards on repeat talk and survive saving/Continue.
Tamarack and the Winona rival course earn their opening through normal inputs.
These checks do not establish intended-level balance or an earned full campaign.

The final combined-pack audit and encounter receipts are recorded in
[STATUS.md](STATUS.md). Earlier walking-cyclist and stale-event candidates are
rejected history and must not be selected as the current game.
