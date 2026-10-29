# R09 quarry grass and dry ore return

Selected as `world.route_grass_r09_revision`. Native and production-client replay passed.
Parent: selected704-map `r03_bank_seep_revision`,
`content-packs/minnesota-r03-bank-seep-v02.crystalpack`, SHA256
`c6fdc6bdad16f49b5a0cc8ea0efcf7f3500d2fa61e9494e69492d4b540e6a96a`.
Child: `content-packs/minnesota-current-grass-r09-v01.crystalpack`, SHA256
`3ed13b6eb3a8d4c3e379fca8c4a721674647b426302528c80149c8d2eb8845fb`.
Identity: `c1d2a410644398812f28b499825c9605669f60861c13d466ff8948b22543eb35`.

## Terrain and Crystal reference

The saved authentic Crystal Route45, complete R09 source map and three actual
production2D cameras were visually inspected. Eight grass metatiles bring native
encounter habitat onto the north bench, western upper meadow, lower ore-track
edge and quarry shoulder. Four attached spoil boulders close real cliff-foot
bypasses around that shoulder. Grass increases49→57metatiles,196→228walking tiles.
Existing quarry water, rock ribs, mine, ore track, rail and spruce neck remain.
The patch reads as reclaimed quarry ground between existing rocky walls.

The quarry-shelf circuit requires four grass steps uphill; return via the
original spoil ledge and lower ore track is dry. This is the named quarry
circuit, not a claim every intercity journey crosses the new shoulder. Both
ledge lanes retain reverse denial and the ordinary long climb. The existing
rail trainer-versus-grass choice and mandatory spruce habitat also remain.
The4/0result holds before/after Cut and without actor bodies. Four adversarial
variants (dry uphill, cliff-foot bypass, removed reverse wall, grassy return)
are rejected by the published proof.

Renders (relative to planning/minnesota):
`renders/route-grass-v01/current-r09-v01/source-review/` retains the complete
map, weak cameras, rooms, Cut states and originals. Three production2D captures
are `r09-spoil-v01-2d.png`, `r09-ledge-v01-2d.png`, and
`r09-north-bench-v01-2d.png` in the parent directory. No3D/2.5D views were made.

## Publication and preservation

The Rust terrain publisher validates the exact native preimage, installed
anchors, revised complete geometry, compact warm coast before/after, and strict
written readback. The bound dossier freezes current Minnesota trainer parties
and current mine rows/border, including the intentionally closed story shaft.
It does not publish or modify those actors or room cells.

The independent735-check preservation audit proves only12cells in2exterior
R09chunks and their encoded block mirrors change. All702other maps, including
R09mine and all current story/caves, plus every event, script, trainer party,
encounter, runtime art, audio, HOME and recovery field remain exact.
Four preservation mutations are rejected. Receipt:
`renders/route-grass-v01/current-r09-v01/preservation-v01/proof-430d4d7ccc0ee4096ac094fcd13bdad63b02be8d7243c0bc3a5da0927e54feaa.json`.
All2304native metatiles match source; all2,359,296terrain pixels match original
source recipes. `source-current-receipt.json` records this, all4directional
states and the fresh zero-ICE scan over R11/Duluth/GrandMarais. R11 remains on
mn_north_shore. All377audio programs pass hashes/framecounts/loop ranges:
`.local/route-grass-review/current-r09-v01-audio.log`.

Source review also checks rail-only passage, timber bridge, sealed Cut sample
camp and87opened walking tiles, field medic, native route openings and all
interactions. Published directional receipt:
`renders/route-grass-v01/current-r09-v01/R09-published-directional-be95799afade2a56.json`.

## Replay

Publish only to a fresh output with the guarded Rust binary:

```sh
target/route-grass/wasm-dev/revise_authored_route \
  content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  planning/minnesota/revisions/route-grass-v01/current-r09-v01/MN_R09.before.layout.json \
  planning/minnesota/revisions/route-grass-v01/current-r09-v01/MN_R09.after.layout.json \
  planning/minnesota/revisions/route-grass-v01/current-r09-v01/MN_R09.installed-dossier.json \
  content-packs/UNUSED-FRESH-OUTPUT.crystalpack
```

The named child already exists; do not overwrite retained packs/renders.

R10-R20, outdoor regional links and final current-story coverage remain open.

## Native gameplay and production client

Five fresh native cases pass on this exact child: exterior geometry including
clipped seams/rails/quarry drop and ordinary return, populated trainer battle/
selected Surf PP/medic/Continue, new reclaimed-grass wildlife, native Cut denial/
opening/sealed cache/reset, and current mine cameras/stairs/drops/mouth/Continue.
The cave case also verifies the exact closed lower story shaft overlay.
Logs: `.local/route-grass-review/current-r09-v01-controller-final.log`,
`current-r09-v01-cut-final.log`, `current-r09-v01-cave-final.log`.
The128step new-grass loop produces5actual wild encounters resolved with RUN.
The trainer battle observes exactly the current native team AGATOT/LATCHQUILL/
AUGRUB, not the superseded Johto team. Actual party damage and PP expenditure
are restored by the field medic, then retained by Save/Continue.

The geometry case starts with an explicit empty-party local placement. The
populated case deliberately supplies source-compatible Feraligatr48/Ampharos46,
initial injury, seven badges, position and clock/divider setup. All later actions
use production inputs. These are behavior/persistence fixtures, not earned
regional progression or battle balance. No authored/native Minnesota team,
encounter table or other creature placement is changed by these test fixtures.

The geometry reached-save passes13production JSON-RPC calls: actual local
movement(60,24)→(59,24), Start/Pack, collected HyperPotion/MaxRepel and UltraBall
in their proper pockets, and return to walking. Stderr is empty. Receipt:
`.local/route-grass-review/current-r09-v01-stdio.receipt.json`.
The separate populated reached-save is retained as `current-r09-v01-reached.crystalsave`.

Two stale assertions were updated in `bevy_shell/tests/iron_range.rs`: actual
opponents must equal the current runtime trainer roster, and the mine resident's
first page must contain Nell's current request. Native outcomes, PP, health,
healing, persistent flags and Continue assertions stay strict. Fresh test build
passes. The initial broad `iron_range` invocation also selected R10/story tests
without their pack environment; that failed log is retained and superseded by
the exact scoped native commands above. The old test executable's cave source
check lacked the closed-shaft overlay; the rebuilt current cave case passes.

Scoped rustfmt and affected-crate all-target Clippy (`--no-deps -A clippy::never_loop`) pass with existing warnings. Full workspace formatting remains blocked by pre-existing offset-allocator differences, beginning at src/lib.rs408; unrelated files were not reformatted. Logs: `.local/route-grass-review/current-r09-clippy-focused.log` and `current-r09-workspace-format.log`.
