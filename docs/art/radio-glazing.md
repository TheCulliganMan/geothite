# Radio studio glazing

Studio panes and their east returns are glass. Their header strips, upright
mullions, bottom rails and grille apparatus remain opaque. Doorways follow the
complete authored source and must never be extended by a guessed return wall.

The authenticated pane builders in `native_radio_studio` and
`native_radio_side` mark only glass body and reflection triangles. The ordinary
interior build then publishes those triangles through the existing blended
scenery path. The new source atlas is assembled from the already paletted live
tile samples, preserving the source UVs and colors; no second palette catalog
or exported asset is introduced. The immutable source images remain opaque.
Missing or invalid live pixels retain the existing opaque source drawing.

The glass body has a faint tint and the reflection surface has a stronger tint.
Physical end stiles preserve the pane silhouette independently of reflection
paint. Opaque wall/cap geometry, continuous side returns, source claims, floor
paint, support and collision keep their previous ownership. Vertex ranges
remain stable for the existing object-fade grouping; only referenced glass
triangles leave the opaque draw. Ordinary rooms are excluded.

Checks must cover both exposed sides, joined east walls, upper caps, the
fifth-floor doorway and ordinary movement. Inspect same-state 2D/native pairs
using headless image capture. Transparent glazing is a scoped material change;
it does not certify all room finishes or character/service behavior.
