# R10 gallery — written native comparison and production views

The 505-map written pack, SHA256
`e0cf8cf00b18c1b25c7cab70e37ebbcc93f1ff25d67fb7aa7f4c4371d567085e`,
was opened only after Root confirmed publication and strict readback. Input is
`.local/minnesota-city-revisions/gallery-e0cf8cf00b18c1b25c7cab70e37ebbcc93f1ff25d67fb7aa7f4c4371d567085e.crystalpack`.
The baseline is the completed505-map northern expansion 3949ad8d. Frozen cave
source SHA256 is `a3ea9a75a23c0b8e0a335aa47ceeffd5762efdccee16512168523260f115aef6`.
At capture time, no source promotion, pack build, Cargo or visible UI was performed by the capture worker. Source promotion subsequently occurred only after the controller result below.

[Whole current-palette room](../renders/caves/r10-native/e0cf8cf00b18c1b2/review-1793065743751041000/whole-native-nite-actor-review.png)
and [six actual production views](../renders/caves/r10-native/e0cf8cf00b18c1b2/review-1793065743751041000/production-decision-cameras.png)
were personally inspected. The whole is a pack-source-art composition, not a
client screenshot of the entire room. The six player frames are actual headless
production classic-renderer captures.

## Exact written-pack checks

- All 252 native blocks match the 18×14 authored rows. Static native/source
  terrain pixels match, including at the actual inherited `nite` palette.
- The original `cave` tileset/art/collision and every embedded runtime/art/audio
  file are byte-equal to the completed baseline. All 504 unrelated maps are equal.
- Group 55/map 3, the original exit event, exterior pairing and all other room
  attributes remain exact apart from the declared 18×14 extent.
- The complete original reader record remains at `(9,5)`, facing right. Its
  original four-line clue and jumptextfaceplayer program are exact.
- The original Revive object differs only in its new position `(29,23)`.
  `REVIVE 1`, `EVENT_MN_R10_ITEM_04`, original item-ball program and typed pickup
  records are exact.
- New typed module caches include the existing MapScripts/MapEvents transport
  sections. Comparison excludes only those two declared transport labels when
  checking behavior programs; the three original reader/text/Revive labels
  remain exactly equal in both module and shared script indexes. The initial
  overly strict cache-label assertion and its log are retained.

The original room declares `time_of_day: nite` and `PALETTE_NITE`. Thus actual
noon client captures use purple cave stone. Generic day-source images use gold
stone. Both are retained, and the current native whole uses the inherited night
palette. No palette rule was altered. The whole comparison uses static native
block art; native water animation phase is shown separately in the actual
production spring capture.

## Actual six captures

Every capture used a private frozen copy of the existing arm64 example,
SHA256 `410c8f79be1a5b6cca7111b53508b8b6f1c9520c4ac22f40a5fdef89701a046d`.
The shared example had changed since the earlier northern review, so its exact
new bytes were frozen before this run. The temporary copy was removed after
completion. All six processes exited 0, totalling 32.901 seconds.

| View | Core position | Actual source/native collision |
|---|---|---|
| Mouth |3,12| Floor, clear arrival above ladder |
| Upper reader approach |9,6| Floor, unoccupied |
| Gallery ledge approach |12,5| Floor, unoccupied |
| East stairs |20,8| Floor, unoccupied |
| Spring cache |28,23| Floor, adjacent to original Revive object |
| Worst lower return |14,20| Floor, below tapered rock spine |

The requested `(14,19)` is Wall. My first proposed substitute `(13,19)` is also
Wall on cave31; it was corrected after reading the actual typed collisions.
Both rejected anchors are recorded. The final `(14,20)` was checked as native
Floor and unoccupied before capture. No image was taken from an invalid setup
and no map geometry changed to accommodate a camera.

All player views use noon and native DOWN standing pose, with the example's
explicit disposable one-partner render fixture. The mouth view is naturally
clamped at the map's west edge, so its player is left of center; it was not
artificially centered. Original 640×576 captures remain untouched. Their central
physical crop `(80,72,560,504)` is reduced exactly 3× to 160×144 and enlarged with
nearest-neighbor for review. The original raw GPU binaries and descriptors,
all encodings, crops, source copies and failed review passes are retained.
All six raw readback pairs were checked for extent, byte length, sibling linkage
and SHA256 identity. The final directory contains 232 PNG artifacts.

The upper reader approach has native foreground-stone occlusion of the player's
lower body. The two mouth stairs, upper ledge, east stair tiers and spring-side
Revive remain legible. The lower return is the broadest remaining floor shoulder;
its separate toe and tapered spine are visible in the deliberately weak frame.
These observations are recorded without changing the frozen candidate.

Exact evidence:
`planning/minnesota/renders/caves/r10-native/e0cf8cf00b18c1b2/review-1793065743751041000/native-capture-summary.json`.
These are native data and presentation checks. Real dialogue, hop, return,
Revive/full-Bag/repeat and Continue acceptance are separately tested by Root's
production-controller cases; the captures do not claim earned regional access.


## Current native and source handoff

Root's actual e0 pack passed all three R10 production-controller cases in `.local/r10-gallery-controller-item-fixed.log` (8.57s): original mouth/passive return, all four stairs, real ledge/reverse/ordinary return; original reader's two fully printed pages; original Revive grant/flag/once behavior; native full-bag refusal, real Potion use to free a slot, retry; and semantic Save/Continue. These local fixtures do not earn regional Q06 access. The tests exposed a real renderer-neutral successful-item clock gap, now fixed by reusing native found-text/fanfare/pocket transitions in the shared controller. Successful pocket pages and movement ownership are directly checked.

The actual written pack also passed 125/125 preservation checks, with no failed checks, in `planning/minnesota/renders/northwoods-native/proof/r10-gallery/comparison-ae43e7811f5c2cd5c2e88c1441ecf2d95053f144396119899111a3d1e505f3b8.json`. Its content identity is `284e14a33daabeda050dfc53ba1c96e53cd513998de7355aaa38e3a5de2606da`. The six headless captures above remain exact production-image evidence, distinct from controller state proof and the full-room source composition. Original forced `NITE`/`PALETTE_NITE` remains unchanged.

After Root explicitly approved that test result, `caves/promote_r10_gallery.py` promoted only the route's room object and its global interior contract. Geometry is18×14; native mouth3,13 and clear3,12, exterior36,17 and passive36,18, original whole Nerd/text and Revive×1 program/flag remain exact. Only the Revive position changes to29,23. The immutable before records, complete immediately-before files, passing controller log and promotion/post-apply receipts are retained under `planning/minnesota/renders/caves/r10-promotion/preflight-fb51461d98d7d988/application-1793067244569140000/`. The candidate source and every earlier render remain retained unchanged.

The post-promotion plan check PASS against the exact e0 pack:20 routes,19 hubs,57 chunks,86 adjacency contracts and158 planned maps, with route/story species and pickup IDs resolved. This is source/inventory and symbolic progression validation; final user visual approval and the connected earned regional journey remain unclaimed.
