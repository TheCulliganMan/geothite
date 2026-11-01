# North Shore passing room

Seven coastal decks now have four clear 16px walking tiles between solid rail
boundaries. Their approaches meet the existing route. The Duluth town mouth
is four tiles wide at x64..67, and Grand Marais's west mouth at y8..11. Both
retain the original eight-badge/lighthouse-signal checkpoint on every added
lane. The two original officers at each town remain. All story programs,
actors, stairs, ledges, doorways, encounters and native graphics are preserved.

This revision uses the selected fauna/title parent and preserves its title
and intro resources. `world.json` controls whether the candidate is selected.
The pack is `content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack`,
SHA256 `e951fddafee3a0c09a2356d4dd683890ac498e23daeb266d08c5f12d79ec1167`.
Only seven maps change; the other736 complete maps remain exact, including every
Boundary Waters exterior. Those lakes, portages and pines stay green/unfrozen.

Native caps85/86 copy existing floor01 and rail44 glyphs. Their outer collision
quadrants are WALL, leaving four FLOOR lanes; the old sprite-over-rail problem
is not hidden with a merely decorative fence. Every original metatile byte is
retained, with32 bytes appended. No new palettes, glyphs or painted art are used.
The lighthouse deck shifts two walking tiles west to keep visible water margins
on both sides. The rejected first pass, which erased those margins, is retained.

`before.json`, `after.json` and `gate-coordinates.json` are the complete bounded
publication inputs. `current-route-chunks.json`, the two town row-layout files,
and the composed town dossiers provide exact current source bindings while
retaining each town's story/activity documentation. The Rust publisher rejects
stale inputs, any terrain outside the reviewed footprints and any missing new
gate lane. It refreshes event wire rows without rewriting existing programs.

Verification under `../../renders/north-shore-width-fauna-v1029/` includes:

- All28 deck-lane walks in both directions, lateral walking, and repeated blocked
  attempts against both rails on every deck row.
- All16 missing-badge/missing-signal denials over both towns' four outward lanes.
- Qualified town-lane roundtrips, full Duluth/Grand Marais walking roundtrip,
  normal movement without any Surf move, and actual save/Continue.
- A separate1885-call production stdio roundtrip, six native saved checkpoints,
  Start menu and byte-empty stderr.
- Ten headless day GPU images from actual reached saves, plus nine earlier
  night/rain GPU images. All were inspected; original PNGs and contact sheets
  remain available. Day review: `native-v1032/nine-views-original.png`.
- Complete source preview and installed native route agree in all5505024 pixels.
  This static comparison is separate from live GPU particles and actor motion.
- All380 grass quadrants remain. The upper habitat still requires four grass
  steps each way, and both ordinary ledge returns retain their grassy detour.
- Strict pack readback, all602 PCM/frame/loop audio checks, the225-species regional
  roster audit, exact parent preservation, and byte-identical Rust rebuild.

The initial controller launch lacked the required64MiB test stack and aborted;
its log is retained. The next full-trip test used stale exported actor positions
and stopped at a moving NPC. The final planner replans actual button inputs
against live actor positions after every step; all three controller cases pass.
The source remains unchanged through those fixture repairs. No reached state,
position, party, flags, RNG or terrain is rewritten during gameplay verification.
Initial local position, prior badges/signal, legal level80 regional partners,
previously beaten coastal trainers, Repel and fixed day clock are named fixtures,
not earned campaign progression. The independent stdio trip does not grant the
coastal trainer victories.

Both affected crates pass all-target Clippy with no warnings in the new modules.
Owned formatting passes. Whole-workspace formatting still reports unrelated
existing differences; no broad formatting/refactoring was applied.

Executed Rust publication, using a frozen local publisher:

```sh
.local/coast-width-publisher-v1007 \
  content-packs/minnesota-fauna-progression-title-v01.crystalpack \
  8ea32c5704ae8e763941295e14bd035ebbeac9e0e6d84018d599aa1dd5b0c504 \
  planning/minnesota/revisions/north-shore-passing-width-v01/before.json \
  planning/minnesota/revisions/north-shore-passing-width-v01/after.json \
  planning/minnesota/revisions/north-shore-passing-width-v01/gate-coordinates.json \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack
```

Build the same publisher with `cargo build --locked -p crystal-assets --bin
rebuild_north_shore_passing_width`; it appears under the chosen Cargo target
folder's `debug/`. Publication requires a fresh ignored output name and the exact
current parent's SHA. Never overwrite a retained render or pack.

A fresh source review can run with the installed candidate and a fresh output:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_north_shore_footbridges.py \
  --pack content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  --output planning/minnesota/renders/north-shore-passing-width-next
```

Warm rendering and static routing take about half a second after pack loading.
The full route and both towns are also shown together in the retained connected
exterior source render under `../../renders/north-shore-wide-v999/connected/`.
Future regional changes must consume the current selected parent and composed
source index. Wider inland winter geography, the newer Ely town candidate and
other statewide map-quality work remain active; this revision does not claim
the complete Minnesota map or campaign is finished.

The world advanced during review, so the final child was rebuilt on the current
fauna/title parent. All three controller cases, the full stdio trip, original
native cameras, all602 audio checks and roster/preservation audits were repeated
on that exact child. A separate12-case stdio check exercises all eight free
inward lanes without badges/signal and all four qualified Marais inward lanes,
with unchanged badges, working Start menus and empty stderr. The earlier
Title-parent candidate and every rejected/accepted render remain retained.
