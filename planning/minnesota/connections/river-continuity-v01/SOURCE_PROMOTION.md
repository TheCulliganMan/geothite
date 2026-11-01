# R03 source reconciliation

**Applied by Root with exact preimage guards.** Canonical R03 now matches the frozen `source-v02/` candidate: layout SHA-256 `594e9f4e050ffe31d329547c3ff27025ce8ea24f32d4d6ba40b8cb91531f9941`, dossier SHA-256 `e59bdf5d4767f570869b6a7612e180e531cea0cf6350fa160cb94cac484429d1`. The fresh preflight and applied receipt are retained under `renders/river-continuity-v01/source-promotion-root-v01/`. Both canonical hashes were independently rechecked for this status update. World and interiors stayed exact. Frozen candidate/preimage files remain unchanged.

The source binds the written first-bend pack `content-packs/minnesota-city-river-bend-v01.crystalpack`, SHA-256 `57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62`; that native bend is also preserved in the later house and long-river packs.

The canonical proposal changes55 effective terrain cells and moves the existing north sign from core `(22,4)` to `(22,8)`. All other dossier values and all192 lower canonical terrain cells remain exact. It preserves the existing authoring strokes, then adds103 explicit native-block placements to retain fitted shoreline edges and the new bend. The 55 effective changes comprise15 inherited source/native differences and41 newly published cells, with one overlap. Exact coordinates, source/native preimages and hashes are in `source-v02/promotion.contract.json`.

The Rust authoring parser now narrowly allows R03's `explicit_source_edges` mode. Restoring the automatic shoreline fitter changes the accepted grid and is not equivalent. Do not extend this allowance to every route as an incidental refactor.

The following was the **pre-application** read-only check. It now intentionally rejects the applied canonical files because its guard requires the frozen before bytes; this prevents accidental duplicate application. The successful fresh Root preflight is already retained. For a future, separately frozen revision, use its own before/candidate contract and a fresh receipt:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/connections/river-continuity-v01/check_source_promotion.py \
  --source planning/minnesota/connections/river-continuity-v01/source-v02 \
  --pack content-packs/minnesota-city-river-bend-v01.crystalpack \
  --output planning/minnesota/renders/river-continuity-v01/source-promotion-NEXT/receipt.json
```

The helper has no apply mode. Root completed the guarded canonical writes after the native bridge/controller and visual acceptance recorded in `applied.json`. The guard procedure for future source revisions remains: Immediately before applying the exact candidate bytes, compare both canonical files byte-for-byte with the frozen before copies and verify every candidate hash in `freeze-receipt.json`. Any concurrent drift must be reconciled rather than overwritten. Verify the entire compiled24×32 grid and sign delta afterward. The installed lower map has21×8 metatiles; it must be compared using its actual row stride, not by concatenating native chunk bytes as if every chunk were24 wide.

`prepare_source_promotion.py` produced the immutable proposal and11 checks/seven deliberate corruption rejections. Its `--output` must always name a fresh directory. `check_source_promotion.py` independently recomputes the target from the frozen native candidate, verifies the source hashes and byte preimages, and writes a separate preflight receipt. Neither tool publishes packs, changes game state or exports a game-content catalog.

Canonical Confluence authoring and shared connection manifests are outside this R03-only promotion. Its10 native terrain changes remain bound by `source-v01/candidate.contract.json` and the independent written-pack proof; do not silently rewrite shared manifests while promoting R03.
