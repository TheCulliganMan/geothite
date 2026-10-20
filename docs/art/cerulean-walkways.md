# Cerulean Gym walkways and banks

The source distinguishes three drawings: pink brick tile05, flat gray bank
shadow tile0b, and timber support tile22. The old `CeruleanBridgeFace` name and
inventory description incorrectly called tile0b wood. The verified pack gives
it GRAY palette0 and a flat gray bitmap; tile22 uses BROWN palette5 and contains
a beam edge, post, highlight and underside. The former “foam22” description was
also incorrect in this port tileset.

Sixteen bank cells now use `CeruleanBankFace` / `special:cerulean-gray-bank`.
Each retains the original continuous closed body beside the water and samples
its own gray source. The two invented timber posts per bank cell are removed.
The exact block/row/column guards reject side-rim cells belonging to tile0a;
water backing still requires an authenticated source sample. No walking,
collision or height ownership changes.

The raised deck keeps its suspended body, real posts and shallow brick relief.
Caps retain their live brick print; thin risers use the same source pink. Body
and piling paint comes from an authenticated tile22 support below a genuine
deck. Normal-based material roles select the source beam, lighter post front,
highlight and underside. This uses actual atlas swatches without stretching a
flat support drawing around the volume. Missing donors preserve the previous
body material. Other maps retain their existing pier treatment.

The same material applies beneath the complete entrance guardians using their
existing adjacent deck donor. Entry grades still transform both material
buffers; original deck/beam/post triangles and normals are preserved. The only
geometry removed is the unsupported bank-post interpretation described above.
Source admission, custom ownership, claims, water/rim exclusions and actor
support remain with the existing source resolvers.

All 39 distinct selected checks pass in 43 executions: Cerulean, elevated-pier,
seating and dining-table filters overlap on four new timber tests. New controls
verify packed bitmap/palette identity, donor mutations and required deck above,
beam/post geometry, actual edge material consumption, unchanged brick UVs and
support. Existing piling and ramp checks now query Cerulean's actual textured
buffer; they require deep vertical piling faces and a raised beam, and retain
the original exact ramp-height tolerance. Other routes' assertions are unchanged.
Standard Cargo filters for `crystal-voxel-view` are `cerulean`, `elevated_pier`,
`seating`, and `dining_table`, with `--include-ignored` and the completed external
pack supplied through `CRYSTAL_RENDER_TEST_PACK`.

Five final native/source pairs were individually inspected: middle, entrance,
east, north bank and ordinary UUDD input. Two moving native originals were also
inspected. Every paired receipt preserves the actual held session. Middle and
entrance before/after classic images are RGBA-byte-identical. The east classic
images differ only in 13,968 blue/cream water-animation pixels; that separate
before/after image is not claimed byte-exact. Each current pair still holds its
own matching source state. The walk records 128 frames, six viewport origins
and no additional terrain builds. All captures use headless image targets and
mute, without opening windows.

The full audit passes 388 maps / 288 scripted states, including all 120 Kanto
interiors / 16 states, with zero mesher errors. Other 387 map records and all
288 state records match the preceding cap-stage audit. Cerulean's 16 bank
labels are normalized to the corrected identity; its remaining source/support/
coverage fields are exact. Removing 32 invented bank posts removes 768 triangles
and 2,304 vertices, leaving 26,320 triangles / 73,104 vertices. Remaining changes
transfer existing body material between buffers, without changing its geometry.

Ignored evidence: `target/kanto-resume/cerulean-timber-review/review.json`, its
original images/receipts, and `cerulean-timber-build/full388.json`,
`audit-delta.json`, test logs and saved build arguments. The current render API
was rebuilt for the other goal's `view_center` schema; its source changes were
preserved. Initial stale-API and material-specific assertion failures are
retained as diagnostic evidence. Private opt0 tests / opt1 QA libraries do not
certify a full Cargo build, release performance or the complete gameplay suite.

This accepts the demonstrated bank identity and walkway-material corrections.
Water, wall, guardian, actor, quest and whole-room acceptance remain separate
work within the broader interior/furniture goal. Pack, source model assets,
audio and user saves are unchanged; game content and generated QA products
remain external or ignored.
