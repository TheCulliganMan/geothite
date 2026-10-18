# Raichu curled-ear sculpture and skin

Raichu's generic primitive assembly is replaced by an original cream-bellied
orange mouse with fitted dark eyes/ivory glints, yellow electric cheeks, a
small nose and double smile, brown hands and planted paws, two fitted back
stripes, curled ears and a long thin brown tail with a broad golden bolt.
The external pack's normal front/back sprites guide the silhouette; no shiny
verification belongs to this batch. The 3D colors are not exact palette parity.

| Vertices / triangles | Logical / runtime parts | Joints | Accessors | GLB bytes | Authoring height / pack metres |
| --- | --- | --- | --- | --- | --- |
| 11,400 / 18,848 | 39 / 25 | 13 | 151 | 505,912 | 0.96 / 0.7874 |

The previous static model had 1,610 vertices, 2,856 triangles and 26 parts.
Coherent same-material/carrier packing groups face details, hand digits and
back stripes, retaining every logical source range. Runtime names stay unique
and bounded. All original 20k vertex, 40k triangle, 64 runtime part/joint,
256 accessor and 1 MiB limits hold. Original authored/pack heights, uniform
scale, full animated bounds and model-admission gates remain intact.

The thirteen controls are floor root, body, head, two arms, two ear bases,
two ear tips, tail base/tip and two legs. Feet stay fixed for battle clips;
the body/leg lower bands blend from the floor root. Fitted face details share
the head carrier, cream belly and back stripes share the body, and complete
brown palms/digits share each arm. Longitudinal ear weights blend head/base/tip.
The thin cord follows its actual sampled curved path from base to tip, and
the complete lightning blade shares the terminal carrier. Idle lasts
3.4 seconds/85 keys; attack/hit last one second/65 keys, with rotation-only
tracks, neutral endpoints and source-cue timing.

Each brown ear comprises a structured oval-ring pinna and rooted lower curl.
Its yellow lining copies an interior strip of the actual front grid, preserving
triangle diagonals, XY coordinates and identical UNORM8 joint weights. The
thin closed layer overlaps its parent. This avoids folds and sinking observed
with the earlier sparsely triangulated concave plates; simple uniform cap
refinement did not resolve them. No parser format or tolerance was widened.
The lining/parent triangle and weight identity is explicitly checked.

World walking exposed an existing classification error: a long asymmetric
tail can move the full silhouette center beyond both legs, merging their
support chains. Pikachu has the same problem: its bound-center X is 0.1304506
while the two haunch centers are -0.1068103/+0.1068103, and that older skin
has no named leg-root controls. `pokemon_gait::support_center` now retains the
existing axis when it separates the supports, otherwise uses authored floor-
root hips, with bilateral named-leg geometry as a fallback. Pads/toe marks
do not choose that fallback. Crawlers/quadrupeds retain their existing axis.
Only classification uses this derived X; source bounds, geometry, placement
and Y/Z centers retain their contracts.

Both mice now have two complete walking chains, two arm controls and a tail
control, with all upper bindings preserved. World vertices stay above the
floor and within full locomotion bounds; contact remains within 0.005 authored
units. Pikachu's GLB is unchanged:
`0e79cd49f616429f000dcf194a29a3e3f6a1908b58107add47c8e1f73617c112`.

Canonical asset, direct CLI and normal background builder match byte for byte.
Both strict Python readers/checker and Rust registry/GPU/world dispatch use
the same GLB. Pichu retains its separate canonical baby export route. Rebuild
only into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/raichu_sculpt.py \
  --output target/raichu-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/raichu-rebuild --only RAICHU
```

Seventeen stored/source Python checks pass in 25.565 seconds: ten source and
seven independently decoded skin checks. They cover closed connected outward
solids, finite normals/winding, original floor/height/budgets, occupied roots
and detached negative controls, visible gaze, real ear notches, fitted surface
vertices/centers/edges, uncrossed neutral/posed ears/limbs/tail, hierarchy and
inverse binds, every independent control, coherent carriers, matching ear
triangles/weights, fixed paws, positive posed winding/bounded strain, neutral
endpoints/source cues and mutated source/bytes/species rejection.

Authoring normalization bounds are cached instead of rescanning every source
vertex for every skin-weight query. A post-cache export comparison preserves
the exact canonical bytes; the export itself takes 0.321 seconds.

Seven focused Rust checks pass in 1.36 seconds: original-size GPU admission,
canonical registry/fixed paws across clips, walking supports/upper bindings/
floor/bounds for both mice, and four classification regressions covering
Raichu's authored hips, Pikachu's geometry fallback, unchanged valid axes
(including Bellsprout, Gloom, Kabutops and Pineco), and missing bilateral
evidence. Shared anatomy/order/physical-size and exact elapsed-idle/current-attack-cue
checks pass in 133.71 and 221.89 seconds, making 26 scoped checks overall.
Both native review clients build in 1 minute 26 seconds. These are scoped
checks, not a full-suite or gameplay-completion claim.

| Source SHA256 | Packed SHA256 | GLB SHA256 |
| --- | --- | --- |
| `648cdc04220d10c5d9f0ffcb22a3f8b6fe77054184ab9034f225b344e6927528` | `0352f76630694336e563c1def6366e9da3d479fd90b43e219c0d57c0fe89c4f0` | `47b70a6be7bfe880b0c9c13b279a312a83dd14e323837430647057334f762f91` |

Six same-state 2D/3D production pairs were inspected inline, entirely
headless and muted: Raichu front/world/battle, Pikachu world, and an opposite
world camera for both mice. Capture times were 181.484/90.608/239.407,
104.486, and 77.242/70.473 seconds respectively. Every receipt reports
preserved authoritative state and stable binaries. Battle admits the complete
Raichu and Rattata models without fallback and labels both species in the HUD.
Original pack sizes remain 0.7874 m and 0.4064 m.

The front and battle images expose Raichu's fitted face, curled lined ears,
planted brown paws and complete thin tail/bolt. Default world cameras remain
substantially foliage-obstructed. Rotating the existing camera from step 2 to
6 exposes more of both mice but still leaves branches across them. This is
not full world visibility acceptance. No grass, foliage, AA, shiny, audio,
pack or gameplay changes belong to this batch.

Both capture clients retain their SHA256 identities throughout all groups:
`render_at_location` is
`004106b83424e7ca53da0e309448a030676ef4ce3c4ce209e47ddc3ede19e586`;
`immersive_battle_3d` is
`4eab4747d9f619680082565e671bd5f936a8aefabfe2597d2be3e815d733416b`.
Receipts and local review images remain ignored under
`target/pokemon-native-review/`. Registry count is 255 skin variants and
20 static species files. Full-roster model and rig improvement remains active.
