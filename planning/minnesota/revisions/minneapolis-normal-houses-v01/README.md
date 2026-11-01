# Minneapolis normal house fronts

Replaces the user-rejected mixed cyan-wall and protruding-door fronts with complete original house assemblies. Eight homes use the exact already imported Pallet Town house, and the Mart uses its complete native Cherrygrove storefront grammar. No new bitmap pixels, palette edits or enlarged footprints.

The rejected two-block wall `10 9d` does not belong below the roof as a complete house. Do not restore it from the old join or waterfront contracts. The former Angler Home also mixed the Pallet roof with this cyan wall.

## Exact source

`before.layout.json` is the full native city preimage. `after.layout.json` changes only32 metatiles. `recomposition.contract.json` lists every native anchor, relocating only nine door/return pairs two walking tiles left to their actual visible entrances. All40 destinations, all other city geometry, actors, signs, cycling scripts, north pool gates, river seams and recovery records remain in place.

Exported native module snapshots and all rendered originals live only in ignored `renders/minneapolis-house-normal-v264/`; they are verification material, not duplicate authored command catalogs.

| Destination | Top-left metatile | Door before | Door after | Return after |
|---|---|---|---|---|
| River Family Home | [6, 3] | [15, 9] | [13, 9] | [13, 10] |
| River Naturalist | [26, 7] | [55, 17] | [53, 17] | [53, 18] |
| Pattern Studio | [32, 7] | [67, 17] | [65, 17] | [65, 18] |
| Bridge Home | [12, 9] | [27, 21] | [25, 21] | [25, 22] |
| Mart | [35, 11] | [73, 25] | [71, 25] | [71, 26] |
| Angler Home | [8, 13] | [19, 29] | [17, 29] | [17, 30] |
| Tool Flat | [24, 24] | [51, 51] | [49, 51] | [49, 52] |
| Gardeners Home | [2, 27] | [7, 57] | [5, 57] | [5, 58] |
| Arrival Info | [21, 28] | [45, 59] | [43, 59] | [43, 60] |

## Native publication

Rust `recompose_authored_city` publishes the explicit terrain and complete indexed anchor contract against the selected parent recorded in `plan.json`. Same-size Minneapolis publication certifies fixed gate footprints, all18 cycling return dispatches and unchanged guarded-water geometry. Unchanged neighboring seams must not republish unrelated raw payloads.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/debug/recompose_authored_city \
  content-packs/minnesota-current-bemidji-plateau-stairs-v01.crystalpack \
  planning/minnesota/revisions/minneapolis-normal-houses-v01/after.layout.json \
  planning/minnesota/revisions/minneapolis-normal-houses-v01/recomposition.contract.json \
  content-packs/minnesota-minneapolis-normal-houses-v02.crystalpack
```

Use a fresh ignored output and resolve `world.json` before rerunning. The command above succeeded; see `renders/minneapolis-house-normal-v264/publication-v280.log`. Final pack SHA256: `9480368160e25c53d01162a4dbdbe301a7b7215e0a5027df58d700906612be99`. `world.json` records the selected local revision after the checks below.

## Review scope

Whole-town originals, every rejected pass and all native GPU frames are retained. Nine complete facade footprints match native pixels. Three wider frames include moving bicycle poses outside the repaired buildings; the southern Arrival Info view needs the connected-neighbor comparator rather than a repeated-border assumption. These differences are retained and do not get concealed by editing screenshots.

Explicit initial Minnesota party and porch positions are local QA setup. Production MCP checks actual doors, both interior exit carpets, old door denial, Start and Continue. They do not establish earned campaign progress.

The broader mill/arts street candidate remains unselected; it has also received these normal house fronts, and must be rebased on this accepted repair before future publication.

## Verified result

- 32 changed native metatiles, nine entrances and nine safe return anchors. All731 other complete maps and all art, audio, cycling/gate programs, river geometry, encounters and recovery data remain byte-for-byte equal. Independent whole-root audit: `renders/minneapolis-house-normal-v264/whole-root-preservation-v280.json`.
- Production controller/MCP: nine working entrances, both interior exit carpets for each building, nine denied old door positions, Start and actual reached-save Continue;281 RPC calls and empty stderr. Receipt: `renders/minneapolis-house-normal-v264/final-production-doors-v285/receipt.json`.
- Eight complete homes match the original Pallet Town house in all four native palette banks. Nine complete facade footprints have zero native-render mismatches. Two fresh final-pack frames, including actual connected southern terrain, have zero full-frame mismatches; seven wider frames are inherited under exact complete scene/art equality. Original moving-cycle comparison differences are retained.
- Rust publisher build, guarded fixed-program regression and affected-crate all-target Clippy pass. Existing Clippy warnings remain. Changed Rust files pass targeted formatting; whole-workspace formatting remains blocked by pre-existing formatting beginning in `crates/offset-allocator/src/lib.rs:408`.
- All377 audio PCM hashes, frame counts and loop ranges pass.

[Fresh native house image](../../renders/minneapolis-house-normal-v264/final-native-v282/house-73/production.png), [all nine native fronts](../../renders/minneapolis-house-normal-v264/native-v271/production-decisions.png), [whole city](../../renders/minneapolis-house-normal-v264/source-v265/whole-city-after-original.png). All originals, rejected passes and conversions remain retained.

The selected source index inherits every parent source and replaces only the current Minneapolis layout/dossier. Historical city base layouts and old cyan frontage contracts are not the current repair source. Reapply this explicit overlay to its recorded current parent, rather than rebuilding the historical compact-city layout alone.
