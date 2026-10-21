# Sentret and Furret fidelity revision — 2026-10-21

Complete eight-angle Battle Revolution references for
[Sentret](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339473/)
and [Furret](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339474/)
were compared with the original assets. The verified local pack supplies all
three Sentret front frames, all five Furret front frames and both backs. Actual
Home game pictures in the [Sentret](https://pokemondb.net/sprites/sentret) and
[Furret](https://pokemondb.net/sprites/furret) galleries resolve the full-colour
coat and resting silhouettes. Sentret's extracted bind stands on its feet;
Crystal and Home instead govern the chosen tail-balanced pose. Furret's
extracted lower, forward-leaning pose supplies volume, while Crystal and Home
govern its rearing battle posture. Downloaded geometry remains reference only.

Sentret replaces the thin hooked tail with a broad, centered, rounded terminal
support and three charcoal bands. Its rounder warm-brown body carries a cream
annulus, tiny dark eyes, pink tissue inside narrow charcoal ears, broad raised
forelimbs and short suspended hind paws. Those shanks remain nearly upright.
The tail's complete root cap is buried inside the body in sampled poses from
all three clips. A second draft had a circular front outline but a conical
end in profile; the final dome retains actual near-end cross-section volume.
The fitted details use normals derived from their stored f32 triangles.

An eleven-joint rig replaces the earlier thirteen-joint rig's invented free
curl. Three upper tail controls carry the body above the fixed terminal
support; ears, forelimbs and suspended hind limbs retain distinct controls.
World travel must not bind those suspended feet to the floor. The anatomical
support classifier reads explicit tucked/suspended annotations from complete
labels, including text after the category separator. A production regression
checks the real tail-supported hop, landing, stopping and unchanged bindings.

Furret gains a fuller curved lower body, deeper muzzle, narrower skull/ears over
the full hips, one waist belt, a continuous brown neck saddle, brown ears with
solid cream points visible from both faces, two cheek bars per side, cream hind
paws and a long, thick, banded tail with closed cream
fur points. A shallow mouth well provides real recessed backing beneath the
dark lining and pink lower mouth. Original cream/brown pigments replace the
literal mustard colour inherited from Crystal's shared indexed palette.
Complete forelimb and tail root caps remain inside the body in sampled poses.
The fourteen-joint battle rig retains its separate four-paw world reposing path.

The brown saddle is a closed section of the actual body, sharing exact seams
and skin influences with the cream front. An earlier fitted overlay passed
battle checks but buried itself during the large travel repose. The material
partition replaces that overlay; its shared seams are checked through stopped,
blended and moving poses. The complete lower trunk follows the pelvis while
the chest and neck divide the turn. Travelling forelimb roots move below the
chest before their paws are planted; all eighteen corners of each buried root
cap are checked through travel. Tail clearance uses the complete stored idle
envelope before planting the four paws, including stationary idle.
The canonical battle geometry and clips remain unchanged by world reposing.

| Asset | Bytes | Vertices | Triangles | Parts | Joints | Accessors |
|---|---:|---:|---:|---:|---:|---:|
| Sentret | 582,264 | 14,430 | 13,752 | 40 | 11 | 193 |
| Furret | 650,784 | 17,037 | 10,040 | 38 | 14 | 202 |

Authored heights remain 1.0 and 1.20. The pack's original dimensions remain
0.7874 m and 1.8034 m under the shared uniform production scale. No byte,
vertex, accessor, winding, strain, support or animation-loop limit is widened.

Thirty-eight Python checks cover source parity, actual closed surfaces,
normals, unchanged budgets, shared band seams and skin influences, fixed
supports, fitted surfaces during motion, full root caps, the mouth recess,
rounded terminal volume and strict absolute-time idle closure. The old models
are separately tested against the new reference-shape guards.

```sh
PYTHONPATH=tools python3 -m unittest -v test_sentret_glb \
  test_furret_sculpt test_furret_glb test_sentry_ferret_fidelity
```

The ignored review destination is
`target/pokemon-fidelity-review-20261021-sentret-furret/index.html`.
An executable containing the exact final GLBs passes 103 selected Rust checks:
the species schema, dense animation bounds, strict idle closure, GPU identity
and physical height, the revised supports and attachments, and the shared
roster motion/travel checks. Fourteen of seventeen reference-shape checks reject
the original models; the discarded coat overlay also fails the shared-seam guard.

Ten headless native comparison pairs were inspected in full: quarter, profile,
rear quarter and ordinary field walking, plus a witnessed walking encounter for
each species. All paired-state receipts preserve game state, and both battles
admit the revised models without source fallback at the original pack sizes.
The classic field omits the added follower, and the move menu hides the player's
back sprite; the separately inspected pack backs and studio orbits establish
rear likeness. Exact embedded native/test asset identities, commands and
verification logs accompany the ignored gallery.

The all-target Clippy command was attempted. Errors in multiplayer mask handling
and three unrelated scenery constants block it; none identify the revised
Furret travel module. The workspace formatting check also reports existing
differences across 487 files. The changed travel leaf is formatted, and no broad
formatting or unrelated fixes were applied. Details are retained in the review.
This pass does not establish every animation, shiny variant, browser view or
whole-roster visual acceptance. Changes remain uncommitted.
