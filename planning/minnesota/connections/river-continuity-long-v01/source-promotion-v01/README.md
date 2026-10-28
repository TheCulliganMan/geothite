# Guarded current-source promotion proposal — no apply

This proposal makes the complete accepted wooded rivers and R05 terrain available as current editable authoring source. It has **not** modified any canonical source, world, interior, native pack or shared Rust file.

The accepted runtime reference is `content-packs/minnesota-river-long-v02.crystalpack`, SHA-256 `3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243`. The frozen 1,678-cell native contract remains unchanged. [promotion.json](promotion.json), SHA-256 `6821496e89ce7a427bf1dc495c61e69bdd6b0b62402d93b614e6f68d441fbd11`, records exact source preimages, byte-for-byte before copies, candidate hashes, cell changes, native chunk rectangles and unchanged story/world/interior guards.

## Sources actually in use

The historical `connections/mississippi-v02/manifest.json` references the two full 64×68 woods layouts in that same directory. They still compile to the previous installed hairpins. Its twelve new-map entries pin each full source file SHA and clipped block SHA. The `mississippi-long-v07` directory named in the old runbook is a render/itinerary epoch, not a separate canonical source authority. Its `hand-authored-intent/` sources are also historical and remain unchanged.

The current world route entry for `MN_R05` points to `routes/MN_R05.json`. The companion full layout is `routes/MN_R05.layout.json`, 32×56 metatiles. The route dossier, every actor/event/story record, world and interior declarations are guarded and unchanged.

Replacing the two woods files in place would invalidate the frozen historical manifest and its documented rebuild. Therefore the proposal creates new full current woods sources under `connections/mississippi-long-v01/` and leaves `mississippi-v02` intact. The proposed `authoring.json` gives one explicit current-source inventory; it is labeled **not an executable build manifest**. Existing historical rebuild/review commands continue to use their frozen historical inputs. Future current rendering/publication must select the versioned current files deliberately.

## Proposed destinations and exact guards

Paths in this table are relative to `planning/minnesota/`.

| Proposed destination | Current preimage | Candidate SHA-256 | Terrain changes from original main source |
| --- | --- | --- | ---: |
| `connections/mississippi-long-v01/MnMississippiLowerWoods.layout.json` | Must be absent; metadata source `mississippi-v02/MnMississippiLowerWoods.layout.json` SHA `4f4f8c27e620fd02b82859ef28dc59337db5abada51eba9b8897ab310d5434bf` | `e6b90ff27a2cc609b3a4290e216fde3254e77bf4d41b52f79e61ce4dc33b07d4` | 773 |
| `connections/mississippi-long-v01/MnMississippiUpstreamWoods.layout.json` | Must be absent; metadata source `mississippi-v02/MnMississippiUpstreamWoods.layout.json` SHA `29bbdd50eb5449def26a4f0102809cfa81cd1e0468cb13a131bd8122ce5dda59` | `c1e2cd6899571b4e66054a122ea13489bcefe1adc2063ec655bf094e18df5eb7` | 779 |
| `routes/MN_R05.layout.json` | Must equal SHA `a3ada399da97d47e4ab58d4800d8ca5f53b81b690e88938f829f454720b66f19` and the exact saved before bytes | `b81819939802960511b3871f525a8efa8a95b3c181375c02960c5c760d09f086` | 147 |
| `connections/mississippi-long-v01/authoring.json` | Must be absent | `7692b5c9fdd2765923912d8d79db60936cf157e9fb469bec6622c95d8d91aff1` | Inventory only |

The three [candidate layouts](candidate/) contain complete explicit row runs, rather than overlay strokes that leave an older hairpin underneath. `before/` preserves the original source bytes. Woods metadata is exact except for its replaced terrain layers. R05 also changes its shoreline policy from `johto_8_neighbors` to `explicit_source_edges`; all its other metadata, including derived-art declarations and dimensions, is exact.

The R05 count differs from the native overlay count for a concrete reason. Its original main layout already lagged the installed baseline at 27 cells. Six of those older cells are within this revision's closed channel. The final R05 source therefore needs 126 current native river edits **plus 21 surviving previously approved hook, seam and bridge catch-up cells**, totaling 147. `r05_older_approved_source_catchup` lists those 21 exact coordinates and values. The complete source proposal has 1,699 cell differences, compared with 1,678 changes in the native overlay; the extra 21 reproduce already accepted native terrain rather than introduce a new design.

## Shore policy and existing bridge art

The R05 snapshot already contains fitted native shoreline edges. Running the neighbor fitter again would alter those reviewed edges. Its candidate therefore needs the explicit policy. The shared `Layout::parse` profile currently permits `explicit_source_edges` for `MN_R03` only. Before using this R05 source through that parser, Root needs the same **narrow MN_R05 exception**, retaining the legacy route policy and dimensions. This proposal does not change shared Rust. The inspected parser SHA-256 is `3dc1fea065011d211853e4ac2b78d6872790335d6e1b567b9e2799a8728eec25`; recheck it before editing because other route work is active.

The primary R05 snapshot exactly matches the current installed native tileset, including existing bridge blocks `83/84/85` at global metatiles `(10,54)`, `(11,54)`, `(12,54)`. Their dual-plane native art/collision was installed by `authored_river_connections_art::install`; ordinary early `headwaters_art` alone does not provide them. The current accepted pack already has this capability. Do not claim the snapshot can be built against an earlier art-only parent without arranging the existing bridge-art stage.

For that earlier ordering only, [the alternative R05 layout](alternatives/MN_R05.before-native-bridge-art.layout.json) retains original base bridge blocks `44/01/46`. Applying the three **existing** derived bridge substitutions reproduces the accepted snapshot exactly. It is not the primary proposed canonical snapshot, and its presence is not proof of a working fresh historical build. Historical publisher module/file preimages remain frozen; a current rebuild needs an explicit reviewed current overlay/build stage.

## Read-only check

[check.py](check.py) has no apply mode. It only reads canonical sources, proposal files and the accepted native pack, then writes a fresh ignored proof epoch. It rejects changed canonical source bytes, appeared new targets, metadata drift, source/candidate hash drift, changed world/story/interior/historical guards, a wrong cropped-chunk stride or any candidate block mismatch against the accepted native maps.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/connections/river-continuity-long-v01/source-promotion-v01/check.py \
  --output planning/minnesota/renders/river-continuity-long-v01/source-promotion-v01/check-next-epoch
```

The checker compares all 15 native chunks, using the full source's actual stride. Both woods use a 64-metatile source stride, including their last 32×20 chunks. R05 uses stride 32 and a final **32×8** chunk beginning at source row 48; there is no added 24-row padding or shifted lower bridge. It also checks the base-art alternative against precisely the three existing native bridge substitutions. Metadata and both stride/last-row corruption counterexamples are rejected by the same comparison functions used for the real proposal.

A passing check means this proposal's geometry and preimages are reviewable. It is **not** an applied promotion, a certified fresh rebuild, a story/program rewrite, or completion of the wider route-quality goal. After any intended canonical changes, Root must record the actual apply receipt, add the narrow Rust source-policy support if needed, deliberately select the new current authoring inputs, verify regeneration against the accepted native geometry, and retain historical inputs and every render epoch.
