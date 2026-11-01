# Four-route visual audit: River Bluffs, Pine Country, Red River Prairie, Border Pines

Read-only sight review on 2026-10-26. Terrain, dossiers, Rust and packs remain frozen. The two corrections below are proposals for selection, not implemented revisions or visual approval. This review does not infer quality from test counts, asset counts or a passing graph.

All whole maps were inspected at their full proportions, then the weak camera and contextual views listed below were inspected. The 160×144 files are source-art player framing, not proof of controller input. Existing production captures were also viewed where noted. No new render, crop or conversion was needed; every cited whole/weak image already has its exact encoded-byte SHA-256 copy in adjacent `history/`. No image was overwritten or deleted.

## Authentic references inspected

- [Route30](../renders/source/Route30.png): pine boundaries interrupt long vertical travel; short side clearings and encounter patches remain legible from ordinary route framing.
- [Route45](../renders/source/Route45.png): the rock ribs, inset shelves and terrain openings stay present in the player's view. The broad geology serves a much narrower navigable lane rather than becoming a full camera of tan floor.
- [Ilex Forest, day](../renders/source/IlexForest-day.png): canopy edges, modest clearings and visibly enclosed side discoveries. Its native hut and tree silhouettes are the relevant control for Border Pines.
- [Route34](../renders/source/Route34.png): genuinely wider grass and road areas can work when habitat, fences, buildings or a useful junction explain them. This is not a rule to shrink every map.

## MN_R05 River Bluffs — select a bounded upper-bench improvement

Evidence: [current whole, revision48191ad8](../renders/river-bluffs/whole-route-48191ad8.png), [weakest160×144 camera](../renders/river-bluffs/viewport-emptiest_ground-native-rgb.png), and [actual native middle-switchback capture](../renders/river-bluffs/native-middle-switchback.png). The whole encoded SHA-256 is `a292035c840e468db57cbcaa9619d07489795cff7b93e61b0b12c1f73ff9f154`; weakest-camera SHA-256 is `4556f2997f5d736b147b58af2fcb17491d9c581c1c856c18f4745056f1f63c15`.

The two-ridge silhouette, usable cave mouths, southbound drops and visible river terrace read clearly as a bluff route. The specific defect is the **upper ridge around core `(23,21)`**, equivalent to meta `(11,10)`. The current weakest camera is almost entirely tan, with a straight white path on its right and bottom. No cliff rib, landmark, actor or decision explains that amount of exposed plateau in this frame. The northwest boulder cluster is elsewhere; adding another scattered rock would not solve the uninterrupted bench.

Proposed correction: reshape only `MN_R05.layout.json`'s first `Upper limestone ridge` polygon with a northern forest cleft between the existing northwest knoll and the vertical ascent. Replace its opening edge `[[5,7],[14,7],...]` with:

```json
[[5,7],[9,7],[9,11],[11,11],[11,7],[14,7],
 [14,13],[20,13],[20,20],[16,20],[16,17],[10,17],[10,20],[5,20]]
```

These are **metatile** vertices. The inset draws actual native cliff faces around the narrow cleft; it does not paint tree props across a plateau after the fact. Keep the vertical white ascent at meta x12, the horizontal switchback at y12, both native ramps, both cave-mouth/return records, both drops, all actors/items/flags, water closures and regional ports. The cleft's right edge is x11, leaving the x12 walking strip intact; extending it to x12 would risk replacing that strip with a cliff face and is not this proposal.

A read-only in-memory raster of this proposal left the existing ramp/drop/room and selected through-path block coordinates unchanged. That is only a preliminary compatibility check. Before accepting it, render the entire route and the same core `(23,21)` camera; inspect the concave native cliff corners and the northwest knoll's approach. Recheck actual actor approaches, required grass, ordinary returns and native port/room/controller contracts after selection. No source was changed for this audit.

## MN_R06 Pine Country — frame the northern junction with connected canopy

Evidence: [current whole, revision2d6709de](../renders/pine-country/whole-route-2d6709de.png), [weakest160×144 camera](../renders/pine-country/after-cut/viewport-emptiest_ground-native-rgb.png), [actual native northern-road capture](../renders/pine-country/native-northern-road-weak.png), and [bird-glade view](../renders/pine-country/viewport-north_bird_glade.png). The weak crop comes from the after-Cut render, but this northern ground is unchanged by that gate. Whole SHA-256 is `04c19804f22e915dd45775c9c631e2378e1ce8a34c0a9ecd3e0fc51aad06d0a6`; weakest-camera SHA-256 is `9039f844c09dba73c4972a1f4bb048e6e6fe3dce9d3e2c7d1657903c1950b24b`.

