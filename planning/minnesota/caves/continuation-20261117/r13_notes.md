# Cedar Shelter: a bank worth exploring

The `MnR13Cave1F` entry in `rooms.json` changes eight native metatiles in
the 10x8 shelter. Harper's dialogue is installed from the regional `story.json`.

The old pool had a long western wall that led nowhere. The new northern bank
connects that shore to Harper's room, making an ordinary dry loop around the
water. The roof pinches the loop into two small chambers instead of adding a
wide empty gallery. A single stone above the water marks the new discovery.
Surf provides a short crossing of the pool, while the whole shelter and both
rewards remain accessible without it. The existing stepped Max Ether alcove
keeps its separate approach.

## Exact content contract

- Map `MnR13Cave1F`; 10x8 metatiles; `cave`, night palette, existing border46.
- Exact normalized `{blocks,objects,events,scripts}` preimage SHA256:
  `98a4937cf9f2eb12d72d9e9924607c39130fa6e081b50ab7b7212459d0970dd4`.
- Safe dry arrival `(7,14)`; retain indexed mouth1 at `(7,15)` and its exact
  `MnR13C00R01` warp2 exterior pairing.
- Retain both object records byte-for-byte, including order: Harper `(13,5)`
  and `MN_R13_SHELTER_ETHER` `(15,11)`.
- Preserve the full `MN_R13_HARPER` HealParty script and native Max Ether
  itemball with `EVENT_MN_R13_SHELTER_ETHER`.
- Add `MN_CAVE_CEDAR_MAX_REPEL`, native hiddenitem `MAX_REPEL`, quantity1,
  event `EVENT_MN_CAVE_CEDAR_MAX_REPEL`, `BGEVENT_ITEM` at `(6,4)`.
  Its target is the single standing rock on the north bank. Harper's
  revised conversation gives a grounded clue and still leads into healing.
- Replace only `MN_R13_HARPERText`. No new actors, trainers, ladders, scenes,
  visible creature actors, wild encounter changes or art assets.

## Source review

The source-only graph checks actual selected-pack collision quadrants, actor
bodies, directional walls and retained warps. With the Max Ether present,
92 dry positions and116 Surf positions are reachable; after collecting it,
93 and117. Every reachable position returns to the arrival. All original and
new interaction approaches are reachable. Water stays inside the room; the
only ladder is the original paired mouth. Whole room and player-scale
decision views were inspected in `target/cave-continuation-20261117/r13/`.
The first wider northern gallery was rejected; its immutable render remains
in the ignored history alongside the narrower accepted candidate.

The native Union Cave and Dark Cave source maps are the reference for useful
side banks and recoverable exploration. Their visible supplies and hidden
items serve different discovery roles; this shelter borrows that distinction
without duplicating their layout or adding an unnecessary battle.
References: [Union Cave 1F scripts](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/UnionCave1F.asm),
[Dark Cave scripts](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/DarkCaveVioletEntrance.asm).
Actual original source-art maps were reviewed from the supplied pack under
`target/cave-overhaul-20261117/baseline/reference/`.

## Migration boundary

This room has no block-changing programs, coordinate scenes, Strength objects
or movable story actors. Its only collectible actor is backed by its retained
event receipt. A bounded migration may discard this room's old block/object
snapshot and place a quiescent in-room player at `(7,14)`, retaining inventory,
party and all story/item flags. Preserve current-slot recovery, reject active
scripts/battles and use the existing movement/canoe safety policy. Do not copy
or relabel arbitrary saves. Root owns Rust integration and final validation;
no controller, client or battle was run for this source candidate.
