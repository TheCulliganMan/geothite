# Minnesota cave redesign

The original Geothite comparison uses Union Cave 1F and B1F, Dark Cave's
Violet entrance, Slowpoke Well B1F, Mt. Mortar 1F Inside and Ice Path 1F
from the local core pack. All six reference rooms have identical block arrays
and events in the hosted Fair Presentation pack. Review used their native
terrain, entrances and events, followed by side-by-side Minnesota renders.

Union Cave 1F gives the through journey a clear direction and puts its ladder
branches off that route. B1F uses water to separate destinations. Dark Cave
uses attached rock to reveal the next leg a little at a time. Slowpoke Well
makes a compact room legible with one raised shelf. In Mt. Mortar, stair
positions make the changes of level readable and the branches lead to
different destinations. Ice Path teaches stops and turns against sharply
different shorelines. These observations inform new Minnesota layouts
while retaining Minnesota's own geography, story and existing tile vocabulary.

Across the reviewed Minnesota caves, long pale floors, isolated pools and
similar stone-framed pockets recur even when the rooms serve different
purposes. Use one visible landform to organize the route through each cave.
Keep optional rewards distinct from that route, and show the player where
they can get back before asking them to commit to a descent or water crossing.

## Authored layouts

[layouts.json](layouts.json), [ore-line.json](ore-line.json) and
[gunflint.json](gunflint.json) hold complete new native metatile rows for six
rooms. Each is bound to its installed block-array hash and preserves its
dimensions, mouth coordinates, event targets, actor positions and programs.

| Room | New route |
| --- | --- |
| River Pocket, R03 | Ordinary stone steps take the entrance bank up toward the walker and supply. Beyond the Strength pinch, a narrow creek bends around the split pillar and opens onto the exposed dry bed. The three original observations still follow that same creek. |
| Ore Line, R10 | The main route cuts around an ore spine. The upper galleries step down into a lower route to the spring. The fracture chamber keeps its gated approach and downhill escape even if the boulder is restored. |
| Gunflint Grotto, R12 | A switchback opens onto Otta's recessed shelter, then follows a narrow spring bank toward a higher cache shelf. The angler's sight lane and the optional Surf island lie beside that route. |
| Cedar Shelter, R13 | Enter alongside a narrow west seep and take the central steps to Harper's protected bench. The Max Ether occupies a separate low recess and the hidden Max Repel remains on the stone beside the seep. Both can be reached on foot. |
| Granite Den, R17 | Turn west from the mouth to the reader and then climb the talus stairs. The Spell Tag stone sits off that ascent and the Revive is on the east lip. A second stair forms the back circuit. The south-facing drop returns from the upper shelf to the reader's level. This dry design removes the den's small fishing pool. |
| Winding Cleft, R20 | The through trail bends between opposing rock shoulders above the flooded fissure. The west supply ledge and the northeast rock cache rise independently from the through path. Both mouths are reachable on foot. The Full Restore remains on the Surf-only foothold. |

The source checker reads the exact hosted archive and models the named collision
permissions used by these rows. It checks outside mouths and return positions,
actor/cache approaches, fixed trainer sight lanes, dry and Surf return paths,
and both item-ball states. River Pocket also checks the pre-badge gate and
both boulder returns. Ore Line checks the fracture chamber both closed and
open. The checker emits whole-room before/after views and paired 160×144 crops
with the installed palette and actor art. These are static source-art renders.
To reproduce them with Pillow and cbor2 installed,
run the following with the checksum-matched hosted pack and an ignored output
directory.

```sh
python planning/minnesota/caves/ground-up-v01/review.py \
  content-packs/minnesota-fair-presentation.hosted.crystalpack \
  target/cave-ground-up-review
```

## Remaining natural caves

| Cave | Ground-up layout direction | Existing contract |
| --- | --- | --- |
| Driftless Hollow, R01 | Cut a stepped limestone passage between the two mouths. Put a shallow fossil shelf in sight from the west and let the eastern return descend along a different side of the spine. | The required crossing stays lit. Preserve the early field lesson, rewards, western and eastern exits, and the separate gallery entrance. |
| Driftless Lamp Gallery | Start with a small shelf at the foot of the ladder. Expose one long fossil seam after Flash and let the passage turn toward the relief in two short reveals. | Flash is optional to the campaign. Keep the relief's interaction, Rare Candy receipt and independent return. |
| Rock Cut, R05 | Use a narrow diagonal cut with one sheltered rest turnout. Put the supply bench above the public path so the downhill ledge rejoins the exit path in view. | Both mouths, healer, Repel and Hard Stone remain. The crossing stays short and the uphill and downhill routes remain legible. |
| Voyageurs Ice 1F | Stage the first slide between an obvious stopping shoulder and a dry turning bank. Open the longer crossing only after the room shows how to change direction. | Preserve entrances, the ladder to B1F, required ice traversal, and the accessible supplies. |
| Voyageurs Ice B1F | Offset the ice basins so each has a new approach. Use a recognizable dry recovery landing and a separate deep reward branch. | Keep the dry Revive route and the ice-dependent PP Up, with safe returns from every reachable stopping tile. |
| Inner Basin | Arrange three indicator alcoves along one visible drain. Opening the outlet should change how the room reads from the entrance and where the player can walk. | Preserve all nine terrain mutations, switch order and return states, and the voluntary Strandrak encounter. |

The Iron Range industrial rooms remain in the separate mine work. Any future
change to the natural caves' mine interfaces must preserve their current gates
and compiled warp pairs.

## Pack integration

These rows are source candidates. The live pack has no terrain changes from
this review. Integration should apply the complete rows in a successor to
Fair Presentation, regenerate both compiled map representations through the Rust
publisher, then rerun core collision and targeted controller fixtures. The
existing wayfinding overlay requires prior standing terrain to stay walkable.
A ground-up replacement changes some of those squares and therefore needs a
new bounded save migration. It should retain safe positions where possible,
map unsafe cave positions to verified arrivals, and preserve item and story
receipts. Test with isolated saves before selecting the new pack.