The lake, necessary outlet bridge, camp service, fishing pier and conifer battle/grass fork make distinct places. The specific weak spot is **core `(39,15)`**, around meta `(19,7)`, at the northern lookout-path junction. Its160-pixel frame contains a white road cross surrounded by uninterrupted green shoulders. The larger native view confirms that this is an ordinary approach between islands of pines, not a purposeful activity clearing. Route30's comparable route framing keeps a useful canopy edge or transition in view. The main road is already narrow; expanding its scenery with more detached objects would not address the shoulder.

Proposed correction: before the existing road paths, add three deliberate ordinary-canopy `forest` rectangles with **half-open metatile bounds**:

```json
[18,5,20,7]
[18,8,20,10]
[22,5,24,8]
```

The first extends the existing northwest shoulder down to the junction; the second joins its southern edge to the existing west pine shoulder; the third connects the northeastern shoulder to the existing northern tree island. This frames the cross with actual connected woodland while keeping the existing white path at meta y7/x17–21, main road at x21, western path at x17, and all existing destinations. Use ordinary native canopy, not a Cut-capable variant. Route dimensions stay32×44; no identities, actors, rewards, story, water, ports, room or field-move contract changes.

The read-only in-memory candidate preserves those existing road blocks and the Cut/ramp/ledge blocks. It has not been rendered or accepted. The selected pass must inspect the same core `(39,15)` frame, both directions through the junction and the full silhouette, then preserve all actual gate/actor/service/grass and source/native contracts. This should remain a shoulder correction, not another route redesign.

## MN_R16 Red River Prairie — no selected terrain change from this pass

Evidence: [current whole, revisiona8b52fde](../renders/red-river-prairie/whole-route-a8b52fde.png), [plainest160×144 farm-lane view, core `(75,30)`](../renders/red-river-prairie/viewport-widest_farm_lane-native-rgb.png), [western shoulder, core `(37,25)`](../renders/red-river-prairie/viewport-widest_west_shoulder-native-rgb.png), and [computed emptiest view, core `(34,26)`](../renders/red-river-prairie/viewport-emptiest_ground-native-rgb.png). Whole SHA-256 is `a003923c41327d162502fe1813551ad9100c3b2dd367e35689b4daa4911715c0`; farm-lane camera SHA-256 is `444ed29ba42b2c7ba212f5a19b7103d325cc23cfa1a7eb0318d447c22dd8602f`.

The large river, second drainage crossing, raised hayfield, flood-meadow grass and fence-bounded cultivation give the whole a horizontal prairie identity. The farm lane is its quietest frame: open white floor occupies much of the view, but its upper fence, lower habitat and cottonwood edge still explain a cultivated field junction. The wider western shoulder likewise shows the field edge and fence instead of becoming featureless terrain to every side. This is a weaker local camera than the river crossing, but the available views do not justify a new obstacle, forced encounter strip or arbitrary width reduction. Adding a quota of trees would dilute the open prairie character.

Leave the source frozen in this pass. A later played approach may identify a specific excessive walking segment or unclear fence opening; that should be addressed at its actual decision point. This is not blanket approval of the route or proof that every activity and regional connection works.

## MN_R17 Border Pines — no selected terrain change from this pass

Evidence: [current whole, revision6c5bbcb6](../renders/border-pines/whole-route-6c5bbcb6.png), [ranger-yard raw160×144 camera, core `(44,53)`](../renders/border-pines/stages/review_border_pines-render-line99-25-crop-bounds-624-776-784-920-result.png), [quiet-spur raw160×144 camera, core `(29,44)`](../renders/border-pines/stages/review_border_pines-render-line99-25-crop-bounds-384-632-544-776-result.png), and [actual native ranger-yard capture](../renders/border-pines/native-ranger-yard.png). Whole SHA-256 is `46a3852d63fe70466588d0c06ed639b62ead84f98ee7a441ad046f82956693c5`; yard-camera SHA-256 is `30f906b0f2d5310bbd40f2c1fd41a3394b9438269b4067e41f69e28cba161ae7`.

The widest ranger yard remains the weakest substantial clearing, but its native camera contains the hut door, sign, ranger, flowers and a connected canopy edge. Its width supports the actual rest destination and passing space. The quiet two-tile cross-forest spur is visually spare, yet the sign occupies the junction and names its two real destinations: den west and hut east. It reads as a connector rather than an unexplained lawn. The full route also distinguishes the marsh crossing/view deck, narrow optional return, old-pine habitat and den branch.

I do not see a new bounded terrain defect strong enough to justify changing the frozen source here. Preserve the actual native Cut return, ordinary pre-Cut access to the cache/watcher, both room returns and the required habitat. Do not narrow the yard into its service approach or add a ledge/Cut gate as decoration. As with R16, this conclusion is limited to this sight review, not user acceptance or completion of the regional journey.
