# Iron Range fitted crosscut

The existing 16×13 approach is recomposed around its actual encounters and
connections. Continuous native rock replaces repeated boulder rectangles; a
western bench holds the original Ether, a narrow northern traverse joins a
bending ore passage, and the southeast Spoil encounter has its own turnout.
One short westward gravel lip provides a useful return across the rock rib.
The lip has no orthogonal cliff or stair neighbor. Every ordinary floor tile
remains reachable without jumping.

The verified local `content-packs/minnesota-iron-crosscut-bends-v01.crystalpack`
and its composed source/entrance records are selected through
`planning/minnesota/world.json`. The Gunflint grotto, newer research services
and all other maps remain intact.

`after.json` owns the complete hand-authored native rows. `before.json` is the
full current terrain preimage; `contract.json` binds the original complete map,
parent pack, actors and warps. `plan.json` records every connection, activity,
flag and verification requirement. All rejected sources and renders remain
retained. No original native graphics, palettes or game catalogs are copied.

| Feature | Native walking coordinates | Contract |
|---|---|---|
| Public mine ladder | active `(7,21)`, passive `(6,21)` | Returns to `MnR09Cave1F (19,9)`; the existing foreman gate opens its actual `(19,11)` return entry |
| Haulage ladder | active `(25,5)`, passive `(24,5)` | Enters `MnIronHaulage (4,23)`; step to `(5,23)` then up into its active `(5,22)` ladder to return |
| Ether bench | ball `(5,5)`, approach `(4,5)` | Original Ether and `EVENT_MN_IRON_APPROACH_ETHER`, unchanged |
| Bench climb | native30 at metatile `(3,4)` | Ordinary up/down travel on walking column6 |
| Access encounter | `(10,12)`, facing left | Original team, sight, dialogue and persistent victory flag |
| Spoil encounter | `(26,18)`, facing up | Original team, sight, dialogue and persistent victory flag |
| Return lip | sources `(13,18)` and `(13,19)` | Native HOP_LEFT jumps to `(11,18)` and `(11,19)`; course `(12,y)` is blocking wall |

Both lip lanes have blocked reverse and side shortcuts. Their ordinary uphill
walkarounds take 27 and29 steps. From the actual Spoil-side approach `(25,18)`,
the entry return is38 ordinary walking steps or21 modeled movement commands
using the lip, counting a two-tile jump as one command. It shortens the return
from an existing activity. No new quest, opponent, item, HM or fee is added.
The original rate-10 cave wildlife remains intact all day.

The executed Rust publisher command is:

```sh
.local/iron-crosscut-publisher-v934 \
  content-packs/minnesota-gunflint-grotto-v02.crystalpack \
  45fa571daa78535af329f6c8f26a8770d24c56cbb1a5223942d5fbb1fc0979ca \
  planning/minnesota/caves/iron-crosscut-v01/before.json \
  planning/minnesota/caves/iron-crosscut-v01/after.json \
  planning/minnesota/caves/iron-crosscut-v01/contract.json \
  content-packs/minnesota-iron-crosscut-bends-v01.crystalpack
```

Build it with `cargo build --locked -p crystal-assets --bin rebuild_iron_crosscut`
under the shared serialized Cargo slot, using the pinned toolchain. Publication
requires a fresh ignored output and strict Rust readback. Stale native modules,
unsupported art, stranded floor, incorrect stairs, a cliff-adjacent or reversed
lip, extra mouths and open borders are rejected. Only this map's blocks, their
owned encoded payload, the publication marker and derived identity change.
All734 other complete maps, original programs, actors, teams, native assets,
wildlife, fishing, recovery and audio remain exact.

For a fast whole-map source loop:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-gunflint-grotto-v02.crystalpack \
  --source planning/minnesota/caves/iron-crosscut-v01/after.json \
  --output planning/minnesota/renders/iron-crosscut-next-source --watch
```

The last geometry revision rendered in0.066 seconds after pack loading. The
first repeated upper alcove and the broad northeastern aisle were rejected and
reworked before publication. All originals, variants, conversions and native
captures remain retained.

Four current controller cases pass: both lip lanes and reverse denial through
Continue, the stair and both real ladder pairs; both actual original trainer
wins with PP/prize changes; native Ether use restoring up to10PP on the selected move; and full-pocket
denial/TOSS/retry/persistence. The initial return test used the historical
Haulage `(5,23)` coordinate as its active ladder. The installed native map shows
the actual active quadrant is `(5,22)`; the corrected two-step course passes.
No gameplay connection was changed to fit the test.

An independent1420-call public stdio trip passes with empty stderr and a
fresh-process Continue after the first jump. All ten full640×576 native frames
match the source at all368,640 pixels each. The complete source and native
camera sheet were inspected at the original night palette. The source audit
checks both collected/uncollected cache-body states. Actual reached saves and
native views preserve the lip and its landing.

Controller fixtures declare original Minnesota partners, prior public-shaft
access, cleared-guard geometry scenarios, depleted PP or full capacity before
input. The battle case leaves both local opponents unbeaten. Every later
movement, battle, pickup, PP restoration, Toss, return and Continue uses
production input. These cases do not prove earned regional arrival or level
balance. Old saved terrain overrides are not migrated; broader Minnesota map
quality work remains active.


Both affected crates pass all-target Clippy with existing unrelated warnings;
crystal-bevy is checked with `location-tester`. Owned Rust formatting passes.
Workspace-wide formatting still reports existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`. The regional policy audit checks4,626
references against225 active Minnesota species, including current fishing and
trainer parties. All377 audio programs pass PCM hash, frame-count and loop-range
verification. A fresh Rust publication replay is byte-identical to the selected
pack.

- [Whole fitted crosscut](../../renders/iron-crosscut-v932/final-source-v947/8416ec371536-1793461255674286000/whole-2x.png)
- [Ten native cameras](../../renders/iron-crosscut-v932/native-v936/MnIronDrillApproach-production-cameras.png)
- [Actual native lip](../../renders/iron-crosscut-v932/acted-v946/lip-18.png)
- [Actual native landing](../../renders/iron-crosscut-v932/acted-v946/landing-18.png)
- [Final acceptance](../../renders/iron-crosscut-v932/acceptance-v949.json)
- [Public production replay](../../renders/iron-crosscut-v932/mcp-v941/receipt.json)

`check_mcp.py --help` documents the independent trip. Use the emitted
`gameplay-v943/geometry-initial.crystalsave` only as the declared initial local
fixture for this exact pack identity. The four current controller cases share
filter `iron_crosscut_controller`; set `CRYSTAL_WINONA_TEST_PACK` and
`CRYSTAL_RENDER_TEST_PACK` to the completed pack, plus a fresh
`CRYSTAL_IRON_CROSSCUT_REVIEW_DIR`, and run them serially with
`--ignored --nocapture --test-threads=1` in crystal-bevy's default test profile.
