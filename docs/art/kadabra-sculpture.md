# Kadabra sculpture and skin

Kadabra retains the detailed original sculpture and reviewed retopology, with
four targeted anatomical repairs. The neck extends into the skull, closing the
old visible gap. Both moustache halves gain hidden roots inside the muzzle.
The brown tail band is derived from the actual tail triangles instead of an
approximate overlapping tube. The other 56 named parts preserve their exact
reviewed f32 geometry, normals, indices and colors.

`tools/kadabra_glb.py` writes the canonical `kadabra.glb`. The old static JSON
is removed. Following the existing Gengar pattern, this GLB also retains its
reviewed neutral authoring input. The default Blender builder calls the
canonical exporter and reproduces the full skin byte for byte. The detailed
`sculpt_kadabra` recipe remains editable in `tools/build-battle-species.py`;
`--only kadabra --review-kadabra-source` builds its detailed source and candidate
JSON in the chosen ignored review directory. That explicit review mode does
not replace the canonical retopology. New geometry requires a fresh review and
new pins; it is not silently accepted from a nondeterministic remesh.

The tail-band helper clips the stored tail surface between two cross sections,
offsets a welded normal field by 0.0018 authoring units and closes the two rims
inside the tail. The gold core stays untouched. Band geometry is reproducible
from the canonical tail and follows the same two-carrier weight field. The
repaired neck and moustache roots are also present in the detailed source.

All 60 logical parts remain, including the five-point red forehead star,
focused gaze and brows, three pink abdomen waves, asymmetric arm pose, six
curled fingers with ivory claws, six long hind toes with ivory claws, swept
moustache halves and the held spoon's hollow bowl. Lossless material packing
combines coherent hand, foot and marking groups into 38 runtime primitives;
source ranges restore all original parts and preserve their actual weights.

The final skin has 4,776 vertices, 5,848 triangles, 18 joints, 223 accessors and
293,136 bytes. Authoring height stays 1.12; verified pack height stays 1.2954 m.
The existing 20,000-vertex, 40,000-triangle, 64-primitive, 256-accessor and 1 MiB
limits remain unchanged. Rest geometry SHA-256:
`465503e6eeaaa78d03ec1985ed08c33fb2bea8dd2097d449bb829c5f3ee5e6a2`.
Packed runtime geometry SHA-256:
`d027eeb4ab22721959745884fec132d2073f9ed1671cb3088292166a42360f58`.

The rig has an identity floor root, torso, blended neck, coherent skull/face,
two three-joint arm chains, two leg carriers, independent ears and moustache
halves, two tail carriers and a spoon carrier parented to the raised wrist.
The spoon pivots at its grip and remains supported by the hand. Moustache
ownership follows anatomical part identity, including hidden root vertices
that cross the center plane. Complete feet, toes and claws stay fixed in battle;
world walking uses two complete support chains and preserves prop/tail weights.
Idle uses 81 keys over 3.2 seconds; attack/hit use 65 keys over one second,
with source battle cues owning timing and neutral endpoints.

All nine Python checks pass: strict canonical bytes and mutation rejection,
lossless ranges, exact unchanged geometry, hierarchy/weights/binds, closed
outward solids, authored triangle shading, actual anatomical root occupancy
through clips, spoon support and bowl/stem contact, fixed floor, winding and
bounded strain, independent carriers, and reproducible fitted band geometry.
A negative control restores the old short neck and proves its gap against the
actual skull. Final front, side and rear old/idle/attack/hit sheets were inspected
as software art previews. Native paired review evidence follows. Shiny
checks remain skipped, grass height is untouched, and browser acceptance is open.


The three focused Rust checks passed against the final skin (0.40 seconds):
GPU activation at the verified 1.2954 m pack height, authoritative rig sampling
and bounds with fixed complete feet and anatomical moustache owners, and two
world support chains through 64 phases with tail/moustache/spoon weights
preserved. The native example was rebuilt before capture.


Both shared all-species geometry and exact source-cue timing checks passed
(2 tests, 124.45 seconds). The rebuilt native Kadabra/Hoppip Route 29 pair was
inspected at preserved state: frame 2921, checksum 3542893732, LCD frame 3623
and virtual time 63.956251295. The connected neck, rooted swept moustache,
forehead star, focused eyes, three pink waves, curled fingers, raised held
spoon and continuous tail band remain readable. Both clawed feet meet terrain,
and the HUD says KADABRA. Normal terrain admission passed at the original
1.2954 m pack height and existing -110-degree review yaw, with normal support,
omission, whole-body and frustum checks active, no fallback and no shader errors.
Ignored evidence: `target/pokemon-native-review/kadabra-normal`. Browser visual
acceptance remains open. Shiny checks and grass-height edits remain skipped.
