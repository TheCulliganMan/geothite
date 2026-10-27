# Duluth, Brainerd and Bemidji native integration

The current combined candidate is
`content-packs/minnesota-cities-review.crystalpack`: 498 maps, file SHA-256
`37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6`.
Internal content identity:
`423ea59cd2b6ea3288c0784f8b464c5c6240cc5e0b1a7b787a2605d34aee18d4`.
Rust publication and strict written-pack loading pass. Its 377 audio programs
retain every PCM hash, frame count and loop range.

This candidate is under review. The first written-pack audit found the
Bemidji gym's global `map_scripts` event mirror still targeting the review hall,
although the map module's events and raw commands already target the city.
The builder now refreshes that mirror and its test reassembles the actual gym
from split payloads. The repaired publication passes all 81
[written-pack preservation checks](../renders/cities/native-handoff/comparison-4e6d9c999f1f9aae20b4ca9e1be0515f0b26f93538394e01727ba7522e7a4dee.json).
The first candidate remains in
`.local/minnesota-city-revisions/cities-6e3295367d6f2b4f6b07ef8f95f403968f2dcd3fe4790991b6e5f7380ef879af.crystalpack`,
with both failed comparison reports retained. The repaired pack also has an
immutable SHA-named copy in that directory.

## Exact input epoch

The base is the 480-map
`content-packs/minnesota-city-gym-base.crystalpack`, file SHA-256
`8e8de35845148a949f5d6dcbc9d493a4785dda2c8bcd247b8ef9d1b5c4a5726a`.
It installs the captured 887afc43 canonical gyms on the Northstar story plus
garden/pine revision. All earlier routes, yellow prairie, rocky Iron Range,
North Shore rivers, Boundary Waters and Grand Marais remain present.

The publication captures seven JSON inputs before construction. It does not
mix later live gym edits into a frozen city reference:

- Gyms: `.local/minnesota-city-source-freeze/gyms-887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189.json`.
- Duluth: `.local/minnesota-city-source-freeze/MnDuluth-932ac34a722a1600a92f90933734efe357bdf86732ad87f21759bf1360eacf6a.json`;
  layout `renders/duluth/source-snapshots/MnDuluth.layout-213a8b6dc5f38c07fb89ca990cb70dab130e6e3abeb09418ec4c5bf86d9ef06b.json`.
- Brainerd: `renders/brainerd/native-asset-input-captures/MnBrainerd-a6e99c25936262a775e76ddb29721e308ca140c37ef6dc36bf4eeb9fbea6c7fa.json`;
  layout `renders/brainerd/native-asset-input-captures/MnBrainerd.layout-2af837ce974c8c9c34e1de98153a090ec234f425e62c1900526c992591f577a9.json`.
- Bemidji: `renders/bemidji/canonical-inputs/MnBemidji-c9cc0571747fa1b43b399ae3d23573ac8f66c791e58a7debb0c181cd5dfcb571.json`;
  layout `towns/MnBemidji.layout.json`, SHA-256
  `bb51f6f46fcfd75d2dd3e2db482d6490d94889688f246f1072cc1626b236ae41`.

Paths beginning `renders/` or `towns/` are relative to this planning folder.
The publisher is `crates/crystal-assets/src/bin/pack_minnesota_cities.rs`:
`BASE GYMS DULUTH_TOWN DULUTH_LAYOUT BRAINERD_TOWN BRAINERD_LAYOUT BEMIDJI_TOWN BEMIDJI_LAYOUT OUTPUT`.
Root owns publication; city workers do not run competing exporters.

## What is installed

Eighteen maps are additive: three complete exteriors and fifteen ordinary
interiors. Three existing canonical gyms retain their geometry, trainers,
puzzles and rewards, with their three return warps rebound to actual city
facades. Towns provide native healing, paid shops and PC interactions. Brainerd
contains its optional gift and badge-gated Surf ranger; Duluth has the Q07
eight-badge/depot receiver; Bemidji's officer remains ordinary dialogue until
the main-story handover is bound.

Bemidji adds two recovery destinations in the separate 256+ identifier namespace.
The original HOME/recovery records and source Fly range remain unchanged.
Core catalog tests pass 7 cases. Native component Teleport and blackout actions
reach the new destinations. The corrected Fly parser fixture passes positive
controls 0/27 and rejects 28/255, both added recovery IDs and 65535, retaining the
original Fly range. Clinic recovery registration for the
other two new towns remains open.

## Review and verification

Root viewed all three whole-town source renders. Native-source rasters from
the actual written pack are now shown in the [connected atlas](../ATLAS.md):
20 routes, 5 towns, 43 paired interiors and six unconnected gym review rooms at
one exterior scale. Actual doors are paired
with rooms, including tags AA onward. City links stay dashed because the new
regional walking seams are not installed.

All 17 asset tests pass against the exact frozen 887 inputs: Duluth 7, Bemidji 5
and Brainerd 5. They include real native HP/status/PP healing, paid shops, paired
warps, ledges, Q07 prerequisites and preservation/refusal cases. All nineteen
production-controller cases now pass: eighteen in the corrected combined run,
then the final Bemidji battle-loss/recovery case in a targeted retry. They cover
all doors/carpets, ledges, healing/PC, paid shops, gifts, actual trainer battles,
Surf grant, Q07 signals, native Teleport/blackout recovery and Continue. Logs:
`.local/minnesota-cities-controller-887-corrected.log` and
`.local/bemidji-city-loss-review-level70.log`.

Initial placement, party, badges and RNG are explicit local fixtures. The losing
battle uses a legal level70 Magikarp with Splash so intermediate HP damage and
spent PP can be observed before native blackout heals the party; no state is
changed after fixture setup. Continue comparisons retain every true event and
engine flag, treating absent and explicit-false sparse entries alike. These
fixtures do not prove earned regional travel.

Earlier shared move-index and Grand Rapids connection-builder compile failures
were repaired before the successful runs. Failed and corrected logs are retained.
The low-memory review uses one Cargo job and
`--config profile.wasm-dev.opt-level=0`; the renderer's patched allocator retains
`--config profile.wasm-dev.package.offset-allocator.opt-level=2`. Production
performance has not been measured with these review settings. All failed and
interrupted build logs remain retained.

Final visual acceptance, regional seams, upstream earned progression, Glacier
Line rides and fairgrounds implementation remain open. Every render, native
readback, source snapshot and rejected pass is retained.

The three workers also delivered exact, uninstalled seam contracts:
[Brainerd](../reviews/brainerd-seam-contract.md),
[Bemidji](../reviews/bemidji-seam-contract.md) and
[Duluth](../reviews/duluth-seam-contract.md). These identify the actual clipped
chunks, signed reciprocal offsets, both walking lanes, required border edits,
safe refusals and story predicates. The current world uses `gatehouse_warp`
records; implementing the proposed walking connections requires an explicit
source-contract reconciliation. Static candidate collision proofs do not
establish native crossings or earned travel.
