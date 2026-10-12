# Flowing goldfish and plated carp sculptures

`tools/goldfish_sculpt.py` replaces the shared generic fish with original
Goldeen, Seaking and Magikarp anatomy. The main battle-species builder dispatches
the pure recipes and rejects the obsolete shared construction. No external
geometry, extracted game source, artwork or content pack is shipped.

All three have a continuous cheek mantle and tapered rear body, an annular
snout, a rounded pink oval lip with a genuine central hole and a recessed dark
oral floor. The mouth is empty space in the stored geometry, rather than a dark
sphere placed over a solid face. Eyes and markings follow actual stored parent
triangles. Thin closed fins have curved outlines and supported folds.

Goldeen has a white body, orange forehead saddle, teal lower irises with sleepy
orange lids, an ivory forward lance, broad scalloped split caudal fans and
swept pectoral paddles. Orange fin washes appear on both faces. Seaking has an
orange body, a white lower chin, broad glossy eyes, two hooked dark flank
saddles, dark spots and gill crescents, spotted white caudal and pectoral fans,
and two spotted lower streamers. Its fin markings are present on both faces.

Magikarp has a red mantle, tiny vacant pupils, a tall open oval mouth, two
continuous curved golden barbels, a three-point upper gold crown and inverted
lower crown. Six staggered plate seams wrap the actual flanks. Its single
vertical forked caudal sail has a red border and supported ivory surfaces on
both faces; white pectoral paddles have red leading rims. Inspection changed
its first paired tail fans to this individual vertical tail silhouette.

Official [Goldeen](https://sg.portal-pokemon.com/pokedex/0118/),
[Seaking](https://sg.portal-pokemon.com/pokedex/0119/) and
[Magikarp](https://sg.portal-pokemon.com/pokedex/0129/) artwork was inspected
in temporary browser tabs as a reference; no artwork is retained in the bundle.

[Katerina Novakova's Magikarp](https://sketchfab.com/3d-models/pokemon-magikarp-aed89b4ee4f1421abc2b1613911d0ca8)
is described by its creator as sculpted, retopologized and texture-painted in
Blender, with diffuse and normal maps. Its
[public metadata](https://api.sketchfab.com/v3/models/aed89b4ee4f1421abc2b1613911d0ca8)
reports downloadable CC Attribution-ShareAlike 4.0 geometry, 26,496 faces and
zero animation clips. Zero clips does not establish absence of a skeleton.
The download endpoint returned HTTP 401, so neither geometry nor rig could be
inspected; no external model was imported. These original sculptures are static.
The full 251-species appearance and usable rig search remain active.

Final stored counts are 36 parts / 66,788 triangles for Goldeen, 70 / 51,716 for
Seaking and 30 / 51,680 for Magikarp. Presentation heights remain 0.80, 1.00 and
0.90, with each model's lowest point at zero. SHA-256 prefixes are respectively
`46003d092e7c8837`, `1a5620320d128144` and `c610ffc7d9e6afa4`.

The final candidate passed all eleven stored-mesh tests in 93.709 seconds.
Checks cover closed connected outward solids, finite unit normals and actual
face winding, recipe parity, height, floor, budgets and unique parts, independent
nonadjacent body face intersections, actual snout/lip/floor attachment, and
empty mouth space across every stored part. A filled-mouth negative control is
rejected. Supported eyes, markings and tail insets are checked against actual
parents, including triangle centers and edge midpoints. Fin, horn and barbel
roots are checked as actual intersecting geometry. Fin folds, leading rims and
carp plate seams use independent point-to-triangle distance to their stored
parent surfaces.

All final front, oblique and rear software renders were inspected. An early
white eye layer clipped slightly; finer fitted geometry and clearance corrected
it. Magikarp's tail inset was refined after a visible artifact. Its single
vertical caudal sail, red border and ivory inset now read clearly from both
oblique directions. Opposite-face goldfish fin markings are supported by the
actual reverse fin surface rather than duplicated at a guessed depth.

Native trials use the current Rust sink-level `CRYSTAL_PREVIEW_MUTE=1` switch
with a fresh executable. It sets both music and transient effects to zero volume
before source playback and preserves timing. No system volume or other
application is changed. These art changes do not alter audio synthesis or PCM.

The installed canonical meshes passed all eleven tests again in 82.474 seconds.
Layout and storage validation passed for the current shared tree of 571 raw JSON
model documents. The fresh native build completed successfully in 23m 07s,
including shared-cache waiting. The production Rust all-251 stored-model check
passed its one matching test in 2.18 seconds (1,245 filtered out).

All three production GPU battle captures completed with exit zero, using one
unchanged freshly built executable with its compiled mute switch verified.
Each full 2000-by-1400 image was independently inspected: Goldeen has 37,143
colors, Seaking 47,294 and Magikarp 42,570. Their individual anatomy is visible
in the scene, with arena, opponent and production battle HUD present. Goldeen's
sleepy teal gaze and orange fin roots, Seaking's hooked flank saddles and spotted
fans, and Magikarp's tiny pupils, barbels and red-bordered single vertical tail
read in the production camera. The mouth openings and fitted face layers remain
visible without detached eye spheres.

These trials use the explicit owned GPU image capture target. They do not
resolve the separate live native-window capture problem or establish gameplay
coverage. Temporary verification images and logs are removed after inspection.
The complete 251-species appearance goal remains active.
