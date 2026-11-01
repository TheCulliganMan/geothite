# Boundary Waters expedition on the current maps

The optional paid expedition is now configured and verified in the current
map continuation. It uses the existing green lakes, pine shores, grassy
portages, campsites and Root Beer Cabin. The old basecamp barricade is retired;
the actual preserve guards still block unpaid access. Ordinary story travel,
the survey shelter and free care remain outside the permit.

- [Actual native capture battle with Chris's back sprite](../../renders/current-safari-native-v1315/wildlife-menu.png)
- [Actual successful catch](../../renders/current-safari-native-v1315/ball-result.png)
- [Kris native back sprite, declared appearance fixture](../../renders/current-safari-controller-v1314/safari_native_compositor_draws_and_refreshes_the_saved_command_window/native-battle/kris-rock.png)
- [Unpaid court](../../renders/current-safari-native-v1315/unpaid-court.png) / [paid arrival](../../renders/current-safari-native-v1315/paid-entry.png)
- [Reached native stairs](../../renders/current-safari-native-v1315/granite-upper-stair.png)
- [Whole preserve](../../renders/current-safari-whole-v1308/whole-original.png)

## Exact integration

Parent: `content-packs/minnesota-reviewed-maps-v02.crystalpack`, SHA256
`110e658e66449fd6ca3dc52fb7de5fa3f0f2bd827d8103aa838a32d63193229c`.
Child: `content-packs/minnesota-reviewed-safari-v01.crystalpack`, SHA256
`f79f8a3226ef2b4d97d25b546871359797f006265beaa5dcded344024ec5e89d`.
The [fourteen-stage combined recipe](../reviewed-maps-v01/README.md) reproduces
the same bytes as `content-packs/minnesota-reviewed-maps-v03.crystalpack`.

That combined v03 pack was selected as `reviewed_maps_revision`. The current
local review selection includes Rochester, Grand Rapids, Mankato and Brainerd, v07 (SHA256
`d94e6ccfa6460f58730fff0e5b23a4167a050cc0709214625af9199718e70a7f`),
under `brainerd_lakeside_loop_revision` in `world.json`. Complete-root comparisons
prove every preserve/basecamp map, script, habitat and this configuration remain
exact. The original Safari verification below ran on v03; the
[combined review](../reviewed-maps-v01/REGIONAL-REVIEW.md) separates that retained
evidence from the nine Rochester, twelve Grand Rapids, six Mankato and four current Brainerd cases. No running game, browser
deployment or saved campaign was changed.

The unchanged authoritative [zone.json](zone.json) has SHA256
`4b31f5ce29f34f9302e769a4ddc4834c6dc6eea749d7561b607fc3a2671e3e7a`.
Rust `with_safari_zone` validates its actual geometry, actors, habitats, native
Minnesota roster/evolutions and rod tables. Admission requires the available
Surf permissions, confirmation and the real 500 fee; it grants 30 balls and 500
successful travel commits. Refusal/cancellation charges nothing. The ranger is
at basecamp `(30,9)`, return `(30,10)`; paid court arrival is local `(15,9)` in
`MnBoundarySafariC00R02`.

The independent preservation audit passes at
`renders/current-safari-preservation-v1306`. All terrain and native art remain
exact. Only twelve related text bodies/mirrors, the two obsolete basecamp
officers/event rows and the optional configuration change. All738 other complete
maps, retained actors, executable scripts, habitats outside the expedition,
rewards, original regions and audio remain exact. The actual permit guards,
later story guards and their identifiers remain intact.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin pack_safari_expedition
target/verdigrim-scene/debug/pack_safari_expedition \
  content-packs/minnesota-reviewed-maps-v02.crystalpack \
  110e658e66449fd6ca3dc52fb7de5fa3f0f2bd827d8103aa838a32d63193229c \
  planning/minnesota/revisions/boundary-safari-runtime-v01/zone.json \
  4b31f5ce29f34f9302e769a4ddc4834c6dc6eea749d7561b607fc3a2671e3e7a \
  content-packs/minnesota-reviewed-safari-v01.crystalpack
```

Those exact outputs exist; use fresh ignored names for another publication.
The executed build, strict readback, preservation and all 602 PCM/hash/frame/loop
plus desktop-presentation checks are in `.local/current-safari-publication-v1306/`.

## Current terrain contracts, not retired cliff jumps

The old paid review expected five high-cliff hops. Those were deliberately
removed by the accepted cliff repair. Current preserve terrain has **zero HOP
quadrants** and five native cave12 flights. Tests now press into the closed
former drops, traverse each real tread in both directions, refuse its side wall
and verify the exact step debit and Continue.

Stair top-lefts in 16px global walking coordinates are `(40,58)`, `(96,18)`,
`(90,26)`, `(110,32)` and `(106,80)`; the walking column is x+1, from y−1 to y+2.
Each flight is three successful commits per direction. Portages retain their
declared habitat and paddle alternatives. Northwind's current ordinary return
is 25 steps/six grass, replacing the historical 38-step/one-grass cliff-jump route.
Palisade verifies the complete six-grass portage rather than assigning an old
partial-path quota to the stair approach. No terrain or encounter rate was
changed to make these tests pass.

## Production input and native presentation

Eighteen controller journeys and two compositor cases pass on these exact pack
bytes. The first nine successes are in `renders/current-safari-controller-v1310/`;
the remaining eleven are in `renders/current-safari-controller-v1314/`.
The retained v1307 failure and v1310 partial-course failure identify obsolete
terrain expectations; they are not counted as successes.

Coverage includes real ranger payment and both cancel paths; all northern
portages/lake arms; all five stairs and closed faces; the actual cabin; native
fishing; party/current-PC delivery; Dex and nickname ownership; Cut permission,
reset and recut; legal Fly/Teleport; invalid escapes; walking retirement; 500
actual travel commits and safe return; and Continue at owned screens.
The selected trainer back stays on the native battle LCD beside Ball/Bait/Rock/
Run. Returning to the field removes it. Step-expiry text uses the field presenter.

The public region also passes six fresh cases in
`renders/current-safari-regional-guards-v1320/`: all twelve seam crossings,
every missing badge/signal/restoration refusal, free care, inbound escort,
Surf boundary denial and actually awarded early rods. Zero-cash free travel
and story requirements are preserved. Early fishing passes with zero badges
and no field moves. The region fixture uses legal native Voyagear; legitimate
public-water wildlife is escaped through production RUN instead of suppressing
encounters or editing state.

The [stdio replay](../../tools/check_current_safari_mcp.py) passes two processes
and 26 calls with empty stderr: actual generated wildlife, native cursor/command
ownership, Run/result, normal Start and fresh Continue. Input saves remain exact.
Its outputs are `renders/current-safari-mcp-v1313/`.

Nine actual played GPU frames in `renders/current-safari-native-v1315/` and both
native trainer variants were visually inspected. The full 640×576 menu matches
the fresh compositor with **zero differing pixels**. Warm rain remains visible
in field views, with no snow; the battle LCD and cabin remain clear. Originals,
whole maps, earlier failures and appearance-only comparisons are retained.

Initial partners, field moves, permissions, money, local placement and divider
replay are declared QA fixtures. Mechanical journeys make no post-input state
edits. The Kris compositor comparison changes appearance explicitly and does
not claim an earned female-trainer campaign. These checks do not establish
unboosted campaign balance, new throw/shake/audio effects, cross-pack save
migration or final regional selection. Broader final regional acceptance remains
open. Assets and Bevy all-target Clippy pass; unrelated workspace formatting
differences remain.
