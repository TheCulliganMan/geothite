# Kanto interiors: checkpoint and restart

The active goal covers all 120 Kanto indoor, gate, and cave maps. It is not
complete. Authored-family counts and successful mesh audits do not establish
visual completion.

## Runtime inputs

Keep the user-supplied Crystal content pack external and ignored. This checkout
uses `content-packs/realtime-clock.browser.crystalpack` for interior verification.
Transfer that file separately; do not add it to Git or rebuild it from an
incompatible export workspace. Selected furnishing assets must be present under
`assets/open-models`; missing selected assets retain their source 2D drawing.
Native verification sets `CRYSTAL_OPEN_MODEL_ROOT` to the absolute path of that
directory. Embedded authored counter and bin meshes are canonical assets, not
alternate fallbacks.

## Shared geometry

`src/shared_model_geometry.rs` in crystal-voxel-view shares identical positions,
normals, and indices through an Arc cache. Hashes only locate candidates; exact
geometry equality authorizes reuse. UVs, paint, cutaway ranges, and source
material ownership remain separate. Regressions verify shared geometry with
independent palettes, including the café/gym bin.

## Current visual work

The café now has complete shared display cabinets, a connected source-gold wall,
and a compact ribbed bin using the canonical gym bin geometry with native gray
swatches. The connected café service counter reuses the existing counter modules
at its original eight-pixel height and preserves the service opening. Its source
palette distinguishes the green upward cap from gold/brown vertical trim.

The floor remains visually busy. House cabinet/console finishes, retail rooms,
gates, gyms, and caves still need review and improvement across the full scope.

## Shared-file commit boundary

The source worktree was tested as a whole. Multiple tasks have edits in the same
renderer files; this task cannot safely commit those files wholesale on their
behalf. The coordinating commit must retain the verified current changes in:

- `crates/crystal-voxel-view/src/interior_models.rs`
- `crates/crystal-voxel-view/src/dungeon_models.rs`
- `crates/crystal-voxel-view/src/mesh/modeled_interiors.rs`
- `crates/crystal-voxel-view/src/mesh/interior_surfaces.rs`
- `crates/crystal-voxel-view/src/mesh/gate_counters.rs`
- `crates/crystal-voxel-view/src/mesh/gate_counter_bindings.rs`
- Shared renderer integration and the no-alternate-fallback asset deletions
- The appended review receipts in `docs/3d-remaining-families.md`

Do not transfer target directories as required runtime assets. Build logs,
verification PNGs, temporary test executables, and source probes are not shipped
or committed. Use the coordinating task's cold-start script rather than creating
another launcher.

## Original goal and checkpoint

Original goal: **kanto kanto interiors need work**. Preserve the entire actual
KANTO non-TOWN/ROUTE scope: 120 maps (100 INDOOR, 13 GATE, 7 CAVE), including
script block variants. Do not declare completion from this café checkpoint.

Last action: corrected the café counter worktop material selection after native
review found green reaching vertical trim. The selection derives the upward cap
height from named cap faces rather than the slightly higher grain primitive.
Stop further visual edits for the requested host transfer.

## Verified checks and remaining verification

- Source regressions passed in `target/kanto-cafe-counter-cap-source-test-receipt.log`:
  11 connected-network tests, 43 Kanto tests, 23 interior-model tests, 31 placement
  tests, and the existing shared-geometry, missing-asset, water and lighting checks.
  These source tests used freshly compiled Rust code with cached dependency rlibs.
- All 16 counter modules preserve geometry and keep green off vertical trim.
- The actual external pack café regression claims exactly 34 counter cells,
  preserves the service opening, rejects edited source art, and leaves footing alone.
- Production build20381 passed (5m45s); native capture44107 passed and was inspected
  at original2560×1600 resolution. It exposed incorrect green trim, subsequently fixed.
- An initial cap-height test failed because grain stood0.0004 above the cap.
  Corrected source rerun12815 passed. Do not describe the initial capture as proof
  of the final paint.
- Final production build and native review are recorded below when terminal.
- Previous whole-scope production audits covered all120 maps and16 declared script
  variants with zero mesher errors. They establish geometry integrity, not visual
  approval. Old-host ignored logs/captures are evidence, not new-host dependencies.

## Restart on the new host

Use `scripts/cold-start-3d.sh` and `docs/cold-start-3d.md` from the coordinating task
for Git, LFS and content-pack setup. Before resuming, ensure the final coordinated
commit includes all shared integration files listed above; this task's isolated
checkpoint alone is not a complete rendering implementation.

From the Geothite repository root, with the actual ignored pack present:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/realtime-clock.browser.crystalpack" \
  cargo test --locked -p crystal-voxel-view kanto_native
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/realtime-clock.browser.crystalpack" \
  cargo test --locked -p crystal-voxel-view mesh::gate_counters::tests
cargo test --locked -p crystal-voxel-view shared_assets
cargo test --locked -p crystal-voxel-view missing_selected_interior
cargo test --locked -p crystal-voxel-view interior_models::tests
CARGO_TARGET_DIR=target cargo build --locked -p crystal-bevy --profile wasm-dev --all-features \
  --example render_at_location --example audit_authored_assets -j 2
CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=240 \
  target/wasm-dev/examples/render_at_location \
  --pack content-packs/realtime-clock.browser.crystalpack \
  --maps CeladonCafe --view both --output-dir target/kanto-cafe-resume-review
```

These normal Cargo commands are the portable checks. The old-host cache-specific
rustc workaround and `target-bevy20-verify` rlib hashes must not be copied into a
new-host workflow. If source tests skip because the external pack is missing,
restore and verify the real pack before claiming that packed-room checks passed.
Never use a placeholder path or a ROM/disassembly export as a substitute.

Inspect the original native PNG and2D comparison before accepting paint. Next
improvements: café floor texture; Red's house cupboard/console palettes; Kanto
retail floors/counters; gate finishes; gym furnishings/walls and missing plants;
cave readability. Reuse canonical assets with independent local palettes and
physical sizing. Missing or unknown roles retain2D; do not introduce alternate3D
fallbacks. Keep collision, authored script ownership, and actor datum unchanged.

No audio or controller changes are part of this checkpoint. Keep all game content
packs, renders, compiler output, source probes and audit JSON ignored and external
where required. Temporary test binaries and native hardlink snapshots were removed
by their verification wrappers.

To repeat the complete geometry audit after visual work, with the same external
pack and selected-asset root:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  target/wasm-dev/examples/audit_authored_assets \
  content-packs/realtime-clock.browser.crystalpack \
  target/kanto-resume-audit.json --kanto-interiors --script-block-variants
```

Inspect the report, then remove the temporary JSON. Do not commit or transfer an
audit manifest as game content; it is only a transient verification artifact.

## Transfer checkpoint verification status

The final native build was still live at checkpoint (old-host session41346),
with output in `target/kanto-cafe-counter-cap-production-build.log`. No failure
was observed, but success was not yet terminal. The corrected final native
capture was not run. Resume with the portable build and capture commands above
and inspect the image. Source material regressions passed; visual verification
of the final paint remains pending.

The checkpoint parent was `aea28005f0f014311642311e3150d18c11c244e9`. Find the checkpoint commit containing this
note with `git log -1 --format=%H -- docs/handoff/kanto-interiors.md`. This
checkpoint includes the shared-geometry module and its isolated lib registration,
the owned connected-counter files, and this note. Remaining mixed-owner renderer
integrations and review receipts are deliberately left for the coordinating
commit. The isolated checkpoint alone is not a standalone cold-start guarantee.
