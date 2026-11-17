# St. Paul civic loop — four-gym review candidate

This **unselected** candidate reworks the four plain pad rooms into a foyer,
an archive with shelving, a steel-floored machinery chamber and a leader court.
The archive shelves and machinery create paths through their rooms rather than
leaving isolated pads on otherwise empty floors. Native reciprocal panels form
one loop: entrance → Belva → Orven → leader landing → entrance. Either trainer
can still be reached first; both wins open the leader grille. Each arrival pad
can also be retraced by stepping off and back onto it.

Inspect the [whole room](../renders/gyms/st-paul-civic-loop-published-v02/MnStPaulGym-whole.png),
[open court source state](../renders/gyms/st-paul-civic-loop-v05-court-open/MnStPaulGym-whole.png),
[native entrance](../renders/gyms/st-paul-civic-loop-published-v02/native-entry.png),
[native archive](../renders/gyms/st-paul-civic-loop-published-v02/native-archive.png)
and [native machinery chamber](../renders/gyms/st-paul-civic-loop-published-v02/native-machinery.png).
These source-state and native-position views are explicit fixtures, not an
earned save. Earlier rejected source renders remain available. The authoring
source is [st-paul-civic-loop-v01.json](st-paul-civic-loop-v01.json).

The room is 9×12 metatiles. Indexed city arrival is `(2,21)` and both carpets
are `(2,23)` and `(3,23)`. Pad pairs in walking-tile coordinates are:

| Pair | Endpoints |
| --- | --- |
| Foyer / archive | `(1,13)` ↔ `(1,3)` |
| Archive / machinery | `(7,9)` ↔ `(11,13)` |
| Machinery / leader landing | `(15,21)` ↔ `(11,9)` |
| Foyer / leader landing | `(7,19)` ↔ `(15,9)` |

Belva is `(3,3)`, Orven `(13,14)`, Brannic `(13,3)`, and the guide `(6,20)`.
The grille is metatile `(7,2)` and opens only after both trainer wins. The pad
builder verifies every endpoint's native warp collision, reciprocal index and
safe dismount, plus the locked and open walking graphs.

The guarded Rust publisher accepts the exact three-gym review parent. It
retains current trainer records, trainer numbers, prior-badge check, reward,
existing gate qualifications and indexed city destinations. It changes the
private art and room, relocates the existing actors and warp endpoints,
rewrites the two `changeblock` operands to the new grille location and art,
and updates guide/Orven routing text while retaining the Surf directions.
The party examples in the source match the installed Minnesota teams; the
publisher preserves the parent records rather than reconstructing teams.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --bin pack_st_paul_civic_loop
target/debug/pack_st_paul_civic_loop \
  content-packs/minnesota-three-gyms-review-v04.crystalpack \
  planning/minnesota/gyms/st-paul-civic-loop-v01.json \
  content-packs/minnesota-four-gyms-review-v02.crystalpack
```

The resulting content identity is
`0ea1c70ac3b17ae4818468f5b5ce5c1730abab2b369380b187c6b900c0c7b334`.
Complete source terrain and collision quadrants match the compiled room. Against
the immediate parent, only `MnStPaulGym` and its private `mn_gym_4` tileset
change, along with its compiled map/script/coordinate indexes and five private
art files. Other map modules, trainer records, city warp destinations and the
audio manifest match. All 602 audio programs pass PCM hash, frame-count and
loop checks.

A fresh rebuild was byte-identical, and a deliberately altered pad coordinate
is rejected before publication. The
production-controller course crosses all four pad pairs in both directions,
checks that the leader grille denies an early visit, earns both trainers and
the leader, verifies the badge and single TM, and tests both city exits,
re-entry and Continue on a pad. It uses an explicit strong-party and prior
badge fixture; it is not an earned regional journey or a balance test. Separate
controller cases pass loss recovery and the prior-badge refusal after battling
the trainers in reverse order. The Rochester, Mankato and Minneapolis local
controller courses also pass on this combined pack. Workspace formatting and
all-target Clippy for both affected crates pass with existing warnings.

No selected-world pointer, hosted pack or user save has changed. Existing
in-room saves need position and block-override migration before selection. The
four designs remain review candidates, and the other four Gyms remain to be
improved and tested.
