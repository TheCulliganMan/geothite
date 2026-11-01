# Bemidji timber roof and plateau repair

The user identified two defects in the whole-town winter preview: the heritage
building lacks its top roof row, and the elevated western district has an open
east side without stairs. This revision fixes those exact defects.

The source candidate adds original Johto `2c/2d` above the existing timber
`22/23,28/29` façade. This is the same complete three-row composition seen at
Violet City's tower. The current lower two rows, entrance and passive return
stay in place. Its native trees/roof collision remain blocking.

| Cell in 32px metatiles | Before | After | Purpose |
| --- | --- | --- | --- |
| `(4,16)` | `02` | `85` (Johto `2c`) | Full left top roof |
| `(5,16)` | `01` | `86` (Johto `2d`) | Full right top roof |
| `(9,8)` | `04` | `69` | Join the native east cliff face |

The east face closes gameplay positions `(19,16)` and `(19,17)`. Its inner
shoulder at `x=18` stays walkable. The existing south staircase retains the
`x=11, y=20..21` lane. The heritage door remains `(9,37)` with the original
indexed warps. No event, road checkpoint or story prerequisite is added.

`before.json`, `after.json` and `contract.json` bind the complete current module,
rows and parent pack. The Rust publisher appends two exact native blocks to the
existing **private** `mn_bemidji_plateau_stairs` art identity. Shared Johto art and
all other towns remain unchanged. It rejects any extra terrain edit and compares
the complete normalized result with its parent before writing.

## Current status

The source previews and three untouched native GPU images were inspected: full
roof, supported cliff and retained south staircase. The Rust-built candidate is
`content-packs/minnesota-bemidji-roof-plateau-v01.crystalpack`, SHA256
`d14d30be991a78440e802bae1b5b5670be837e9c2dddef7017978952c3215fdd`,
identity `89016b35a870fd0589fd34648ca982c3cdd29a67b2125cb58e4cbb0d0117bcc6`.
It has not been selected, deployed or started in a running client.

Seven production stdio cases pass: both cliff quadrants from both sides, south
stairs both ways, blocked roof approach and actual heritage-house entry/return.
All seven also pass Start and Continue with zero badges and empty stderr. The
first replay stopped one input short of the house's native doorway transition;
its rejected log/saves remain under `gameplay-v1049`. The corrected complete
input replay and full RPC record are under `gameplay-v1054`.

The parent comparison proves 742 other complete maps exact; every original
Bemidji field except three cells exact; all original art bits/palettes and
metatile recipes exact; all 602 audio entries/programs/payloads exact; field-move
replacement data exact. The added roof reuses existing glyphs, so only the
private collision table and appended metatile file change. All 589,824 whole-town
source/installed pixels match. The active225 roster policy audit passes.

The dedicated publisher builds and both owned Rust files pass formatting.
Workspace-wide formatting reports unrelated `offset-allocator` differences.
The required all-target Clippy attempt encountered the concurrently authored
`pack_northstar_public_roads` CLI before its library export was installed
(`E0432`); its full log is `.local/bemidji-roof-clippy-v1050/clippy.log`.
The focused library/publisher Clippy check is queued behind the current shared
Cargo owner, with its result to be recorded at `.local/bemidji-roof-clippy-v1056/`.
These limitations are not evidence of a new roof/stair gameplay failure.

New source files are `bemidji_roof_plateau.rs`, its dedicated CLI and the two
module/re-export lines beside the existing Bemidji stair registration. No other
Rust source edits are needed for this candidate. No selector, branch ref, commit
or running client changed. Winter expansion remains isolated and unfinished.

## Fast full-town review

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_bemidji_roof_plateau.py \
  --pack content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  --source --output planning/minnesota/renders/bemidji-roof-source-NEXT
```

Choose a fresh output path for every pass. The measured warm composition after
pack load is **0.204 seconds**. The renderer writes the full town plus tower and
plateau close-ups; source matching alone does not establish gameplay acceptance.
The `--source` switch is omitted when checking an installed candidate.

Inspected originals and 2x views, including the snowy candidate, are retained in
`planning/minnesota/renders/bemidji-roof-plateau-v1046/`. The snowy rendering is
an isolated art proposal, not a selected winter pack. Apply this roof/cliff
revision before rebasing the winter core; the earlier winter-core-v03 rows
contain the rejected two-row tower and must never overwrite this repair.

The previous workspace-wide formatting command still reports unrelated
`offset-allocator` changes. The two new Rust files were formatted separately;
full crate checks remain pending with the build queue.
