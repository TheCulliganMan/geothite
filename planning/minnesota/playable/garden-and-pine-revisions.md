# Civic garden and northern pine edges

`content-packs/minnesota-canopy-review.crystalpack` is the current471-map route
review pack. Content identity:
`6f26934750dd991fcf33eb9f37b7fce3025c3198509db4c462605156c29a3f11`.
File SHA-256:
`7b375d6f676fc5a387b967b79e8afa35a94e13764b0064038ac0a1ad6542f7bb`.
It retains the preceding yellow prairie, Iron Range rocks, North Shore rivers,
basecamp and all earlier native route work.

## Sight changes

- **R04 Twin Cities:** the east rose bed extends one row toward the promenade;
  a north-connected elm divider separates its approach from the neighboring
  bench garden. The previous apron camera showed mostly plain paving and lawn.
  It now contains a coherent bed edge and tree boundary, with both the bench
  approach and continuous city path open. Nine metatiles change.
  [Whole](../renders/twin-cities/whole-route-d786421f.png),
  [native walking camera](../renders/canopy-review/native-r04-garden.png).
- **R06 Pine Country:** three explicit canopy strokes frame the northern
  junction, joining existing woodland on its northwest, southwest and eastern
  shoulders. The white road and lookout branch stay in place. Fourteen lawn
  metatiles change; ordinary native trees introduce no new Cut gate.
  [Whole](../renders/pine-country/whole-route-43404183.png),
  [native junction](../renders/canopy-review/native-r06-junction.png).
- **R07 Leech Lake:** the north pine lobe extends into two lawn metatiles above
  the arrival turn, bringing its edge into the walking view. T08's pocket and
  eastward landing/item branch remain accessible. This is a modest shoulder
  correction; it does not claim to redesign the entire lake circuit.
  [Whole](../renders/leech-lake/whole-route-before-ed510dae.png),
  [native north turn](../renders/canopy-review/native-r07-elbow.png).

Root inspected all three complete maps, the earlier weak views, before/after
walking pairs and the three actual noon640×576 native captures. The native
captures are headless; all originals, raw readbacks, rejected passes, crops and
conversions remain retained. User visual acceptance and the full regional
quality/pacing goal remain open.

## Exact preservation and production input

The [immutable comparison](../renders/terrain-revisions/1e6310ded1e3-to-6f26934750dd/comparison-bf921cf4921acc7bf989c44859afd777469e2ede07001ebc48bc235527519f34.json)
passes: only five existing exterior block arrays, their matching base64 payloads
and content identity differ. The other466 maps and every other field/scalar type
remain exact, including actors, parties, scripts, encounters, art, attributes,
rooms, connections, manifests, audio and HOME/recovery. Twenty-five metatiles
change in total. Source/native whole terrain pixels match. Nine before/after/
dossier source files are protected in the comparison's `sources/` directory.
Eleven adversarial checks reject unintended changes.

The first comparison correctly refused R04's fence recipe because its reviewer
had not applied the existing source `collision_order:[2,3,0,1]`. The native Rust
art builder and both packs already contained the correct wall/floor quadrants.
The reviewer now validates a four-quadrant permutation and compares its exact
result; wrong-order and duplicate-order mutations reject. No game geometry or
native fence collision changed to resolve that proof-tool defect. The rejected
comparison remains beside the successful report. There are156 retained proof
images plus intermediate render stages.

**Eight production-controller cases pass on this exact pack:** R04 geometry
and the populated both-direction grass/battle journey; R06 populated five-trainer
journey, native Cut denial/opening/reload, and Surf boundary/seam; R07 populated
eight-trainer round trip, native Cut denial/opening/reload, and Surf boundary/
seam. They retain selected PP mutation, resources/services, ledges, rooms where
present and save/Continue. Position, partners, badges and RNG are explicit
fixtures; these tests do not establish earned connected regional progression.
Logs: `.local/canopy-r04-controller.log`, `canopy-r06-controller.log` and
`canopy-r07-controller.log`.

Fresh R04/R06 journey saves load through the actual stdio MCP executable. Start
shows PACK, B closes it and native movement changes R04 `(4,18)`→`(4,19)` and
R06 `(51,32)`→`(51,33)` on their first chunks. Both stderr streams are byte-empty
(`.local/canopy-mcp.log` and observations). All377 audio programs match PCM
hashes, frame counts and loop ranges (`.local/canopy-audio.log`).

## Replay

Use the protected source triples and apply R04, then R06, then R07 with the
existing Rust `revise_authored_route` binary, beginning with the preserved
`content-packs/minnesota-prairie-review.crystalpack`. Intermediate verified packs
are retained in `.local/minnesota-canopy-revision-stages/`.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_authored_terrain_revision.py \
  --before content-packs/minnesota-prairie-review.crystalpack \
  --after content-packs/minnesota-canopy-review.crystalpack \
  --revision .local/r04-canopy-before.layout.json planning/minnesota/routes/MN_R04.layout.json planning/minnesota/routes/MN_R04.json \
  --revision .local/r06-canopy-before.layout.json planning/minnesota/routes/MN_R06.layout.json planning/minnesota/routes/MN_R06.json \
  --revision .local/r07-canopy-before.layout.json planning/minnesota/routes/MN_R07.layout.json planning/minnesota/routes/MN_R07.json \
  --self-test
```

The connected atlas still distinguishes two published towns/25 rooms from the
city source designs and planned travel. Three new city Rust builders are under
independent asset/controller integration; they are not installed by these
terrain-only revisions. Glacier Line and fairgrounds work remains in scope.

A separate `minnesota-northstar-canopy-review.crystalpack` also applies these
exact three terrain revisions to the later Northstar Q06/Q07 story candidate.
Its Rust publication and strict written-pack loading pass. The additional
[exact comparison](../renders/terrain-revisions/ddbc5521227a-to-fd5ebeb28cb9/comparison-2e96a6fd9d0a7c7dbd9e52a98be3990e8a6818cae7008c09247596a7342886e3.json)
also passes: only those25 terrain metatiles, matching block payloads and content
identity differ from the Northstar story pack. All other fields and scalar
types remain exact; eleven adversarial checks reject unintended changes and
156 additional proof images are retained. Combined earned-story play remains
open. It is the prepared input for the
current city/gym integration, not a replacement proof for the route-only pack.
