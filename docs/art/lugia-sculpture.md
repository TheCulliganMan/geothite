# Lugia sea guardian sculpture

`tools/lugia_sculpt.py` supplies an original dedicated battle model. The pure
recipe is dispatched by the battle-species builder. Lugia enters the exact
battle registry and leaves the shared-species list; direct `icon_lugia` routing
and the original icon mesh remain intact. A Rust regression checks both paths
and confirms the dedicated battle geometry differs from the icon.

The [official Lugia artwork](https://sg.portal-pokemon.com/pokedex/0249/)
was visually inspected as reference. No reference pixels or third-party geometry
were imported. The [Free3D listing](https://free3d.com/3d-model/lugia-pokemon-43982.html)
is unrigged, unanimated and under a personal-use license. The previously
reviewed downloadable CC BY Lugia listing did not provide an accessible model
or a validated skeleton. This original sculpture is static; usable open rig
research remains incomplete.

A pear-shaped white body carries a fitted pale blue belly, long curved neck,
compact head, narrow fitted eyes, backward-swept navy eye vanes and a white
occipital crest. Two bent wing arms support broad palms and five rounded,
volumetric digits each. Five pairs of rooted navy plates follow the back.
The continuous swept white tail ends with two navy fins. Powerful haunches,
ankles, feet and six blunt white toes ground the sculpture. The open roaring
jaw has a dark recessed lining and pink tongue with an actual empty gap.

Review corrected upright eye vanes to sweep backward and moved the recessed
mouth lining forward from inside the head. The final front, oblique and rear
software views were individually inspected. They show the mouth opening,
hand fans, back plates and tail from complementary angles.

The final source has 56 parts and 61,688 triangles, height 1.45 and floor zero.
Its SHA-256 is
`e1fa3b6aff4c09f956c5307960a1f03f4129ddbc583b528871a2eeede55e8990`.
The installed source is byte-identical to the reviewed candidate. The original
icon hash remains
`11728ee754b25b52dad479653c8d632e3788c4ae783a59ac50778f0613c1ad7b`.

Thirteen candidate tests passed in 6.701 seconds; all thirteen canonical tests
passed again in 5.256 seconds. Tests inspect stored triangles for closed
connected outward solids, face normals, recipe parity, height, grounding,
budgets and anatomical counts. Independent intersection and volume checks
cover body, neck, wing arms, fingers, eye vanes, crest, back plates, tail and
terminal fins. All hand digits must join palms, arms and body; all feet and
toes must have real roots and remain grounded. Fitted belly, mask, eyes and
pupils must follow actual parent skin at vertices, triangle centers and edges.
The mouth-gap check pairs stored ray crossings into occupied intervals and
rejects filler between upper jaw and lower jaw/tongue. It also rejects a
detached hand and requires the dark mouth lining to be visible ahead of the
head while recessed behind the upper beak.

The Lugia Rust routing regression passed with one matching test and zero
failures (0.02 seconds); other package targets contained no matching tests.
The current all-251 Rust regression also passed: one matching test, zero
failures, 1.16 seconds and 1,266 filtered tests. Its compiled test binary
contains the full current Moltres, Lugia and Ho-Oh source hashes. The fresh
production build exited zero after 21 minutes 40 seconds including cache waits.

The production executable contains the full canonical Lugia fingerprint and
compiled mute switch. The muted owned-image GPU capture exited zero in 2.65
seconds, with a 0.86-second renderer write. The 2,000 by 1,400 image has 16,072
colors and was inspected in full. Lugia's actual broad hand fans, head vanes,
open jaw and tongue, blue belly, feet, back plates and curving tail are visible
with the arena, opponent and battle HUD. The tail partly overlaps the body in
this angle; complementary software views cover its complete shape. Executable
inode, size and modification time and canonical model hash stayed unchanged
through all three replacement captures.

Every trial set `CRYSTAL_PREVIEW_MUTE=1` before process creation for both music
and effects. The backend sets new sinks to zero before source append and keeps
music refreshes at zero. No system volume or other app was changed. The separate
live-window capture issue and full 251-species appearance objective remain open.
Images and logs are ignored verification scratch, not model deliverables.
