# Spearow and Fearow fidelity revision — 2026-10-21

The original assets were compared through complete eight-angle orbits with the
actual Battle Revolution [Spearow](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339353/)
and [Fearow](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339354/)
models, all seven/five supplied Crystal front frames, and both back sprites.
Actual Home renders in the [Spearow](https://pokemondb.net/sprites/spearow) and
[Fearow](https://pokemondb.net/sprites/fearow) galleries establish full-colour
pigments where the small indexed sprites share shadow colours. Export texture
artifacts were checked against those captured game pictures. Downloaded game
geometry was reference-only; the authored recipes contain no imported mesh.

Spearow keeps Crystal's compact standing posture. Its new original recipe is
`tools/spearow_sculpt.py`, exported through the existing `spearow_glb.py` skin.
The cream body is deeper, the warm ochre head has swept cheek/throat points,
and the shorter pink bill surrounds an actual open, backed mouth. Narrow eyes
wrap the revised skull. Closed coral shoulder cases occupy their body joins;
folded flight vanes have broad profile planes. White distal feather sections
meet the actual pink outline and share its wrist control. Short upright
shanks replace the forward-leaning shafts; toe projection remains separate.

Every Crystal Fearow frame is airborne. The new original `fearow_sculpt.py`
uses the raised-wing phase, a low forward-curving neck, downward long open bill,
small fitted side eyes, a narrow sagittal red comb, seven rooted pale nape
feathers and curled pink claws below the belly. Its closed feather fan turns
through sweep as well as pitch. The former duplicate feather bank and broad
back plate are replaced by a narrower solid leading spar. Profiles now have
filled feather area rather than thin vertical strips. Five brown tail vanes
have separated layers. `raptor_bird_glb.py` moves the pivots and bindings with
this anatomy; tucked feet follow the two leg controls under the body.

Fearow now receives the existing airborne battle clearance, including that
offset in camera and GPU culling bounds. Original pack dimensions remain
0.3048 m for Spearow and 1.1938 m for Fearow, using the shared uniform scale.
Pidgeot's original recipe output and canonical bytes remain exactly unchanged.

| Asset | Bytes | Vertices | Triangles | Source / packed parts | Joints |
|---|---:|---:|---:|---:|---:|
| Spearow | 204,836 | 3,703 | 2,848 | 28 / 28 | 10 |
| Fearow | 602,768 | 13,219 | 16,748 | 101 / 21 | 19 |

Authored full heights remain 0.75 and 1.23 with a zero lower datum. The new
geometry retains all existing byte, accessor, vertex, triangle, winding,
20-percent strain and loader limits. Forty-two Python checks cover source
parity, actual closed components, joints/binds, occupied roots through motion,
fitted layers, open-bill volume, fixed Spearow supports, Fearow leg ownership,
independent controls and strict absolute-time loop closure. Nine of the ten new shape
checks reject the immutable earlier assets. The wing-plane check
compares actual projected triangle area, rather than a depth bounds box.
The filled-bill negative control checks occupied intervals, so a solid enclosing
the whole gap cannot evade detection merely because its boundary lies outside.

The retained review is
`target/pokemon-fidelity-review-20261021-spearow-fearow/index.html`.
It includes shared-camera 4-by-2 reference/before/after sheets, every complete
orbit, Crystal front/back frames, Home reference pictures and decoded idle,
attack and hit poses. Fresh production builds pass 85 selected Rust checks, including all 73 shared
travel checks, stored source/schema parity, dense animation bounds, strict
absolute-time loop closure, GPU activation at pack size and airborne battle
bounds. The normal background builder reproduces Spearow, Fearow and unchanged
Pidgeot byte-for-byte. Exact compressed canonical assets were located in the
test executable and both native examples before review; executable and asset
hashes remained fixed through all captures.

Twenty native/classic PNGs form ten matched pairs: quarter, profile, rear and
ordinary walking in Ecruteak, followed by an ordinary Route 36 walking encounter
for each bird. All ten receipts report preserved state. Both final battle
scenes are admitted (`accepted=true`, `reason=None`), use modeled participants
with no source fallback, retain zero placement datums and preserve the verified
pack dimensions. Both complete bodies fit the battle frame. Spearow remains
small in the field at its real 30.48-cm size; Fearow's flight and tucked claws
remain readable from the rear and during travel.

The classic field omits the added follower, and its move menu covers the
player back sprite. These frames establish matching state, not that missing
sprite's likeness; the extracted back pictures and complete studio rears supply
that comparison. No dedicated browser or shiny acceptance is claimed. All 274
unaffected canonical variants remain byte-identical. Reference-only downloads
and private scratch were removed after retaining the review evidence.
All changes remain uncommitted; the wider roster review remains active.
