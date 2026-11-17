# Granite Den: climb in, hop back

Source candidate on `content-packs/minnesota-gym-cave-overhaul-v02.crystalpack`.
The `MnR17Cave1F` entry in `rooms.json` keeps the compact 7x8 den, its two existing discoveries and native
cast. Two meaningful terrain replacements turn the cache loop into a small
climb with a faster return: native stair block12 at metatile `(2,4)` and
native HOP_DOWN block39 at `(3,4)`. This is an authored source revision, not a
published pack or gameplay claim.

The player climbs the west steps around the central granite outcrop to the
visible Revive and old hidden Spell Tag. From `(7,8)` the return ledge jumps
over `(7,9)` to `(7,10)`; it cannot be climbed in reverse. The ordinary west
stair route remains available in both directions. The existing researcher at
`(6,10)` is beside the landing, not on it. This uses a native collision ledge,
not a painted shortcut or a script that teleports the player.

## Exact content contract

- Map `MnR17Cave1F`; 7x8 metatiles; `cave`, night palette, existing border46.
- Exact normalized `{blocks,objects,events,scripts}` preimage SHA256:
  `de70e346196ff8a7cfc3d05208586a382f31032c1d7404be2386e2ff3032e068`.
- Safe dry arrival `(9,12)`; mouth1 stays at `(9,13)` with its exact
  `MnR17C00R00` warp2 exterior pairing.
- Retain both original object records and their order. Revive actor
  `MN_R17_ITEM_04` stays `(5,5)` with `EVENT_MN_R17_ITEM_04`.
  Researcher object ID `MNR17CAVE1F_NPC2` stays `(6,10)` and still dispatches
  `MN_INTERIOR_R17_LICHEN_READER`.
- Preserve `MN_HIDDEN_R17CAVE1F`: SPELL_TAG, event
  `EVENT_MN_HIDDEN_R17CAVE1F`, background event `(8,8)`, which remains FLOOR.
- No new rewards, event flags, actors or trainers. Replace only the plain
  `MN_INTERIOR_R17_LICHEN_READERInteriorText` pages from the regional `story.json`.
  The researcher keeps the patient lichen observation and explains the
  local steps/ledge without acquiring knowledge of distant progression.

## Source review

The static source graph reads native selected-pack collision quadrants and
uses HOP direction on the departure tile, matching the Rust movement code.
The outward `(7,8)->(7,10)` jump is valid, its reverse is blocked, and the
ordinary stair route remains. The old hidden-item corner `(8,8)` returns to
the arrival in six movement edges with the ledge, versus eleven without it.
All49 reachable resting positions return; collecting the Revive opens a
fiftieth position without creating a trap. Both actors, the hidden-item
approach and the mouth remain reachable. Whole room, stair/ledge and cache
cameras were visually inspected in `target/cave-continuation-20261117/r17/`.

Original Union Cave and Mt. Mortar source-art rooms informed the asymmetric
return and visible supply placement. This den is intentionally a short
optional discovery, not another sprawling route or a compulsory puzzle.
References: [Union Cave 1F](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/UnionCave1F.asm),
[Mt. Mortar interior](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/MountMortar1FInside.asm).
The original maps were reviewed from the supplied pack's source art.

## Migration boundary

No block-changing scripts, coordinate scenes, Strength objects, hiding story
actors or callbacks exist here. The old Revive and hidden Spell Tag use their
unchanged event receipts. Clear only this room's saved geometry/roster during
the approved migration; move an eligible in-room player to `(9,12)` while
preserving all earned state. Root owns exact identity, quiescence and write
safety checks and final Rust integration. No controller, client or gameplay
was executed for this candidate.
