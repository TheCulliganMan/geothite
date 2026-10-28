# River continuity authoring

The current native river journey is traversable with the declared Surf and
Waterfall fixture, but its physical geography needs work. This folder keeps
the continuity review separate from the published winter pack and concurrent
city authoring. The immutable parent is
`content-packs/minnesota-northern-winter-v02.crystalpack`, file SHA-256
`92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33`.

## First bounded correction

`source-v01/candidate.contract.json` declares **51 terrain cells and one sign
coordinate**, with full native module preimages and explicit before/candidate
layouts. It is a detached source proposal; native publication and input tests
remain the integration owner's responsibility.

The Minneapolis south river enters `MnR03C00R00` at local core x12–17, y0
(city x32–37, y63; reciprocal south offset10 / north offset−10). The old east
turn compresses the Mississippi into a single metatile row before its actual
east connection into `MnMississippiConfluenceC00R00`. The proposal widens that
turn to three metatile rows, keeps the native north/south walking bridge and
east/west Surf underpass, and matches all16 core edge cells to the real neighbor.
It keeps every connection and map rectangle unchanged.

The existing `MN_R03_NORTH_SIGN` event moves from core `(22,4)` on the old
pine wall to `(22,8)` on a visible native sign beside the south approach. Its
script, text and event type must remain exact. The publisher must update the
typed event, matching native event declaration and required source mirrors;
changing only a rendered sign is insufficient. All NPCs, trainer sights,
items, room mouths, other events and programs stay fixed.

The relevant source-native images are saved under
`planning/minnesota/renders/river-continuity-v01/first-bend-v07/`:

- `twin-cities-before.png` and `twin-cities-candidate.png` show the complete
  connected city area, preserving actual positions and blank unmapped space.
- `whole-corridor-before.png` and `whole-corridor-candidate.png` show the whole
  Winona–St. Cloud corridor at one scale.
- `bridge-south-*-4x.png`, `bridge-west-surf-*-4x.png` and
  `sign-approach-*-4x.png` show the native160×144 source cameras at integer4×.
  They are static source previews, not reached gameplay or GPU captures.
- Every full-resolution original, native160 crop and rejected earlier epoch
  remains saved. The rejected passes are not publication candidates.

`bridge-static-proof.json` records both walking directions, six east/west
Surf rows in both directions, lateral rail closure, north/south Surf exit
denial and the relocated sign approach. This verifies source geometry only.
Run native controller tests for those actions, the sign's actual text, the
original badge/falls gates, Continue, and the complete long river journey
after publishing through Rust. Do not infer gameplay success from these PNGs.

Reproduce the source preview into a **fresh** epoch:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/connections/river-continuity-v01/review_first_bend.py \
  --pack content-packs/minnesota-northern-winter-v02.crystalpack \
  --output planning/minnesota/renders/river-continuity-v01/first-bend-NEXT
```

## Continuing work and boundaries

The first correction does not resolve these other visible problems:

1. **Parallel R05 watercourse and repeated long reaches.** The old straight
   channel still reads as a second river beside the required long wooded
   course, although its existing buoy break prevents the old Surf shortcut.
   The two64×68 wooded reaches also repeat a giant eastward hairpin silhouette.
   The dedicated long-river author owns detached candidates for the12 woods
   chunks and `MnR05C00R00/R01/R02`. Keep the long quiet journey, real water
   seams, few bank trainers, useful grassy landings and all existing lookout,
   cave, bank and walking promises. Recompose the river and backwaters by hand;
   deleting required river distance is not a fix.
2. **R04 phantom civic canal.** `MnR04C00R00` has WATER_21 reaching its north
   and south boundaries without water neighbors. Its two fenced bodies contain
   96 and84 core water tiles, with no local land-to-water mount witness. This is
   a visible geography problem; a Surf escape has not been demonstrated. Make
   it a contained civic pond/canal with visible shore caps and truthful sign
   wording, keeping the old Foot-only bridge and all story behavior. A separate
   author owns R04 and St. Paul proposals under coordination with this folder.
3. **St. Paul water handoff.** Retain the real east outlet and south connection,
   but review the hard southeast corner stem together with the lower wooded
   reach. A reachable graph alone does not make this bend coherent.

Actual exterior positions, in metatiles relative to Minneapolis:

| Map | Origin | Size | River/road contract |
| --- | --- | --- | --- |
| Minneapolis | `(0,0)` | `40×32` | River north and south remain real ports. St. Anthony Falls and badge gates remain native. |
| R03C00R00 | `(10,32)` | `24×24` | City river through north; Minnesota River branch south; Mississippi outlet east. |
| R04C00R00 | `(40,9)` | `28×18` | Foot road west/east; fenced civic water has no actual water seam. |
| ConfluenceC00 | `(34,32)` | `6×8` | R03 west offset0; ConfluenceC01 east offset−4. |
| ConfluenceC01 | `(40,28)` | `32×12` | Main river west/east. Keep its north forest bank separating R04. |
| ConfluenceC02 | `(72,28)` | `32×12` | Main river west/east. |
| ConfluenceC03 | `(104,28)` | `4×12` | Main river turns north into St. Paul, north offset−36. |
| St. Paul | `(68,-6)` | `40×34` | South core74–77,y67 ↔ Confluence north core2–5,y0; east core79,y52–55 ↔ LowerWoods west core0,y24–27. |

**There is a real one-metatile gap at world y27 between R04 and Confluence.**
Do not paint terrain in the atlas to hide it or add a purported direct map
connection across it. The agreed civic-pond fix needs no new join.

`audit_water_edges.py` reads token resolution and numeric terrain classification
from the current production Rust sources. It includes WATER_21, all water and
current variants, whirlpools and directional buoys, and records the source
hashes. The inventory covers all99 native exteriors. It distinguishes local
mount witnesses from unreachable decorative water and does not simulate event
gates, current motion, campaign access or native input. Its results supplement
visual review; they cannot establish geographic quality.

Its local component counts include numeric `Terrain::Water` cells only. They
exclude the opt-in bridge permissions `e0`–`e3`, whose Surf layer is water along
the bridge axis. In particular, St. Paul's three pure-water components are
joined by the native east/west Surf underpasses; the raw count is not evidence
of three disconnected basins. Use the production bridge layer when tracing a
complete Surf journey.
