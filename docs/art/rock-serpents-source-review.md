# Onix and Steelix source review

The [official Onix](https://sg.portal-pokemon.com/pokedex/0095/) and
[official Steelix](https://sg.portal-pokemon.com/pokedex/0208/) pages returned
200. Their named primary artworks were individually visually inspected in
background image-only tabs; no reference media was copied into the repository.

Onix has a long tapering chain of many irregular grey boulders, a wedge-shaped
rock face, narrow fitted ivory eyes, a pointed nose ridge, a dark open jaw and
one long narrow forehead horn. The primary art arches the large upper rocks
behind its head and tapers toward many small tail stones. Steelix has a wide
angular metal head, a broad low jaw and block-like tooth divisions, narrow
ivory eyes with red irises, irregular metal body segments, alternating long
body spikes and a pointed tail. The old Steelix model's two head prongs are
absent from the primary art.

The paired baseline software render of `actor_props/big_onix.mesh.json` and
`battle_species/steelix.mesh.json` exited zero and was fully inspected. Onix
has only seven simple polyhedra, tiny protruding eyes and a broad rectangular
jaw. Steelix has a smooth oval head, oversized protruding eyes, two incorrect
head prongs, simple rocks separated by exposed dark cylinders, and no proper
toothed jaw. Both need individual anatomy. Onix's earlier native review also
flagged apparent ground clearance; any replacement must be checked in the
actual battle placement, not just normalized to floor zero in software.

Primary Sketchfab metadata and archive endpoints were checked directly:

- [lucasprs51450 animated Onix](https://api.sketchfab.com/v3/models/905c217897e14657891c32e34c58bf3f):
  metadata 200, downloadable flag, CC BY-NC 4.0, 11,460 faces and one listed
  animation. Its description advertises flowing animation and a black outline.
  The archive endpoint returned HTTP 401, so bones, skin and clip were not
  available for inspection. The license restricts commercial use.
- [Beany-Boy Onix](https://api.sketchfab.com/v3/models/1c2a3a41f54445a492e379ab2565a906):
  metadata 200, downloadable flag, CC BY 4.0, 610,136 faces and zero listed
  clips. The description identifies a personal modeling project. Its archive
  endpoint returned HTTP 401.
- [davisanand Steelix](https://api.sketchfab.com/v3/models/9b04ffe8729b41c6aef21c20a6d7b2f9):
  metadata 200, downloadable flag, CC BY 4.0, 1,851 faces and zero listed clips.
  The description is brief appreciation of the species; its archive endpoint
  returned HTTP 401.
- [nguyenlouis32 Steelix](https://api.sketchfab.com/v3/models/0a14c5de826c4fb4b9f152369b5a9f83):
  metadata 200, downloadable flag, CC BY 4.0, 1,312 faces and zero listed clips.
  The description is species lore; its archive endpoint returned HTTP 401.

No external archive, geometry, rig, skin or animation was retrieved or imported.
A listed animation is evidence of a clip listing, not verified accessible rig
contents. Zero listed clips alone does not prove a missing rig. These checks
cover specific candidates and do not prove no usable open model exists.
