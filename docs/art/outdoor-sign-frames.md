# Outdoor sign frames and live inscriptions

Four original authored sign families replace the remaining 87 complete outdoor sign drawings: 348 source cells across 40 base maps. This count comes from matching the exact residual source identities in the external pack and the prior authored audit; it is not a visual signoff. The 296 Kanto cells form broad, low twin-post civic boards; 24 modern Johto/Battle Tower cells form taller enamel wayfinders; 24 park cells form green framed signs; four Ilex Forest cells form a braced cedar reading sign.

Each sign has closed posts, feet, structural rails, backing, shaped crown, bevels and fasteners, with a separate rear silhouette. The source lettering is intentionally retained as a live cropped surface inside the modeled recess. The adapter never replaces text with decorative lines or bakes source pixels into the model. The assets contain original geometry and materials only. Source dialogue, collision, warps and actor footing remain authoritative and unchanged.

## Source ownership and fallback

The adapter binds ten complete 2×2 drawings by atlas, metatile, source phase and all four tile identities. A single revised, missing, rephased or occupied cell rejects the whole drawing. No map allowlist forces signs onto unrelated art. Every existing or custom applicable live profile owns its matching source identities, even when a viewport crop or missing ground prevents that larger profile from resolving. Cinnabar's existing eight sign cells retain their current model, and the already-authored Johto atlas sign family is outside this kit.

Only verified backing from the same atlas and metatile can supply ground. Kanto's mixed checker ground keeps the native 30/39 alternation; park signs retain the native 00/16/06/00 plaza pattern. Modern signs preserve lawn, path and brick backings. If one required backing sample is absent, the entire sign falls back. A crop can therefore deliberately keep the old presentation; missing evidence never invents a floor.

All geometry stays inside the original 2×2 plot and the lower source edge. Rendering claims those four cells once. It does not consult or edit events, collision, scripts, map data or warps. The current scene atlas supplies the face UVs every frame, preserving palette and current texture content.

## Authoring and checks

`tools/build-outdoor-sign-frames.py` creates the four assets and one editable Blender source with named component meshes and materials. The source is distributed through the existing hash-verified `art/johto/source` chunks. `tools/check-outdoor-sign-frames.py` verifies bounded runtime models, positive solid volumes, six face directions, material groups, normalized live face metadata and 429 front-ray samples per inset to reject solid geometry that would hide lettering. Every Blender component is checked for manifold edges and positive volume before export. Compressed runtime JSON uses the existing lossless model storage format.

The four assets total 5,988 prototype triangles and 69 closed components. Runtime JSON totals 121,324 compressed bytes; the editable Blender source is 166,932 gzip bytes in three verified bounded chunks. Front and rear authoring previews were inspected; the inset ray checks and isolated source-guard harness pass. These are distinct from native integration checks.

The focused Rust tests cover all ten patterns, all five identity fields of each sign cell, any preowned cell, foreign/same-number grounds, checker and park backing order, complete and cropped custom profile overrides, existing Cinnabar exclusions, unchanged footing/source identities, exact text UVs, non-square grid metrics and shifted origins, repeated append, changed post-resolution art and partial canvas rejection.

## Native review status

Integrated Rust tests and the full-map production audit pass. Ten native views cover both Kanto backings, modern Johto, the tower, forest and park families. A roof-occlusion defect found in Azalea was verified with default, side and reverse player-reveal views. Authored building ownership now feeds [whole-object translucency](../occluder-translucency.md), replacing the former capsule-shaped hole while preserving source backing and sign geometry. Authoring previews prove geometry only; they intentionally show blank inset panels because game lettering remains live. They do not prove actual palette, text sampling or gameplay.

Review front, oblique and rear views in CeladonCity/PalletTown (dotted/checker Kanto backing), AzaleaTown/GoldenrodCity (grass, path, brick), BattleTowerOutside, IlexForest and NationalPark/NationalParkBugContest. Verify lettering stays within the frame while scrolling, adjacent objects are not consumed, foot placement and actor occlusion look correct, custom/cropped scenes fall back whole, sign interaction opens the same dialogue, and nearby exits/warps still work. This kit closes only the specified outdoor-sign source family; indoor directories and other remaining room families are separate work.
