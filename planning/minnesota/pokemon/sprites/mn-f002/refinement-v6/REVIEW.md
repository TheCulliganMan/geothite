# Native verification

Selected manifest: `manifest-selected.json`.

Selected native output: `target/minnesota-pokemon/proposals/MN_F002-v6-solid-white4-ink12/`.

All six selected frames were inspected at native size and in the 8× review. Each front is 56×56 with 784 native 2bpp bytes; each back is 48×48 with 576 bytes. Every PNG uses exactly its species' declared four RGB555 colors. Native 2bpp was decoded independently, respecting front row-order and back column-order tiles, and matched both indexed JSON and PNG pixels exactly.

Selected conversion: Solid exterior contours, white-detail threshold 4, interior-black threshold 12. SourceBacked leaves some diagonal seams weaker; black threshold 8 makes the charcoal body and steel joints overly dense. White threshold 16 gave no useful gain on the flattened source.

The 1536×1024 source uses different front/back column splits to avoid neighboring fragments. Front columns split at x=385/885; backs at x=432/942. The row split at y=535 lies in a clear gutter. All measured source bounds stay inside their own cells. The back crops contain no feet and are intentionally clipped at the lower shoulder/body boundary.

The more highlighted revision-4 alternative is preserved at `target/minnesota-pokemon/proposals/MN_F002-v4-solid-white64-ink12/`. Its highlights were noisier at native size, so revision 6 is preferred. All other successful source/solid and threshold conversions remain in their proposal directories.

`tools/test_fire_starter_fidelity.py` was inspected. It tests Cyndaquil, Quilava and Typhlosion 3D GLB geometry, not these regional pixel sprites; it was not used as evidence for this art refinement. Verification here compares the accepted original creature landmarks and native encoded frames directly.

Subsequently integrated at root's request on 2026-10-26. The canonical family manifest and `target/minnesota-pokemon/native/MN_F002/` now use these six selected views. The full-family review hash matches the manifest; the current production Rust graphics loader accepted all 21 runtime files. The previous canonical manifest and native versions remain preserved. No profile or palette values changed.
