# Crystalline ice bird with a continuous curling streamer

`tools/articuno_sculpt.py` replaces Articuno's earlier shared legendary-bird
shape with original individual anatomy. The main battle-species builder
dispatches this pure recipe and rejects the obsolete Articuno route. Zapdos,
Moltres and unrelated shared overworld props retain their existing sources.

The model has a slender upright blue cranium and throat, three crystalline
deep-blue crown blades, fitted ivory almond eyes, restrained dark pupils and a
short grey hooked bill. Its pale breast has feathered lower points and meets a
fitted pale throat collar. Raised bent wings have nine curved primary vanes and
seven smaller shoulder coverts on each of their upper and underside surfaces.
Grey feet have three forward digits, one rear digit and ivory talon tips.

The blue tail is one continuous closed elliptical ribbon with a broad curling
silhouette. Parallel-transport frames keep its cross section aligned through
the turn. Three quiet longitudinal folds fit to rays through the actual stored
ribbon triangles; they are not positioned at an assumed radial depth. The
curve opens smoothly rather than crossing itself at a tight turn.

Official [Articuno](https://sg.portal-pokemon.com/pokedex/0144/) artwork was
inspected visually in a temporary browser tab. No reference image is retained
in the repository or bundle. The first review showed the ribbon edge-on and
too much solid wing spar. The broad ribbon frame and shorter spar exposed the
individual vanes; the next stored-triangle test caught a tight-turn crossing.
A more open curl corrected it. Surface-fitting and a readable fold width
removed the broken subpixel channel appearance in the final software views.

The inspected [Master Galanodel model metadata](https://api.sketchfab.com/v3/models/e8c9a50843a5442483eadf414e8d7d65)
advertises a downloadable CC BY 4.0 model with five animation clips, described
as a Bedrock adaptation of Galarian Articuno. It is a different form from the
required Crystal ice bird; its skeleton has not been inspected and no geometry
was imported. The inspected [nguyenlouis32 Lugia metadata](https://api.sketchfab.com/v3/models/c07e5814167b48c887b688f8783e0b9b)
advertises downloadable CC BY 4.0 geometry with zero animation clips, but its
download endpoint returned HTTP 401. Its description supplies species lore,
not authoring provenance. No Lugia geometry or skeleton was retrieved. These
observations do not establish that no suitable open rig exists; the full search
remains active. This original Articuno sculpture is static.

Final stored Articuno has 109 parts and 63,816 triangles. Its presentation
height remains 1.28 and its lowest point is zero. The SHA-256 prefix is
`2db96ffe5e04a517`.

The final candidate passed all eleven stored-mesh tests in 15.912 seconds;
the installed canonical model passed again in 15.759 seconds. Tests cover
closed connected outward solids, finite unit normals and actual face winding,
recipe parity, preserved height and grounding, budgets and feature counts,
independent nonadjacent body/neck/streamer/feather crossings, actual volume
roots for crown, bill, tail and both banks of every wing layer, and complete
digit support. Fitted eyes and pale plumage are checked against stored parent
triangles at vertices, front face centers and edge midpoints. Streamer folds
are checked by independent 3D point-to-triangle distance. Detached-wing and
detached-fold negative controls are rejected.

All final front, oblique and rear software images completed with exit zero and
were inspected. Canonical layout and storage validation passed for the current
shared tree of 572 raw JSON model documents. The fresh native build completed
with exit zero in 8 minutes 38 seconds, including its shared-cache wait. The
Rust all-251 regression ran its matching test and passed in 1.49 seconds, with
1,249 other tests filtered from that crate.

The production GPU capture completed with exit zero in 3.10 seconds for the
whole verification command; the renderer reported writing the image in 0.47
seconds. Its 2,000 by 1,400 image has 48,301 distinct colors and was inspected
in full. Articuno's crown, pale bib, layered blue wings, grey feet and curling
tail are present alongside the opponent, arena and production battle HUD.
The battle oblique view partly occludes the tail behind the body; the separate
front and rear software inspections cover its continuous geometry.

Before launch, the executable contained the complete canonical model SHA-256
`2db96ffe5e04a51752eb20cac43c521f174a193433759ae2904806a925ed1ca6`
and the compiled mute switch. Model payloads are compressed in the build
output, so this verifies their raw-source fingerprint rather than expecting
literal JSON bytes in the executable. The executable inode, size and
modification timestamp and the canonical model hash stayed unchanged across
the capture. `CRYSTAL_PREVIEW_MUTE=1` was set before process creation; the
native backend sets new music and effect sinks to zero before appending audio
and keeps music-volume refreshes at zero. No system volume or other application
was changed. The capture explicitly targeted the owned GPU image.

This proves the replacement renders in that production image path. It does
not resolve the separate live native-window capture problem or establish that
every species looks finished. The full 251-species appearance goal remains
active. Temporary capture images, logs and Python caches are verification
scratch, not shipped model deliverables.
