# Required character rendering

The 3D world uses `open_people_skin::PeopleGpu` for every resolved human source,
indoors and outdoors, including remote players and bicycle riders. Each actor
instance owns a weighted `Person`; there is no optional legacy character field,
rigid joint renderer, or embedded human catalog.

Native applications load the shared artwork from `assets/open-models` by
default. `CRYSTAL_OPEN_MODEL_ROOT` overrides that directory. The required
`human-male` and `human-female` rigs must be present and valid. Optional unused
rig variants may be absent. UV paintings use the same default root, so the
production client and character reviewer load the same paintings.

Browser startup requires a complete production `open-people.json` bundle.
Missing downloads, timeouts, malformed bundles, or missing production rigs stop
startup. The web build script always bundles character artwork and does not
silently create a preview without it. Scenery remains independently optional.

The retired `catalog.glb`, rigid human decoder and animation implementation,
character generator, human GLB exporter, geometry compactor, and legacy
`characters.blend` source have been removed. Static bicycle files containing
baked old riders have also been removed. Bicycle frames are native Rust meshes;
riders use the same weighted character skin as walking characters. Their seated
pose has not yet received the full costume/limb collision and multi-angle
review required by the broader character quality work.

This removal does not mean every character is artistically finished. The UV
painting and anatomy review remains ongoing; see [character review](character-review.md).
It does not change gameplay, content packs, or audio.

## Validation of legacy human removal

The regression exercises all 77 resolved human source identities through the production weighted body/costume renderer and passes. Actor lifecycle, motion and prop checks pass (16 checks), mandatory-rig failure checks pass (2 checks), and bicycle triangle normals pass. The browser target compiled with mandatory person bundle loading. These checks cover renderer integration; they do not establish that the character art is approved or that gameplay is playable.

The subsequent Chris anime head refinement has its own passing regression for UVs, skinning, non-head body positions, accessory anchors and finite normals. Actual native four-angle idle/run renders and local-pack room screenshots are recorded in the [current art direction](stylized-character-faces.md). The rest of the character art remains intermediate and unapproved.
