# Central lake route variation

R06 now bends around a wooded promontory on its eastern approach, giving the
player two actual turns between the lookout junction and fishing pier. Its
southern bridge landing has narrower pine shoulders. R07 retains its broad
lake crossings and wet peninsula, with a recessed west cove and sheltered pine
shoulders on the lower shore. These identities complement R08's ancient mire.
No dimensions, connections, arrivals, actors, rooms, gates or active pack pointer
changed. The compatible 21-cell R06 and 19-cell R07 grass revisions are retained.

`MN_R06.layout.before.json` and `MN_R07.layout.before.json` reconstruct the exact
installed terrain in `minnesota-r10-rock-gallery-v01.crystalpack`, including its
actual narrow town seams. Candidates retain those seam cells rather than
silently widening them to an older source layout. Candidate dossiers reconcile
the installed Minnesota trainer teams and encounters; the publication changes
terrain only, preserving all native events and encounters byte for byte.

## Executed publication

The completed Rust executable `target/runtime-fps/wasm-dev/revise_authored_route`
was reused read-only. These commands succeeded:

```sh
target/runtime-fps/wasm-dev/revise_authored_route \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  planning/minnesota/revisions/central-lake-variation-v01/source-v01/MN_R06.layout.before.json \
  planning/minnesota/routes/MN_R06.layout.json \
  planning/minnesota/routes/MN_R06.json \
  content-packs/minnesota-central-pines-v01.crystalpack
target/runtime-fps/wasm-dev/revise_authored_route \
  content-packs/minnesota-central-pines-v01.crystalpack \
  planning/minnesota/revisions/central-lake-variation-v01/source-v01/MN_R07.layout.before.json \
  planning/minnesota/routes/MN_R07.layout.json \
  planning/minnesota/routes/MN_R07.json \
  content-packs/minnesota-central-lakes-shape-v01.crystalpack
```

The resulting ignored pack SHA-256 is
`bfbf00d1c6757821704e492e99b6aabb1a536805f54c8f2cdca97135e52ce7b9`.

The source equivalent is `CARGO_TARGET_DIR=target/central-lakes-shape cargo run
--locked --profile wasm-dev -p crystal-assets --bin revise_authored_route --`
followed by the same five arguments. Disjoint reviewed changes may be present
in the input pack: the publisher verifies these route preimages before writing.

## Evidence

- The Rust publisher validates complete route geometry and installed anchors.
- The native audio verifier passes all 377 programs' PCM hashes, frame counts
  and loop ranges, and desktop runtime validation passes.
- Both route-specific source reviews pass, including required habitat, native
  ledges and return paths, necessary bridges, effective Cut gates and closed
  reachable Surf perimeters.
- `review_authored_terrain_revision.py` passes exact whole-pack preservation
  for both revisions, including nine adversarial mutation checks. Only the four
  exterior terrain arrays and their encoded block payloads change: 17 and 21
  cells for R06; 8 and 20 for R07. Source and native art match exactly.
- The reused `crystal_bevy-01787b9da148a652` binary passes R06 northern-bend
  traversal plus both routes' Cut denial/removal/collection/Continue and Surf
  perimeter/reciprocal-seam tests (five tests total).
- The old populated-journey assertions expected Johto species/text. They fail
  against the already-native Minnesota parent and were narrowly corrected
  in `pine_country.rs` and `leech_lake.rs`. The refreshed production tests pass
  all three Pine Country cases (including the new two-bend detour and complete
  populated journey) and the Leech Lake populated eight-trainer/three-wild
  journey on the combined region-polish parent. Selected PP, resources,
  meaningful grass traversal and Save/Continue remain asserted.
- Actual headless 2D captures were inspected at R06 `(54,37)` and R07 lower
  chunk `(49,15)`, noon. The pine turn is readable at player scale; the peninsula
  camera shows shoreline, grass and its turn together.
- Changed test files pass direct rustfmt checks. Workspace `cargo fmt --all --
  --check` reports unrelated existing formatting in `offset-allocator`.

Immutable renders and preservation proof live under ignored
`planning/minnesota/renders/central-lake-variation-v01/`. Full source views are
also archived by the existing Pine Country and Leech Lake review tools. Packs
and generated verification output remain ignored.
