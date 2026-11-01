# R06 current-story grass and ledge verification

R06 is reviewed on the accepted704-map current_story_revision. Its earlier
21-cell grass/ledge revision was already present, so this pass changes no terrain
and publishes no new game pack. The full statewide grass objective remains open.

Pack: `content-packs/minnesota-northstar-current-story-v03.crystalpack`.
SHA256: `560e49986d14e48c07526e9878eddc69be46d546ad079774f04b10f58e7e5c88`.
Identity: `f2808268fbda859cf97b79b8460086c8fc0251cc4870f68504d58f1426a822aa`.
All three complete R06 modules are identical to the accepted R05 parent beneath
the new story. The current canonical terrain matches all1408native metatiles.

## Crystal behavior and visual review

The saved authentic Crystal Route29 reference, complete R06 actor map, and three
production2D cameras were inspected: northern ledge, camp habitat and the narrow
pine-promontory bend. R06 has49grass metatiles (196walking tiles), including useful
camp/fork/glade approaches. The northern glade band requires7grass steps on the
ordinary outward journey and0in that band on the return hop. Existing southern
shared habitat still crosses both ways. The result holds after actual catalog
Cut replacement and even without actor bodies, so it is terrain-driven.

Both ledge lanes work natively with blocked reverse, an ordinary grass climb,
and dry downhill trail. The original Cut lookout/return, mandatory outlet
bridge, fishing pier, cabin and Perry's battle-or-grass alternative remain.
The lookout camera includes44water tiles. No new3D/2.5D view was made.

## Exact source and native checks

The bound dossier freezes actual native parties and cabin rows/border while
leaving the game unchanged. Source checks compare all objects, signs, warps,
room geometry, trainer identities/parties and compiled healing routine. All3
native maps and11outdoor objects match. Native/source terrain pixels are equal
for all1,441,792pixels. The source render validates the Cut targets, water
closures, mandatory bridge and both ledges before/after Cut.

Receipt (relative to planning/minnesota):
`renders/route-grass-v01/current-r06-audit-v01/f2808268fbda/receipt.json`.
The published directional proof and all3adversarial negatives pass:
`renders/route-grass-v01/current-r06-audit-v01/R06-directional-10e580d4a5d670de.json`.
Both full source states, all intermediate renders and the3native captures remain
under that render directory. Current captures are r06-ledge-704-v01-2d.png,
r06-camp-704-v01-2d.png and r06-bend-704-v01-2d.png.

Five native controller cases pass on this exact704-map pack: promontory bend,
both glade ledge lanes, populated five-trainer/eight-opponent journey,
prerequisite-denied/allowed Cut and target/persistence checks, and Surf/rocks/
reciprocal lake seam. Real wildlife is encountered and escaped; trainer wins
spend PP. Medicine, poison/recovery, cabin returns, pickups and Save/Continue
party/bag/badges/flags are exercised. Log:
`.local/route-grass-review/current-r06-704-controller-all.log`.

The inherited populated fixture deliberately uses a source-created Mew60 with
HM-compatible moves, injury/poison, five badges, explicit location and initial
replay clock. Actions after setup use production input. This is behavior and
persistence evidence, not an earned or balanced regional journey. No party,
encounter, trainer or runtime policy is changed by this review.

The reached save `.local/route-grass-review/current-r06-704-reached.crystalsave`
passes12production JSON-RPC calls: actual local movement(51,32) to(50,32),
Start/Pack, collected Repel/Ether and GreatBall in their pockets, and return to
walking. Stderr is empty. Receipt:
`.local/route-grass-review/current-r06-704-stdio.receipt.json`.
All377audio PCM hashes/framecounts/loop ranges pass. R11's8chunks, Duluth and
GrandMarais have zero ICE blocks or scripted ICE changes on this full story pack.

## Scope and next route

The active game selector is preserved. This audit is stored independently of
story/cave publication ownership. All earlier execution receipts keep their own
identity bindings. R07-R20 and outdoor regional links remain open; final current
story-aware coverage must account for modules changed by later integrations.
