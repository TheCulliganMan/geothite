# Furret continuous banded skin

The production baseline was inspected against its matching 2D state. Its
protruding eyes, stacked body/tail bands, visible seams, and cream palette
require refinement. The replacement recipe in `tools/furret_sculpt.py` uses
shared-ring colored sections with matching exterior normals, fitted eyes,
glints, four fitted brown cheek marks, nose and mouth, a broader head, outward-swept pointed ears, small
forepaws, grounded hind paws, and a continuous curved tail silhouette.
Anatomy is checked against the pack sprites and
[official Furret artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/162.png).

The verified pack gives a height of 5 feet 11 inches, or 1.8034 metres, and
RGB555 palette anchors `(23,21,4)` and `(14,9,2)`. Their RGB8 expansions are
`(189,173,33)` and `(115,74,16)`. The normalized draft height is 1.20. The
current draft has 33 parts and 11,866 vertices, within the existing limits.

Each colored body or tail section is closed. Neighboring sections share exact
ring positions and exterior normals, avoiding inflated overlay tubes and
exterior shading seams. Hidden end caps retain their own flat normals.

All four independent draft regressions pass:

```sh
PYTHONPATH=tools python3 -m unittest test_furret_sculpt -v
```

They check closed outward geometry, shading, shared seam positions/normals,
pack palette anchors, floor contact, geometry budget, and face clearance
against the actual colored body triangles. The first neutral render prompted
a broader head and less upright ears; the revised oblique render was inspected
again. References and neutral review output stay under ignored `target/`.

`tools/furret_glb.py` now exports a fourteen-joint standard skin. A pelvis,
chest, neck, and head chain drives the rearing body. Paired ears, forepaws,
supporting legs, and a three-stage tail have dedicated joints. Hind paws stay
on the identity floor root. All colored body/tail seam vertices use identical
skin influences. The fitted face shares the body's position-dependent weight
field rather than separating from the neck during a bend. Idle, attack, and
hit clips use rotation keys and return to neutral at both endpoints.

The draft GLB preserves neutral digest
`23d49398710d1399a28120ee0734869065766ebb09a5d9b19b7b48edb4f9565b`.
It has 33 parts, 11,866 vertices, 10,372 triangles, 14 joints, and 169 accessors
in 461,276 bytes. The export and all nine stored-model regressions passed:

```sh
PYTHONPATH=tools python3 tools/furret_glb.py \
  --output target/pokemon-native-review/furret-rig.glb
FURRET_GLB=target/pokemon-native-review/furret-rig.glb \
  PYTHONPATH=tools python3 -m unittest test_furret_glb -v
```

These tests additionally check decoded hierarchy and ownership, exact seam
weights, floor clearance, winding, edge strain, neutral endpoints, and body,
ear, forepaw, and tail articulation. Neutral and attack poses decoded from
the stored GLB were rendered and inspected.

The canonical skin replaces the production Furret JSON and is registered with
both the exact-species model catalogue and the standard battle skinning path.
The focused Blender builder publishes this same GLB. Ten combined canonical
Python checks passed. A metadata correction describing the two planted hind
paws preserved the binary geometry, normals, weights and clips byte-for-byte;
canonical-byte and route checks passed again, and the real focused Blender
builder matched the final GLB exactly.

Two production Rust checks passed: registry playback keeps the hind paws fixed
while the body, ears, forepaws and tail move, and GPU skinning uses fourteen
joints at the verified 1.8034-metre pack height. The schema check passed for all
68 standard rig variants, and all 44 GPU skinning checks passed. The full
model authoring check passed after correcting the verifier's explicit Unown
form lookup: 59 editable Blender sources, 609 runtime models in 609 files,
and 730 embedded references.

A fresh native build produced paired 3D/2D normal and shiny captures from the
production battle renderer. Both pairs were inspected and preserve their own
state. Normal: checksum 1267042444, core frame 0, LCD animation frame 56,
virtual time 1.378591333 seconds. Shiny: checksum 2060605118, core frame 0,
LCD animation frame 47, virtual time 1.267893291 seconds. The normal and shiny
fixtures have independent DVs and stats; preservation applies within each pair.
The full silhouette fits the frame, supporting paws remain grounded, the body
and tail bands stay continuous, and the small eye and cheek details remain
readable. Source gold/brown and shiny pink/brown palette anchors match their
paired classic views. Captures stay ignored under `target/pokemon-native-review`.

This accepts Furret's current battle-model milestone. It does not complete the
all-Pokémon goal: broad rig coverage, world motion and browser fidelity still
need work. At this milestone the standard registry covers 43 species through
68 variants; the separate legacy Pidgeotto rig is not included in that count.
