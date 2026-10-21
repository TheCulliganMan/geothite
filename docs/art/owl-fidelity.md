# Hoothoot and Noctowl fidelity revision — 2026-10-21

Complete Battle Revolution game orbits for
[Hoothoot](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339475/)
and [Noctowl](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339476/)
were compared with the original assets at all eight angles. All six front
animation frames and both back pictures were read through the verified local
pack. Actual Home game renders in the [Hoothoot](https://pokemondb.net/sprites/hoothoot)
and [Noctowl](https://pokemondb.net/sprites/noctowl) galleries resolve full-colour
pigments and standing wing carriage. The extracted bind poses expose feet and
flight feathers differently from Crystal's resting stance. Those pose/export
differences are recorded rather than adopted as the new battle posture.

The old Hoothoot has invented chest chevrons, blue needle brows, white irises,
long folded wings, a curling tail and a forward-sloping shank. The original
replacement recipe uses a deep round body, broad pale belly and short hem,
coral irises inside charcoal clock rings, three lower lashes per eye, two broad
diamond clock-hand ends, small outward wings, a pink hooked beak, a low rooted
tail and one upright pink support with dark ankle bands and ivory claws. The
second foot is wholly tucked inside the body in sampled poses from all three clips.
Lashes occupy the outer ring instead of overpainting the coral iris.

Noctowl gains a deeper breast and skull, two broad gold three-point plumes,
a connected solid gold brow bridge, black/ivory eye rings with red irises,
pink bill and claws, gold feet, six dark breast triangles and deep brown folded
wing cases. The complete shoulder caps are buried inside the body. The old
candidate exposed twelve of eighteen cap corners; the corrected stored mesh
keeps every cap corner occupied in sampled poses from all three clips. The gold
bridges physically join both the head and plumes in the sampled motion.

Both pure recipes remain in `tools/*_sculpt.py`, exported by the existing
standard skin authors. Pivots and weight fields follow the revised anatomy.
Body reference heights remain 1.0 and 1.30; production dimensions remain the
verified pack values 0.7112 m and 1.6002 m under the shared uniform scale.

| Asset | Bytes | Vertices | Triangles | Parts | Joints | Accessors |
|---|---:|---:|---:|---:|---:|---:|
| Hoothoot | 354,752 | 7,029 | 8,684 | 38 | 12 | 210 |
| Noctowl | 372,248 | 7,706 | 10,076 | 52 | 14 | 239 |

Travel preserves Hoothoot's tucked leg/foot bindings and constructs one complete
support for an original one-foot hop. The anatomical classifier reads the full
label for the tucked annotation, including text after the category separator.
All toes inherit that complete leg even when their centers cross the body's
left/right axis; coordinate signs do not create another support.
Noctowl retains its separate standing battle posture and flying world travel.
No copied game mesh, rig, texture or content pack enters the authoring recipe
or shipped assets. Changes remain uncommitted.

Thirty-two Python checks pass and cover canonical bytes, actual f32 closed surfaces,
normals/winding, finite dimensions, fixed battle supports, joint/bind ownership,
unchanged strain and loader limits, fitted layer clearance in motion, complete
buried wing caps, occupied brow/crown joins and strict absolute-time idle loop
closure. The tiny mouth seam's test examines every front triangle; it does not
force extra subdivisions to satisfy the old arbitrary face-count assumption.
Ninety-seven Rust checks also pass, including the real single-support hop,
landing/stopping, preserved tucked bindings, shared roster locomotion, dense
animation bounds, GPU skinning, schema and strict production idle closure.

The Python checks can be reproduced from this checkout with:

```sh
PYTHONPATH=tools python3 -m unittest -v test_hoothoot_glb \
  test_noctowl_sculpt test_noctowl_glb test_owl_fidelity
```

The ignored retained review is
`target/pokemon-fidelity-review-20261021-owls/index.html`. It contains complete
shared-camera reference/before/after turntables, all Crystal frames, Home
pictures and decoded idle, attack and hit poses. Production verification is
complete for ten headless native pairs: quarter, profile, rear quarter, ordinary
field travel and a witnessed walking encounter for each species. All twenty
images were inspected. Every receipt preserves state; both battle scenes admit
the intended model without source fallback, with unchanged physical heights and
zero support offset. The exact canonical compressed assets were located inside
both freshly built executables and the test executable; their identities remain
unchanged after capture. Field 2D omits the added follower, and the battle move
menu hides the player back picture, so separate pack backs and studio views
establish rear likeness.

The review retains executed commands, byte-parity evidence, receipts and check
logs. Both packages used optimization level 1 for this offscreen review;
geometry budgets, physical scale and numerical limits are unchanged. All 274
other canonical variants remain byte-identical. This does not establish shiny,
browser or every-pose acceptance. The wider roster review remains active.
