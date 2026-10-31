# Minneapolis mill and arts streets

Six complete native facades now form closer street groups in the west bank. The museum and Arts Studio share a two-walking-tile street. A new two-tile passage connects that street directly to the real boathouse frontage. A continuous western lane keeps the Supply Cooperative connected to the museum. The Workshop court reaches Bridge Home and the existing riverside bench.

The nine previously corrected house/Mart fronts remain complete native assemblies. No new pixels or palettes, empty room shells, trainer teams, gates, water, ledges or river seams are added. The six moved buildings keep all existing room programs and activities.

## Exact current sources

- `before.layout.json`: full40×32 native preimage from the selected normal-house city.
- `after.layout.json`:100 deliberately changed metatiles, all remaining city cells explicit.
- `recomposition.contract.json`: every native anchor,20 changed positions:12 door/return anchors, six readings and two residents. All20 guard triggers and18 cycling starts remain fixed.
- `town.dossier.json`: all40 actual room pairings, nine complete current house assemblies, native actors/readings and retained original-Minnesota trainer parties. Historical foreign-team proposals are not copied into this source.
- `sources.json`: inherited parent source index with current Minneapolis layout/dossier overrides; generated when this revision is selected.

Final parent: `content-packs/minnesota-current-duluth-mart-v01.crystalpack`, SHA256 `d77baff3e597e6527450400022e0b9acf655b856983cf96e49fbebfe275464f3`. This preserves the newer Duluth Mart, Bemidji stairs, corrected Minneapolis houses and earlier accepted cave and regional work. Final native output: `content-packs/minnesota-minneapolis-mill-arts-v02.crystalpack`, SHA256 `6f02a4fd9a2294a5c04825ffc3873b07b5a46a4d1d93e5a4951ec9fe6125d6c1`.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/debug/recompose_authored_city \
  content-packs/minnesota-current-duluth-mart-v01.crystalpack \
  planning/minnesota/revisions/minneapolis-mill-arts-v02/after.layout.json \
  planning/minnesota/revisions/minneapolis-mill-arts-v02/recomposition.contract.json \
  content-packs/minnesota-minneapolis-mill-arts-v02.crystalpack
```

This command was executed successfully. For a rerun, use a fresh ignored output; resolve `world.json` and verify the entire native preimage before substituting another parent. The native publisher certifies the fixed north-pool gate/cyclist program epoch and does not republish unchanged neighbor payloads. Packs and exported module snapshots stay ignored.

## Walking and activity review

| Useful connection | Before | After |
|---|---:|---:|
| Museum to boathouse |121 steps|13 steps|
| Arts Studio to boathouse |94|18|
| Museum to Arts Studio |65|11|
| Cooperative to museum |19|19|

These are actual ordinary walking links, checked with native input. The museum-to-boathouse passage was also walked in reverse. The museum's existing activity is proven through real travel: the host initially requires all three river plaques, reading the actual water/grain/workers plaques enables the MAGNET reward, production Pack shows MAGNET x1, a repeat conversation grants nothing further, and Continue preserves the completed activity and item quantity.

All40 building destinations were visited and returned correctly. Both room exit carpets of all six moved buildings pass:46 distinct room exit carpets total. The initial automated tour exhausted its route bound after33 destinations because replanning every short segment chased moving cyclists. Its FAIL receipt is preserved. The remaining seven destinations were verified by continuing the untouched actual reached save using a committed-route walker; no party, position, flags or bag were rewritten. After rebasing on the newer Duluth parent, all six moved buildings/both carpets, the useful walking links, relocated readings, reward-once Pack quantities and actual Continue passed again through766 production RPC calls with empty stderr.

Eight retained production-controller cases pass on the identical city/room/art epoch: native cycling victory/battle return, autonomous patrol and declined challenge; denied Waterfall, qualified ascent/downward current, northern and southern water seams, and the missing-Plain-badge bypass denial. Their explicit native-Minnesota party/badge/placement setup is QA, not earned campaign progress.

## Renders and evidence

- Whole town: `renders/minneapolis-mill-arts-v289/source-v290/4330320ad75f-1793377727572882000/whole-town-original.png`.
- Five actual native street views: `renders/minneapolis-mill-arts-v289/native-v292/production-decisions.png`. All five match native source pixels. A fresh final-pack museum-street frame also matches exactly; four wider views are inherited under exact complete city, room, actor/program, party and runtime-art equality across the Duluth rebase.
- Exact whole-root preservation: `renders/minneapolis-mill-arts-v289/whole-root-preservation-v300.json`. All731 other complete maps, art, audio, species, parties, encounters, native programs, gates and recovery remain exact.
- Complete acceptance and retained limitations: `renders/minneapolis-mill-arts-v289/accepted-v305.json`.
- Actual connected state: `renders/current-connected-after-minneapolis-v304/full-state-native-current.png`. All103 outdoor maps share one direct-seam group, with no overlapping footprints or inconsistent connection cycles. This atlas is static geometry, not proof of earned statewide traversal. Its152 water-boundary flags are inherited unchanged and require reachable-Surf/bridge analysis before deciding which need barriers.
- All377 audio PCM hashes, frame counts and loop ranges pass: `renders/minneapolis-mill-arts-v289/audio-v303.log`.

The warm full-city source check/render takes0.256 seconds in this checkout. The generic renderer remains usable with `--watch`; every preview receives an immutable version directory. All originals, rejected passes, GPU frames, conversions and reached saves are retained. No Rust implementation changed in this street-data revision; the guarded publisher build/test/all-target Clippy receipts are retained with the preceding normal-house repair.

Other Minneapolis districts, broader map and cave quality, interior content and complete earned campaign acceptance remain open. This is a verified local street refinement. No browser deployment or save migration is claimed.
