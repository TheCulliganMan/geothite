# Temporary human GLB migration

This bounded bootstrap converts the existing 75 human rigs into the exact
reviewed standard GLB catalog, applies the frozen source changes, and retires
only the explicitly listed previous assets. It stores no geometry wrappers.
Only draft PR 5 in TheCulliganMan/geothite on feat/connected-johto-3d can run it.
The separate workflow is a single-parent child of this bootstrap and pins its
exact commit. Preparation proves all human float32 geometry/material/bind
identities and the complete target Git tree. Publication follows successful
checks, rechecks the branch, removes all bootstrap/workflow files, verifies the
exact target tree again, and uses one normal fast-forward push. It cannot merge
or deploy and does not change credentials, permissions, or repository policy.
