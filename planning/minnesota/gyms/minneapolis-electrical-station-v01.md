# Minneapolis electrical station — three-gym review candidate

This **unselected** candidate replaces the six identical bins and gray locks
in the installed Minneapolis Gym with native machinery panels and visibly
charged barriers. A walking aisle faces each row of three panels, with a clear
landing below the two barriers. The room is 7×11 metatiles; the passive arrival
is `(6,19)`, the exit carpets are `(6,21)` and `(7,21)`, and the existing city
warp destinations remain indexed as before. Lenvi is `(3,17)`, Jossa `(10,14)`,
Avrin `(6,3)`, and the guide `(9,19)`.

The course retains the selected regional pack's actual rules: defeat both
trainers, then use the left and middle panels in the top row in that order.
A wrong second choice resets the locks while retaining the trainer wins; once
solved the circuit stays open. This is not the different, staged battle/lock
sequence from the older `minneapolis-circuit.json`. Each panel is a solid
native machine with a real adjacent interaction tile; each red grid uses
explicit blocking collision until its gate opens.

Inspect the [complete closed room](../renders/gyms/minneapolis-electrical-station-v06/MnMinneapolisGym-whole.png),
[first lock open](../renders/gyms/minneapolis-electrical-station-v06-first-open/MnMinneapolisGym-whole.png),
[both locks open](../renders/gyms/minneapolis-electrical-station-v06-both-open/MnMinneapolisGym-whole.png),
[native entrance](../renders/gyms/minneapolis-electrical-station-published-v04/native-entry.png)
and [native barrier view](../renders/gyms/minneapolis-electrical-station-published-v04/native-barriers.png).
The source-state event views and native position fixtures do not represent an
earned playthrough. Earlier rejected source renders are retained, including a
first console arrangement that made the middle panel unreachable and a second
that blocked the approach to the opened gates. The authoritative candidate is
[minneapolis-electrical-station-v01.json](minneapolis-electrical-station-v01.json).

The guarded Rust publisher accepts the exact two-gym review parent. It retains
the installed trainer records, switch scripts, event flags, reward and city
warp destinations. It remaps the existing actors, six background-event
coordinates and four gate block changes, and replaces Lenvi's obsolete reference
to a can with a console clue. It also updates room dimensions and the compiled
indexes. The source party examples match the installed Minnesota teams; the
publisher preserves the parent trainer records rather than rebuilding them.
The existing room validator checks the open and closed route and each control.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --bin pack_minneapolis_electrical_station
target/debug/pack_minneapolis_electrical_station \
  content-packs/minnesota-two-gyms-review-v01.crystalpack \
  planning/minnesota/gyms/minneapolis-electrical-station-v01.json \
  content-packs/minnesota-three-gyms-review-v04.crystalpack
```

The resulting content identity is
`94f8e130be14bf18d626f3340b15aa4a586abf7337def60101592e14a80bdca5`.
A fresh rebuild was byte-identical, and the publisher rejects an altered gate
prerequisite before writing a pack. Source pixels and all
collision quadrants match the compiled room. Against the immediate parent,
only `MnMinneapolisGym` and its private `mn_gym_3` tileset change, along with
its compiled map/script/coordinate indexes and five private art files. All
other map modules, trainer records, city warp destinations and the audio
manifest match. All 602 audio programs pass PCM hash, frame-count and loop
checks.

The production-controller course physically operates all six panels, and verifies
early lock refusal, both trainer
wins, the first lock, an incorrect second choice and reset, the correct pair,
the leader battle, badge/TM, both city carpets, re-entry and Continue. It uses
an explicit strong party, prior badges and entrance fixture; it does not prove
ordinary Minnesota-party balance or an earned regional journey. The previous
Rochester and Mankato local controller courses also pass on this combined pack.

No selected-world pointer, hosted pack or user save has changed. Existing
in-room saves need position and block-override migration before selection. The
three designs remain review candidates, and the other five Gyms remain to be
improved and tested.
