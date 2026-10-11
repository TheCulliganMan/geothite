# Cleffa and Igglybuff

These are original renderer-native sculptures, authored in
`tools/baby_fairy_sculpt.py`. Their canonical mesh JSON files are generated
through the existing battle-species builder. Clefable and Wigglytuff keep their
existing recipes and assets.

Cleffa has a continuous peach pear body, broad brown star ears, an angled rolled
forehead tuft with a fitted curl, small dark eyes with ivory reflections, warm
cheeks, a wavy smile, inward hands, two feet, and a short rear puff. Igglybuff has
a pale balloon body, three joined crown lobes, a fitted forehead spiral, ruby
irises and dark pupils, a delighted mouth, small side hands, and petal feet.
Facial layers follow the actual parent triangles rather than an analytic proxy.
The established model heights remain 0.60 and 0.58 respectively.

Shape and color references: [Cleffa official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/173.png)
and [Igglybuff official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/174.png).
No source artwork is shipped with the assets.

The downloadable Sketchfab listings checked included Cleffa
`8071756ead9548eb8d734cd181b947df` and `881ec68b435e4d9abd60648b755495ea`,
and Igglybuff `eb434d6210e94307a155fb664c7ce6d8`. Their metadata did not establish
a usable original rig. Two other Igglybuff listings,
`217a11c999cc4e858278bcb2cedc0806` and `87ad19da768c42cba9e0784558c090dd`,
explicitly describe AI generation. No external model was imported. These local
assets are static geometry; they do not claim skeletal animation.

The [Cleffa metadata](https://api.sketchfab.com/v3/models/8071756ead9548eb8d734cd181b947df)
and [Igglybuff metadata](https://api.sketchfab.com/v3/models/eb434d6210e94307a155fb664c7ce6d8)
were rechecked: both advertise CC Attribution and zero animations. Zero
animations alone does not prove an absence of a skeleton. Both official download
endpoints returned HTTP 401 without authentication, so their geometry and rigs
could not be inspected through that route.

`test_baby_fairy_sculpt.py` checks closed outward geometry, normals, exact recipe
parity, heights and grounded feet, distinct anatomy, stored-triangle face fit,
front-face clearance, physical appendage joins, and supported spirals. Front,
rear, oblique and native battle views must also be reviewed; geometry checks
alone do not establish visual quality or gameplay correctness.

Verified locally: all nine sculpture regressions passed; canonical layout and
all 571 JSON documents passed storage validation; the Rust catalog regression
passed for the 220-species battle catalog. The remaining 31 shared species are
covered by the separate shared-species regression. Final front, rear and oblique renders were inspected,
followed by successful native battle captures of both species with the ignored
`content-packs/realtime-clock.browser.crystalpack`. Their small species scale and
visible features were retained. Native captures establish presentation only;
they do not establish controller or MCP gameplay. Cleffa has 17 parts and 24,376
triangles; Igglybuff has 19 parts and 28,684 triangles.
