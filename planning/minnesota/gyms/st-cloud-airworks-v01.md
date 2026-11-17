# St. Cloud airworks — five-gym review candidate

This **unselected** candidate frames the suspended wind course with native
wooden tower supports and gives the entrance a timber landing. Native chevrons
on every wind tile show its actual forced direction. The stone platforms,
trainer placements, two gated crossings and useful return lanes retain their
existing gameplay geometry. The layout deliberately remains an elevated course
with open space below the crossings; the supports and visible direction are the
changes, not a new movement mechanic.

Inspect the [whole room](../renders/gyms/st-cloud-airworks-published-v01/MnStCloudGym-whole.png),
[lower crossing open](../renders/gyms/st-cloud-airworks-v04-lower-open/MnStCloudGym-whole.png),
[both crossings open](../renders/gyms/st-cloud-airworks-v04-court-open/MnStCloudGym-whole.png),
[native entrance](../renders/gyms/st-cloud-airworks-published-v01/native-entry.png)
and [native east platform](../renders/gyms/st-cloud-airworks-published-v01/native-east-platform.png).
The event views and native position fixtures do not represent an earned save.
Earlier rejected source studies remain archived. The authoring source is
[st-cloud-airworks-v01.json](st-cloud-airworks-v01.json).

The room remains 9×11 metatiles, with passive arrival `(4,19)`, carpets
`(4,21)` and `(5,21)`, Wyrel `(4,15)`, Hallen `(14,10)`, Saven `(3,3)` and
the guide `(7,18)`. The eastbound lower crossing, westbound upper crossing,
top eastbound return and bottom westbound return retain the installed directional
collision. The two gates retain their existing trainer requirements and location.

The guarded Rust publisher accepts the exact four-gym review parent. It retains
city warp records, actor positions, trainer records, trainer numbers, story,
badge and reward behavior. It remaps the gate block IDs to the new private
art and updates the guide and trainer directions while retaining the Fly
information. The source party examples match the installed Minnesota teams;
the publisher retains the actual parent records. It compares every native
collision quadrant to the parent before publication, then runs the existing
wind-course and gate validator. A deliberately changed blocking tile was rejected,
and a fresh rebuild was byte-identical.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --bin pack_st_cloud_airworks
target/debug/pack_st_cloud_airworks \
  content-packs/minnesota-four-gyms-review-v02.crystalpack \
  planning/minnesota/gyms/st-cloud-airworks-v01.json \
  content-packs/minnesota-five-gyms-review-v01.crystalpack
```

The resulting content identity is
`1804f99330dcbc18b467156507acf2b29e34d86bbf2adf273ee0b3c79fd57175`.
Complete source pixels and collision quadrants match the compiled room. Against
the immediate parent, only `MnStCloudGym` and its private `mn_gym_5` tileset
change, together with the map/script indexes and five private art files. Other
map modules, trainer records, actual city warps and the audio manifest match.
All 602 audio programs pass PCM hash, frame-count and loop checks.

The literal production-controller course passed all four wind lanes, wrong-end
entries, opposing input, both trainer victories, the leader, both city carpets,
re-entry and Continue. Its explicit setup uses Ampharos 31 / Graveler 30,
four prior badges and four Hyper Potions; it does not establish an earned
Minnesota-party journey or ordinary regional balance. Three additional local
controller cases passed loss/recovery, the prior-badge refusal and full gate
persistence with strong or losing fixtures. The Rochester, Mankato, Minneapolis and St. Paul local controller courses also
pass on this combined pack. Workspace formatting and all-target Clippy for
crystal-assets pass with existing warnings.

No selected-world pointer, hosted pack or user save has changed. Although
collision and positions are retained, saved block overrides can refer to old
private tile IDs and require migration before selection. The five designs remain
review candidates, and the other three Gyms remain to be improved and tested.
