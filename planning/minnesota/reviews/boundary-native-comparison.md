# Boundary Waters native comparison

`tools/review_boundary_native.py` accepts exactly the original 48×40 and expanded
64×56 metatile source epochs. It remains read-only and is also imported by
`render_progress_atlas.py`; that caller's `verify(pack, layout, cabin, art)` API
is unchanged. No atlas or pack publication is performed by the reviewer.

The original published 498-map `content-packs/minnesota-cities-review.crystalpack`
was checked before and after this tool change. Pack file SHA256 is
`37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6`;
content hash is `423ea59cd2b6ea3288c0784f8b464c5c6240cc5e0b1a7b787a2605d34aee18d4`.
Both runs passed. Exact reports and all source/native/full/camera images are
retained under `renders/boundary-safari/native-compare/before-dynamic-review/`
and `dynamic-old-epoch/`.

The original comparison proves 1,920 equal exterior blocks and complete native
pixel equality, 20 authored bodies plus two closed-admission guards, 13 signs,
eight reciprocal connections, four traversable Water seams, 348 closed perimeter
cells, the 42-cell sealed entry court and the original paired cabin. Six
negative fixtures are rejected: changed stable map ID, removed sign, moved
guard, opened perimeter, broken connection and incorrect cabin carpet return.

For expanded source `cliff-lakes-v17` (layout SHA256
`6c07a960b1d5b5f7e0f00c5931ebe6407eef552b6af53c788b66afb5de0ea516`), the expected native proof is 3,584 blocks,
six exterior chunks, 18 signs, 14 reciprocal connection records, seven
undirected adjacency seams and 476 closed perimeter cells. IDs 1–4 remain the
original four chunks, ID5 remains the cabin, and new southern chunks are 6/7.
Every native actor/sign position is compared with the supplied exact layout.
The source's bottom gate row determines the two actual blocking guards; the
closed text remains mandatory. BWS_SIGN_01 and BWS_SIGN_06 retain the installed
closed-admission override; all other sign scripts and lines match source.

The cabin mouth is source-derived. In v17, DOOR `(71,55)` and passive Floor
`(71,56)` belong to `MnBoundarySafariC01R01`; both exterior warp records target
cabin passive arrival3. Cabin carpets1/2 and its passive3 must all return to
that exterior's passive2. All other exterior chunks must have no cabin warps.
The 20 interior blocks remain exactly the authored empty traditional-house
room. Base Johto blocks 00–7f retain exact pixels and collision; imports80–83
retain authentic recipes and new84 must equal native cave12 art/collision.

A source-only collision probe using v17 and the installed authentic art found
42 closed-court cells and positive Water pairs on all seven seams (horizontal
rows: 12/30/6; vertical boundaries: 18/60 and 33/34). These are source expectations,
not a claim that the expanded native pack has passed. The executed original
replay is:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_boundary_native.py \
  --pack content-packs/minnesota-cities-review.crystalpack \
  --layout planning/minnesota/revisions/northwoods-expansion/before/boundary-waters-safari.layout.json \
  --self-test \
  --output planning/minnesota/renders/boundary-safari/native-compare/dynamic-old-epoch
```

For the expanded run, select the exact written pack with `--pack`, pass
`--layout planning/minnesota/renders/boundary-safari/expanded-candidate/cliff-lakes-v17/boundary-waters-safari.layout.json`
and select a separate output directory. The expanded native execution is recorded below. Production controller
acceptance remains a separate publisher-owned check. No proposed publication
filename is treated as an existing build.


## Executed expanded comparison

The publisher confirmed strict write/read of the 505-map pack. This worker then
verified immutable `.local/minnesota-city-revisions/northwoods-3949ad8dff81537f9b664f914f5358825da40512fbfdae52c2fffa1a049441b7.crystalpack`,
SHA256 `3949ad8dff81537f9b664f914f5358825da40512fbfdae52c2fffa1a049441b7`.
All 3,584 exterior blocks and whole native/source terrain pixels match frozen v17.
The 22 bodies, 18 signs, stable IDs, 14 reciprocal records/seven Water seams,
476 closed perimeter cells, 42-cell sealed court,133-block art recipes and
relocated paired cabin pass. All six native corruption fixtures are rejected.
Exact evidence and native/source originals are retained in
`renders/northwoods-native/3949ad8dff81537f/native-verify-1793064684803419000/boundary-native/`.

The first stricter text comparison exposed legitimate native wrapping:
`BWS_PALISADE_PORTAGE_SIGN` authors 19 characters on its second line. Native
`grand_marais_modpack::authoring::text` wraps at 18 interior columns, producing
`NORTH / SOUTH` then `LAKES`. The reviewer now derives that same word wrapping
and still requires exact resulting lines. The two installed closed-admission
sign overrides remain explicit. The failed first comparison and its R11
artifacts are retained; no sign source or native program was changed.
