# Independent bounded winding/shelter merge proof

The completed written native composition passes **194 checks and eight corruption negatives**. Exactly13 map modules and300 terrain metatiles change relative to the541-map Leech Shore story parent. All528 other whole modules remain exact, including all70 derived city/room-family records and all eight current gyms. This worker performed read-only pack/source comparison; no Cargo, compiler, exporter, publication, controller, GPU or save-migration execution is claimed by this proof.

[Final machine proof](winding-shelter-541-03c9-7e5f-final-proof.json), SHA256 `0ea892087a25b1226bd564aaab80ac0e453c750589743eb95bb74e6ddbb17544`.

| Exact completed input | File SHA256 |
|---|---|
|`.local/minnesota-city-revisions/leech-shore-03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97.crystalpack`|`03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97`|
|`content-packs/minnesota-northstar-map-quality.crystalpack` (reviewed inherited geometry)|`9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2`|
|`content-packs/minnesota-northstar-seep-shelter-review.crystalpack`|`7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a`|

All three file hashes remain stable after the audit. Each native framing/FNV hash is verified by the existing read-only loader. The final map roster remains exactly541; no map, room or runtime recovery destination is added or removed.

## Exact owned changes

| Native module | Changed terrain metatiles |
|---|---:|
|`MnR03Cave1F`|18|
|`MnR05RockCut`|45|
|`MnR06C00R00`|28|
|`MnR08C00R01`|33|
|`MnR10C00R00`|2|
|`MnR10C01R00`|0; alias attribute only|
|`MnR10Cave1F`|16|
|`MnR12Cave1F`|81|
|`MnR15C00R00`|27|
|`MnR16C00R00`|11|
|`MnR16C01R00`|7|
|`MnR17Cave1F`|21|
|`MnR19C00R00`|11|

Every owned whole module becomes byte-value exact to its story-parent record after normalizing only blocks and the two R10 tileset attributes. All compiled typed commands, objects/NPC anchors, events, warps, flags, trainer records, encounters, fields and all other map attributes remain exact. Block labels remain existing and unique. All complete frozen R08/R15/R19 source grids, including untouched clipped chunks, match the actual final native geometry. R12 matches frozen v03 source `f346edbf365966dd81a90fa7f044bd4b92dfa35f54d1f8c2e90b9b6169ac3d32` exactly; metadata-only v04 changes no terrain. Inherited R03/R05/R06/R10/R16/R17 geometry matches reviewed9aa9.

All six R08 story caps remain native05: north-left `(0,17)/(0,18)` and south-left `(11,23)/(12,23)/(14,23)/(15,23)`, in local metatiles. The explicit R03/R06/R07 closures, Bemidji caps, Brainerd cap/mouth and all five installed R10 city-seam cells remain exact. Every one of the541 native connection vectors and flags is preserved, including the newer Leech shore and all preceding chapter links. The old actor display-name mismatch in active R08 authoring is not silently applied to native trainers.

Beyond blocks, only the two R10 exterior tileset attributes select `mn_r10_rockstairs`; its complete definition and five alias runtime files exactly match9aa9. Original runtime files, shared Johto/cave/gym art, all compiled audio/audio manifests/compression and report fields remain exact. Exactly two alias manifest IDs are appended; runtime ID gains exactly those two suffixes. Original REPEL changes200→100; all other item fields/items remain exact. Normalizing only these declared changes reconstructs the **entire story-parent GameDataSet**.

## Independent content identity and mirrors

The identity proof does more than check that a hash string changed. It independently extracts the five exact in-memory CBOR value spans from each completed pack, then hashes the native tuple `(format_version, data, audio_manifest, runtime_files, report)` with its CBOR array header. This matches `derive_compiled_game_pack_identity_from_manifest` and `cbor_identity_hash.rs` without letting a second encoder choose different float widths or struct order. No raw spans/catalogs are written. All three recomputed identities match their stored native identities. Final content identity is `f26fb7584c6a66d0964b78f570fb6273bb2fef82c1d5895a246ae08dec10ca37`.

All541 global attribute/NPC/encoded-block mirrors agree with their native modules. Every map's raw MapScripts/MapEvents section agrees with its typed rows after checking the native derived sequential command indices. All13 owned maps' script bodies also agree with the global raw bodies.

There are **nine pre-existing text-body disagreements in three unchanged modules**: seven `MnBoundarySurveyChief*Text`/`MnBoundarySurveyReceiver*Text` labels, `MnGrandMaraisTamarackThanksText` and `MnGrandMaraisFisherHouseFisherText`. Their cached bodies and global bodies each remain exactly unchanged from03c9, proven by fingerprints and complete module/catalog equality. Their runtime effect is not investigated here. The first audit correctly refused a blanket claim of all541 script-body coherence; its failed proof, source and log remain under `renders/map-quality-541/independent-proof-first/`, and the original failed report remains retained. Final proof expressly preserves this baseline disagreement rather than repairing it or claiming it absent.

## Negatives, reproduction and limits

Eight mutations independently fail the final positive proof: moving R12's original actor while synchronizing the NPC mirror, changing SUPER_REPEL, reopening the sixth R08 cap while synchronizing its block payload, changing R12 terrain, changing the unrelated Pine gym, deleting a global raw script, deleting an alias file and adding unsolicited art. Each negative records the actual failed checks; the positive baseline passes first. No pack is written by a mutation.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/audit_winding_shelter_541.py \
  --output planning/minnesota/reviews/winding-shelter-541-03c9-7e5f-final-proof.json
```

The tool refuses to overwrite an existing output; choose a new output path for a repeat. Input paths/full file hashes are deliberately frozen, and source authority hashes are retained. Root's12 successful strict Rust publication stages remain in `renders/map-quality-541/publication-receipts.json`; R12 publication is separately logged by Root. Those executions are not substituted for this comparison, and this comparison is not a controller or GPU run. Root owns final movement/dialogue/PP/Continue, native capture, audio verification and guarded source promotion. [R12 geometry and exact probes](../caves/MnR12Cave1F.depth.review.md) and [bounded R19 candidate](../routes/MN_R19.prairie-approach.review.md) retain their visual limits; the whole20-route quality goal remains unfinished.
