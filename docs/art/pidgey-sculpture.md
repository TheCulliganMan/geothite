# Pidgey sculpture and production skin

Pidgey's canonical body is an original small bird with a ten-joint standard
glTF skin. `tools/pidgey_sculpt.py` authors the geometry and
`tools/pidgey_glb.py` supplies the hierarchy, weights and stored clips.
The normal background builder calls the same exporter. The current
[family fidelity review](pidgey-family-fidelity.md) supersedes earlier visual
acceptance, including the subsequently rejected round-headed, raised-wing draft.
The [latest facial follow-up](pidgey-face-fidelity.md) also reopens the earlier
folded model's face after further user feedback.

The current head has a tapered forward skull, a brown forehead framed by
continuous diagonal cream brows, short swept crest feathers and narrow ivory
eyes close to a broad short pink bill. Cream brow feathers sweep backward over
the dark temple wedges. The cream throat joins the full breast. The open bill
has a separate lower jaw, empty aperture and occupied dark lining recessed
behind it. This corrects the previous tiny bill, far-lateral eyes and owl-like
forehead; valid topology alone had not caught that facial layout.
The eye pair now sits around the bill with larger pupils, a shorter brow quill
and a smaller mouth gape. The upper breast carries the throat in profile;
the dark lining stays above the lower bill floor. Both profiles retain the
eyes' wrap around the skull.

Crystal's four front frames contain both raised and folded wing positions;
the folded frame governs this resting pose. Brown shoulder cases cover cream
profile vanes, and three brown tail feathers sweep low behind the body.
Short pink shanks stand above the pads. Each foot has three forward toes,
an opposing rear toe and ivory front talons. Toe projection is independent
of the leg shaft, which does not slope forward.

Reference volume and pigments come from the complete Battle Revolution orbit
and the actual Home render in the [Pidgey sprite gallery](https://pokemondb.net/sprites/pidgey).
The gallery's X/Y and Let's Go thumbnails are pixel art, not captured 3D views.
All four supplied Crystal front frames and its back picture were inspected.
Reference-only downloaded geometry is never an authoring input or runtime asset.

The current GLB has 44 named parts, 8,757 vertices, 12,600 triangles,
251 accessors and 440,812 bytes. Its height remains 0.82 with floor zero;
the verified local pack supplies the production height of 0.3048 metres.
Neutral geometry SHA256:
`18d0a3308234b669a7e7ac0ef9763a22b17b303cfb9711c401c7d4883552ad59`.
Canonical GLB SHA256:
`57fbd44917fd0782b67253fabb3f31aad9f7fa846ce23c1dc52ac050aa07bf0c`.

Pads, toes and talons stay on the fixed root. The face, brows and recessed
lining move with the head; the lower bill has its own hinge. Paired wings,
flight tips, tail and crest retain separate controls. Idle lasts 3.2 seconds;
attack braces early and the tail follows later. Stored clips retain neutral
endpoints and source-relative attack/hit timing. A flat-ended idle wave closes
under the production sampler's strict absolute 10-microsecond check.

```sh
python3 tools/pidgey_glb.py
PYTHONPATH=tools python3 -m unittest test_pidgey_glb test_pidgey_family_fidelity -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view pidgey --lib
```

The nine stored-model checks pass for exact regeneration, loader budgets,
closed actual components, shading, hierarchy/binds, anatomical ownership,
fixed supports, floor/winding/strain through all clips and cue timing.
Family checks additionally cover the corrected facial layout, broad short
bill, real mouth aperture, folded feather plane and upright shanks. The four
additional facial checks reject the actual folded model rejected by the user.
The stored-model checks also cover fitted face cap centers/edges and the full
buried head root through the clips. Counts and digests are
identity checks; they do not establish resemblance. Production capture coverage
and its limits are recorded in the family review.
