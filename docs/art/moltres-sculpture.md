# Gold flame bird with curved, closed fire banks

`tools/moltres_sculpt.py` replaces Moltres's shared legendary-bird primitives
with an original individual sculpture. The battle-species builder dispatches
this pure recipe. All three original Kanto bird recipes now have their own
authoring paths, and the obsolete shared primitive route rejects execution.
Shared overworld props retain their separate sources.

Official [normal-form Moltres artwork](https://sg.portal-pokemon.com/pokedex/0146/)
was inspected visually in a temporary browser tab, which was closed after
review. No reference image was downloaded into source or the shipped bundle.
The [external source review](legendary-bird-rig-research.md) has not yielded a
validated usable open rig. This sculpture is original and static; no third-party
geometry was imported.

A slender gold breast and long narrow throat support a compact head, fitted
black eye sockets, narrow ivory lenses, small dark pupils and a long grey
pointed beak with a fine charcoal join. Each wing has a slim, curved gold
leading arm over a broad red-orange fire bank. The closed flame outlines have
irregular tips and scalloped lower edges. Their shallow curved surfaces support
broad orange cores and three smaller hot gold pockets on both sides. The head
has a three-lick crown flame with orange and gold cores on both sides. A broad
backward-swept tail flame has its own fitted orange and gold cores. Grey feet
have three forward digits, a rear digit and ivory talon tips.

The first review showed overly solid red flame panels. Replacing small isolated
orange patches with broad warm cores made the color transition clearer. Core
outlines are inset from actual vertical sections of the authored fire outline;
blindly scaling a concave outline would not preserve its narrow notches. The
stored-surface regression caught an orange cap crossing the curved parent
between fitted vertices. Denser cap subdivisions corrected the triangle-center
and edge clearance. Gold leading arms also have enough real depth to stay in
front of overlapping flame colors; an independent stored-ray check verifies
that clearance. The tail remains above the feet so grounding does not raise
both feet into the air.

Final Moltres has 62 parts and 75,968 triangles. Its presentation height remains
1.28 and its lowest point is zero. Its canonical SHA-256 is
`4448b4b3d118c282803b64e15ee35eb3ab6b62d76e23384dc456494bdfa2e915`.

The final candidate passed twelve stored-mesh tests in 59.617 seconds, and the
installed canonical source passed all twelve again in 47.913 seconds. Tests
cover connected closed outward solids, finite unit normals and actual face
winding, recipe parity, budgets, height, grounding and feature counts.
Independent nonadjacent triangle checks cover the body, neck, wing arms,
fire banks, fitted wing color layers, crown and tail. Stored-ray volume checks
cover neck/head/beak/crown roots, wing/body and fire/arm roots, tail/body roots
and the complete shin/foot/digit/talon chains. The grey feet and ivory claws
must remain close to the floor. Every fitted eye and fire color layer is
checked at vertices, front triangle centers and edge midpoints against the
actual stored parent, including the corresponding back-facing parents.
Independent stored-ray samples require the leading arms to clear overlapping
flame colors. Detached flame-wing and tail negative controls are rejected.

Final front, oblique and rear software images completed with exit zero and
were individually inspected. Canonical layout validation passed; storage
validation passed for the current shared tree of 573 raw JSON model documents.
All three edited Python files parsed, and the relevant Git whitespace check
passed. The installed source remains byte-identical to the reviewed candidate.

The first fresh native build and Rust all-251 regression exited 101 because
a shared gym-scenery initializer lacked `gym_stone_pixels`. The current shared
source supplies that field. The retried production build completed with exit
zero in 21 minutes 40 seconds including shared cache waits. The current all-251
Rust model regression passed with one matching test, zero failures, 1.16 seconds
and 1,266 filtered tests. Other package targets contained no matching tests.
All three current replacement source hashes were verified in that test binary.

The fresh production executable contains the full canonical source hash and
compiled mute switch. A muted owned-image GPU battle capture exited zero in
4.22 seconds; renderer write time was 0.94 seconds. The 2,000 by 1,400 image
has 47,915 colors and was inspected in full. Moltres's gold leading arms,
red-orange banks, warm flame cores, crown, grey bill, legs and ivory claws
render alongside the arena, Pidgeotto and battle HUD. Its tail is partly seen
behind the body; complementary software views cover its full outline. Model
hash and executable inode, size and modification time were unchanged before
and after the capture and across the three replacement trials.

Every native trial set `CRYSTAL_PREVIEW_MUTE=1` before process creation. The
backend sets both music and effect sinks to zero before appending sources and
keeps music refreshes at zero; startup music log messages do not mean audible
playback. No system volume or other application was changed. The separate
live-window capture issue and full 251-species appearance goal remain open.
Generated captures and logs are ignored verification scratch, not deliverables.
