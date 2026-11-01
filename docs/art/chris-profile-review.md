# Chris profile review

Larger actual mapped front and side captures show a flat vertical face profile
and a broad angular rear-head/hair shelf. The cap bill silhouette is improved
locally; the whole character remains unfinished. The offline single-view mode
uses the production mesh, materials and animation at ordinary proportions.

A Chris-only deformation adds a receding chin and small nose contour while
keeping eye/crown vertices and mixed-weight neck vertices fixed. Initial
geometry comparison against the previous actual production GPU body shows
identical UVs, indices and skin IDs/weights. Only 539 of 15463 vertices moved,
only in Z, at most .0941764 in source units; none are body/mixed-neck or eye/
crown vertices. This comparison covers the initial .09 nose coefficient,
not the latest revision. Evidence: external chris-profile-diagnostic/geometry-check.json.

Initial build 54172 passed in 1m42s. Its larger side and front captures (76462,
33548) were individually inspected. The chin profile improved, but front
triangle-derived normals caused dark patches. The candidate was rejected.

Normal revision build 91274 passed in 1m25s. Capture 30782 was individually
inspected; a rounded dark nose bump remained. This variant was also rejected
for the toy-like appearance. Successful capture exits do not imply art approval.

The latest source reduces the nose coefficient from .09 to .025 and retains
the authored smooth anime normal field, preserving the receding chin. Build
30703 is active in chris-profile-anime-lighting-build.log. It still needs fresh
front/side, motion and mounted inspection and invariant checks. The rear-head/
hair shelf remains unresolved. No profile variant is accepted as finished,
and no new character revision is deployed. Previews remain muted.

The anime-lighting variant build 30703 passed in 1m20s; reviewer SHA256
30c811eacabe1a19f5498e94f273cb4a69e7b40c655ad9080f1e8ef968275665.
Captures 61159 (large front), 22984 (large side), 11603 (Run .125),
58098 (Run .625) and 91628 (mounted .25) all exited zero. All five actual
images were individually inspected, fourteen views total. The clean drawn
front face returns without a rounded dark nose bump, and the side chin
recedes. This local profile improvement is retained. Full head/hair, outfit
and player approval remain outstanding. No deployment occurred.

The latest production-body comparison preserves UVs, skin IDs/weights,
triangles and normals exactly. Only 537 vertices moved in Z (max .0941764
source units); no body/mixed-neck or eye-plane/crown vertices changed. See
external chris-profile-anime-diagnostic/geometry-check.json.

Source head bounds closely match the actual sculpted body; the remaining
angular rear-hair shelf is not explained by a gross cap placement error.
A Chris-only rear-scalp refinement now selects dark atlas hair, weighted
to the head, behind the face. Two pinned-boundary refinement passes leave
front facial and crown-support regions outside the selection. Build 52634
is active in chris-capped-hair-build.log. Its appearance and invariants are
unverified pending fresh capture.

The two-pass rear-hair refinement built (52634 PASS0,1m11s). Capture 51172
passed and its actual large side image was inspected. The silhouette showed
little useful improvement while production GPU vertices rose from 15463 to
42045; this version is rejected. A direct prefix comparison of vertex arrays
is not valid because downstream UV duplication changes vertex ordering after
head subdivision. No body-preservation claim is based on that comparison.

The replacement uses one refinement pass plus direct rounding of selected
rear hair toward its ellipsoid support. Points shared with unselected
triangles are pinned by geometric position, preserving selection boundaries
and avoiding cracks. Normals blend smoothly toward the rounded surface.
Build 19403 failed on a duplicate Vec2 import in concurrent cave code; the
current cave file already removed those duplicate imports. The retry is in
chris-capped-hair-rounded-build-retry.log. No visual acceptance or deployment
is claimed for this latest candidate.

Rounded rear-hair build 59935 passed in 1m36s; reviewer SHA256
1983c4d8ddc5fabffd4177beef921f3243b364c9a32323176c66c863479f2aff.
Five captures (19562 side,25539 back,2477 Run .125,42842 Run .625,35469 mounted
.25) exited zero. All five actual images were individually inspected in
fourteen views total. The cap-edge ridge is reduced and the connection stays
closed visually in these samples. This local hair-support improvement is
retained; the character remains unfinished and undeployed.

Actual GPU region comparison preserves 21162 body triangles and 1510
front-head triangles exactly, including positions,normals,UVs and skin data.
Vertex count is 20849, compared with 15463 before the hair change and 42045
in the rejected version. Evidence is external
chris-rounded-hair-diagnostic/region-check.json.

Fresh focused regression 98566 passed one test in .05s; physical mounted
contact/culling regression 71996 passed one test in8.74s. These cover the
latest Chris cap/profile/hair, and Kris before her new profile candidate.
The full Cargo command6743 is still finishing other selected crate builds;
its overall result is not yet claimed.

Cargo command6743 finished PASS0. Its actual capped-hair protection test
passed in.05s; the otherselected crate filters matched zero tests.
