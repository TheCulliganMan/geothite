# Minnesota trainers: compact geography required

## Minneapolis cycling dialogue

The 18 optional riders in `minneapolis-cyclists.json` now have individual
conversations about commuting, errands, repairs, river rides, hills and meeting
friends. Their native lines are at most 18 columns. Teams, patrols, approaches,
sprites, flags and optional-battle settings are unchanged. The focused
[cycling text publisher](CYCLING-DIALOGUE.md) installs this revision on an
existing cast; its unselected local pack passes preservation, audio and four
controller regressions. The actor-adding cyclist builder must not be rerun on an
already installed cast. The existing progression-reward builder
already gives a free rebuilt Bicycle at the Minneapolis workshop after the first
badge; its installed reward must be retained when revising the dialogue.

## Missing battle-return commands

The optional Minneapolis cyclist builder previously emitted `reloadmapafterbattle`
in all 18 encounter programs without compiling the corresponding map-command
records. Winning then failed with `has no Rust runtime mutation`. Trainer builds
now compile those records, and `repair_minnesota_scripts` restores missing
Minnesota map commands from the existing pack's own scripts. It rejects conflicting
records and preserves existing scripts, geography, teams, artwork and audio.

Verified local repair commands:

```sh
CARGO_TARGET_DIR=target/minnesota-script-repair cargo build --locked \
  -p crystal-assets --bin repair_minnesota_scripts --bin verify_crystalpack
target/minnesota-script-repair/debug/repair_minnesota_scripts \
  content-packs/minnesota-current-hm-r15-v01.crystalpack \
  content-packs/minnesota-scripts-repaired-r15-v01.crystalpack
target/minnesota-script-repair/debug/verify_crystalpack \
  content-packs/minnesota-scripts-repaired-r15-v01.crystalpack --audio --desktop
```

The ignored repaired pack retains 730 maps and all 377 audio programs. Every PCM
hash, frame count and loop range passes. Preservation and fresh-build regressions
pass; all three production New Game starter journeys and the region-specific
stdio dialogue/input-ownership check pass. An actual cyclist victory executes the
return script, sets its defeat flag and releases overworld input. This descendant
is registered as `script_command_repair_revision` in `world.json`; later review
builds inherit the repair. Original packs and saves remain intact. The reported
opening warning was not reproduced on the current pack; this repair specifically
addresses the confirmed battle-return error.

## Geography and publication

New trainer and original-NPC publication requires the compact Superior geography
from `content-packs/minnesota-superior-compact-v03.crystalpack` or a verified
compatible descendant. The builder checks actual map dimensions and absence of
retired maps; a renamed pack or an inherited manifest cannot bypass the guard.
The original-NPC artwork revision path applies the same guard.

Canonical geography and verification belong to
[the compact Superior revision](../revisions/superior-gunflint-quality-v01/VERIFICATION.md).
The trainer builder never changes terrain, connections or shared geometry.

## Retired lineage

`minnesota-trainers-v01/v02/v03.crystalpack`, their
`minnesota-trainer-*-base-v01.crystalpack` parents, and the old
`minnesota-shore-gate-v02.crystalpack` recipe are historical. They are not current
selectable build implementations or valid publication parents. Historical packs,
source, renders and audit logs remain intact for comparison and save-compatibility
regressions. The placement source is now [history/expansion-v1.json](history/expansion-v1.json);
[historical verification](history/PRE_COMPACT_VERIFICATION.md) does not authorize
rebuilding that lineage.

## Migration status

No new pack has been published from stale geometry. The old 110-trainer placement
source cannot be applied unchanged to the canonical compact pack: two Gunflint
placements target retired maps (`MnR12C01R00`, `MnR12C01R01`), and four placements
target towns absent from that pack (`MnMoorhead`, `MnInternationalFalls`). Existing
Grand Marais and Gunflint positions also require a fresh placement review. The
map owner must supply the final merged compact parent before publication; retain
legitimate North Shore/Gunflint content and reauthor only trainer positions.

Collection-based scaling remains implemented: the highest-level currently owned
non-egg Pokemon in the party or any PC box sets difficulty, with authored regional
minimums. Minnesota species, legal moves/evolutions, defeat flags and battle saves
retain their existing contracts. Johto/Kanto behavior remains unchanged.

Read-only compatibility tests still load archived v03 without regenerating it.
All eight focused asset tests pass, including new guard regressions that reject
the stale parent, accept canonical compact dimensions, and reject a reintroduced
retired exterior. Both publisher binaries rebuild successfully. The rebuilt trainer CLI rejects the
stale regional parent before writing any output (`compact-guard-cli.log`).
Task evidence remains under ignored
`.local/minnesota-trainers-v1/`; no historical render is removed.

Changed Rust leaves pass scoped rustfmt and whitespace checks. Whole-workspace
formatting still reports unrelated differences. The required all-target Clippy
run stops at the existing `crystal-core` `bad_bit_mask` error, before checking the
asset crate (`compact-guard-clippy.log`); this is not a passing full lint run.
