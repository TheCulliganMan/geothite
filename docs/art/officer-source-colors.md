# Officer uniforms follow source palettes

The Officer's jacket, trousers, fitted cap crown and raised collar use the
pigment from the already paletted source sprite. Power Plant and Lavender
Radio Tower assign green; other objects use red or blue. Do not assign one
fixed blue uniform to the shared Officer model.

`npc_uniform` selects that pigment and repaints only reviewed clothing UV
islands. `open_people_skin` caches the variant by character kind and color;
body mesh, joints, bind poses, scale and source identity remain shared. The
Officer's trousers follow the same source pigment. Hair, skin, shoes, white
trim and transparent pixels remain protected.

`uniform_caps::append_with_color` changes the police crown only. Its visor,
band and gold badge keep their materials and its geometry remains identical.
`uniform_details::build_with_uniform` colors the raised officer collar,
retaining its existing geometry, skin weights, buttons, badge and dark seams.
Default callers preserve the original appearance.

The external-pack corpus regression covers all 33 Officer objects, their three
used palettes, four palette banks and six sprite frames. The actual-rig GPU
regression verifies isolated paints and head accessories with unchanged body
and attachment geometry. The live actor test verifies that changing a palette
refreshes only that actor, preserves the authoritative frame and player, and
does not rebuild the actor on ordinary animation frames.

Review evidence belongs in ignored `target/kanto-resume/officer-colors-review/`.
Use headless native captures paired with the same-state 2D renderer. A small
set of accepted uniform views does not certify every room or character detail.
