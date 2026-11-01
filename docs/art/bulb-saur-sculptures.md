# Evolved seed saur sculptures

`tools/bulb_saur_sculpt.py` builds original Ivysaur and Venusaur sculptures.
Production now uses their [standard skins](bulb-saur-skins.md) through the
shared loader. The former generic JSON copies are retired. Native same-state
visual acceptance remains pending.

The existing bodies share a generic rounded quadruped recipe. The candidates
have distinct growing and mature torso/skull profiles, four rooted supporting
limbs, complete feet and twelve curved ivory claws. Eyes, red irises, pupils,
nostrils, mouths, tongues and angular markings conform to stored parent
triangles. Venusaur has a broader body and cheeks, lower eye apertures, fleshy
slanted brows and fitted muted pink ear interiors. Ivysaur retains taller
alert ears and a closed folded rose bud. Its initially excessive bud height
was shortened after the first offscreen sheet.

Six closed serrated leaves have folded central keels and rooted round midribs.
Venusaur supports a woody stalk, five broad rounded petals, fifteen fitted
cream markings and a raised golden flower heart with rounded stamen tips.
The previous seven triangular petals are not reused. Both recipes normalize
uniformly to the existing authoring height of 1.20; pack dimensions will
continue to own physical scale during integration.

Initial review found inward shading at the concave bud cap. Explicit radial
cap fans replace the polygon fan. Rounded claws and veins retain their
geometry while sharing smooth radial normals. Reducing the mouth refinement
too far made the tongue fail its actual-parent clearance check; the finer
mouth surface was restored. These are candidate iterations, not acceptance
results. The final offscreen sheet was inspected after joining Ivysaur’s torso
to its head. All nine geometry checks pass, including actual closed-volume
attachments, complete anatomy, fitted feature clearance, distinct mature
proportions, fixed height/floor, recipe parity and the existing budgets.

Export and test without opening windows:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/bulb_saur_sculpt.py \
  --output target/pokemon-native-review/bulb-saur-candidates
BULB_SAUR_TEST_MODELS="$PWD/target/pokemon-native-review/bulb-saur-candidates" \
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_bulb_saur_sculpt.py -v
```

Without the environment override, the tests read the installed canonical
GLBs; they generate candidates in memory when those assets are absent.
Explicit review directories can contain exported neutral JSON or canonical GLBs.
Shiny checks remain skipped. Grass, scenery, camera and audio are outside this
candidate work.

Ivysaur has 58 logical parts, 18,609 vertices and 29,796 triangles. Venusaur
has 100 logical parts, 19,789 vertices and 31,500 triangles. Their canonical skins pack them into 26 / 29 runtime primitives while
retaining every logical source range.
Actual-parent tests caught Ivysaur’s torso sitting too far behind its head;
its forward placement now joins the skull and all four supporting limbs.

The source geometry SHA256 values are:

- ivysaur: `4195e2b91009093f48f1ca35847d42e9058b0147cc4b13209b9da41fca21bd7f`
- venusaur: `ba3e09a51c8df2b1d3e4a30ea99c955679b8b1d7fdb630f349bd7645b471230b`

The pack-sprite comparison prompted a mature-body refinement: a 15% wider
torso, head and supporting limbs, corrected normals and hip pivots, and rooted
rounded skin bumps. The original full-canopy bounds are checked bit-for-bit.
Ivysaur's canonical asset remains byte-identical. Refined Venusaur native review
is pending.
