# Upper Mississippi Woods grass and westward return

Accepted as `world.route_grass_upstream_woods_revision` after the gates below.
Pack: `content-packs/minnesota-current-grass-upstream-woods-v01.crystalpack`.
SHA256: `535f3ed1b2dd4e03adccd2412001ac651cc959a66e1b7267fb6385dd90834fea`.
Identity: `7260ca192597ff2650f3a11e3d665d08034a92d8a4d006f33ed7ba8eb825bf52`.
Parent: selected Iron Range Descent, SHA256
`a8853b0dfeb67185b7d286c729e34df48284821acd3cffd5976c1b1f71a4aef9`.

The native Crystal Route29 reference, complete six-chunk source composition and
five actual native2Dviews were visually inspected. The northern Ether bank now
has grass on the ordinary eastbound walk and a dry native westward hop. Both
ledge lanes require three grass steps eastbound. The ordinary distances are
seven and five tiles; the dry return displaces three tiles in either lane.
Actual native inputs verify the lip, hop, reverse-wall denial and ordinary walk
around both ends. This is a short bank loop, preserving the quiet winding river.

Five metatiles change in `MnMississippiUpstreamWoodsC01R00`: cave-free original
Johto4c makes the westward ledge, a former grass cell becomes its dry approach,
and three lawn cells become encounter grass. Grass changes792to796walking
quadrants. The two new reverse-wall quadrants are the only lost old walking
cells. All other old walking access, water, fitted shores, actor/event footing
and the twenty existing native trainers remain. All unoccupied grass is reachable
from legal Surf shores. Supplies retain positive grass costs2/3/2; the medic's
rest landing stays dry. The weighted audit includes the actual westward hop,
all legal Floor/Grass dismounts and current actor bodies.

The old prepared source was reconciled against the selected Descent parent:
43fresh field-equality checks prove all six complete map records, encounter tables,
twenty complete trainer records, fishing/field-move catalogs and native Johto art
remain identical. The full4352metatile source preimage matches. This is not a
hash-only rebase; all controller and client acceptance was repeated on the child.
The20installed bank-refinement cells absent from the frozen fun source remain.

Native acceptance consists of three cases: all24bank activities (twenty actual
trainer battles with selected Surf PP/money/XP/victory flags, three once-only
supplies and actual medic HP/PP restoration); wild/RUN/ordinary return/Continue;
and both eastbound grassy lanes/dry west hops/reverse denial/reached checkpoints.
The activity case uses a strong local Mew80/Surf plus Voyagear40/Dozebear40 before
any input, with initial badges/items and healer injury/PP explicit setup. These
prove behavior and persistence, not earned regional access or battle balance.
Every authored regional trainer, wild, zone/swarm and bound fishing/time-table
species remains in the active original Minnesota roster, excluding retired
families and removed evolutions.

The production client starts from the actual reached lower-lane checkpoint at
16,19. Ordinary inputs step onto actually added grass at15,19, trigger a real
wild battle, visibly select RUN and show escape narration, then return to16,19.
There are54JSON-RPC calls and empty stderr. Fresh-process Continue, movement to
17,19, Start/Pack/Items/Balls and return to walking add15calls with empty stderr.
All JSONL requests/responses, witness text and reached saves remain retained.

The independent757check audit proves only five cells, their sole encoded block
mirror, one manifest append and derived identity change. All729other complete
maps, all art/audio/story/actors/parties/encounters/fishing/recovery and all
previous Iron Range terrain/stairs remain exact. Five mutations are rejected;
raw pack identities recompute exactly. North Shore/Cedar remain ice-free and
approved inland winter is preserved. All377audio programs pass hashes, frame
counts and loop ranges. Five inspected native2Dcaptures match every pixel.
No3Dpreview was produced.

Authoritative new sources are under
`planning/minnesota/connections/current-river-grass-v01/`. Frozen fun/refinement
sources and all previous renders/candidates/logs remain history. Source images
and native captures are under
`planning/minnesota/renders/route-grass-v01/current-upstream-woods-v01/`.
Runtime acceptance logs use `.local/route-grass-review/upstream-woods-*`.
Scoped Rust formatting and fresh assets/Bevy-all-targets Clippy pass with existing
warnings. Whole-workspace formatting still reports unrelated existing differences.

Three outdoor groups (Boundary Safari, Confluence, Rainy River Icefields), eighteen
relevant town exteriors and final current-story coverage remain open.
