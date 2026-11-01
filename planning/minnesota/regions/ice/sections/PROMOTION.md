# Guarded final winter source promotion

`promote_sections.py` defaults to dry run. It does not write a crystalpack or export content. Frozen source-v03/source-v04 files and every native/render history remain unchanged. Root owns canonical application after scoped acceptance.

The exact final pack is `content-packs/minnesota-northern-winter-v02.crystalpack`, SHA `92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33`, identity `043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f`. The tool checks the independent 701-positive/10-corruption written proof and actual final five-case controller receipt `.local/winter-sections-controller-v04-receipt.json`, its log hash, pack binding and exit status.

Root's acceptance receipt must have these fields, plus real hash-bound evidence rows:

```json
{
  "status": "ROOT_ACCEPTED_NORTHERN_WINTER_V02",
  "pack_sha256": "92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33",
  "winter_sections_promotion": {
    "source_contract_sha256": "716a7394c78ab02908a68ce9fe8e3425694f03e6f13630f8458d0ad8ed24a865",
    "native_preservation_pass": true,
    "native_pixel_comparison_pass": true,
    "controller_pass": true,
    "root_visual_acceptance": true
  },
  "evidence": []
}
```

The empty evidence array above is documentation, deliberately invalid for execution. Each real row requires `path` relative to this repository and `sha256`; the tool reads and hashes every referenced file. Root must create the receipt from actual observed acceptance, not from this example.

From the repository root, run the tool with `--root-acceptance` pointing to that real receipt and `--output` a new ignored review folder. Review the dry run, then repeat with a different fresh output and `--apply`:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/sections/promote_sections.py \
  --root-acceptance planning/minnesota/renders/ice/root-native-acceptance-v02/acceptance.json \
  --output planning/minnesota/renders/ice/sections-native/canonical-promotion-v04/accepted-dryrun-01
```

The example acceptance path must be the actual Root-created receipt. The tool rejects missing or mismatched evidence. Add `--apply` only for Root's authorized application after the successful accepted dry run.

All four complete canonical file preimages must still equal source-v04's `.before` files. The tool rechecks all four, world and interiors before each atomic write. It archives full before/proposed-after files, exact candidate hashes, allowed editorial deltas and independent readback receipts. Any concurrent source drift is rejected; there is no blind rebase or stale full-dossier replacement. A write failure retains a failure receipt and explicit already-applied paths, and never rolls back over another writer.

Canonical layout changes are only `compiled_tileset`, `shore_fitting`, `layers`, and appended `winter_section_review`. Dossiers retain all original fields/bytes and append only `compiled_tileset` plus `winter_section_review`. Accepted status adds only `winter_section_review.status` and `native_evidence` relative to frozen v04; every other review field and full compiled layout grid remain equal. Evidence explicitly limits acceptance to winter terrain/art, preserved gameplay and local fixtures; it does not claim earned campaign access, all-route visual parity or a completed International Falls town.
