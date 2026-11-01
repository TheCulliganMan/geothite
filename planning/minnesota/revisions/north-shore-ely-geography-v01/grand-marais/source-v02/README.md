# Grand Marais west coast / north inland — source v02

North Shore enters the **west** side of the installed 20×19 harbor. A separate
north lane between the Center and Mart leaves toward Gunflint and inland Ely
snow. This supersedes source-v01's south-quay proposal. The town stays green;
all six installed doors, rooms, services and local lighthouse behavior remain.

The inspected source epoch is `west-proposal-v02`, under the matching Grand
Marais render directory. Whole town, closed/open officers, valid player cameras,
the complete joined R11 coast and both Foot/Surf seam cameras were inspected.
These are native-source renders, **not GPU captures or controller execution**.
The dark area below the shorter town is outside authored map extent.

| Real seam | Town core coordinates | Neighbor core coordinates |
| --- | --- | --- |
| West North Shore Foot | `(0,8/9)` | R11C01R00 `(47,8/9)` |
| West Superior Surf | `(0,16..35)` | R11C01R00 `(47,16..35)` |
| North inland Foot | `(18/19,0)` | relocated Cedar `(56/57,37)` |

Town origin is `[246,-400]`. West offsets are `0/0`; north/Cedar south offsets
are `-19/+19`. The geography owner supplies the actual R11 terminal headland,
Cedar relocation and inland connector. The joined render uses exactly its
`geography/west-r11-headland-v01/contract.json`: 66+240 R11 cells. All 22 west
collision pairs match: two Floor and twenty Water. Town-only publication is an
intermediate stage until the neighbor geometry stage is applied.

The town's 28 explicit cell edits close the obsolete east Foot/water exits,
open the north inland lane and shift the existing west lookout bank inward.
The ten established reef recipes reuse old Johto glyphs and palettes. All
524,288 old-block pixels and 40,960 new donor-quadrant pixels match their source
in morn/day/nite/dark. No new glyphs or incidental Cut quadrants are introduced.
The full source layout compiles to the candidate's exact 380-cell array.

The original west checkpoint remains at `(2,8/9)`; the former east officers move
to `(18/19,2)` and their four facing/movement arguments rotate north. Existing
eight-badge and Duluth signal predicates remain exact. The local lamp quest
still has its own separate flag. Missing prerequisites block outbound travel;
the original inward mechanism hides the officers, moves the player two steps
inside, then reconciles the officers behind the player. Both closed officers
occupy the two lanes intentionally; passing state hides them.

Surf uses the existing west check at `(2,16..36)` regardless of facing, covering
bank launches and the bottom corner detour. A tiny inward wrapper at column 1
invokes the installed west inward mechanism only while facing RIGHT. The bank
shift keeps inward `1→2→3` and denial `2→3` entirely on native Water. Other
directions on column 1 do not push the player. Static graph proof checks closed
outbound refusal, open reachability, unqualified inward care and all six Foot
door approaches. Production Surf-mode and script/input ownership remain pending.

The fixed source parent is the actual 693-map inland-art pack, file SHA
`1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a`, identity
`317e9c039914d27c83dfed01ef55b4720023867fd9d8d5aef9ea1b1a452aec3e`.
The unchanged whole town preimage is
`377210a771f41efaa9c5bd18a9035462a61a14da7e34e0967d79421b5e6c93c1`.
Frozen `native-edit.contract.json` SHA is
`c47d93919126bc32f3aa8d6b6f9034aff5427ceacea866ba93044a9576852278`.

The bounded Rust API accepts unrelated upstream additions while guarding all
owned town/art/raw preimages. Its CLI requires the caller's explicit input file
SHA and a fresh output. It reuses the existing reef installer and preserves the
parent's inherited cached/uncached distinction. Full inverse residual checks
retain every unrelated map, program, encounter, audio, report and service.
The detached dossier/connection candidates retain the actual six-room lineage;
the alternate seven-room shore successor is not transplanted. Canonical/world
inputs remain untouched, with before SHAs in `source-preimages.json`.

Source rerender, using a fresh epoch each time:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/revisions/north-shore-ely-geography-v01/grand-marais/propose_west_coast.py \
  west-proposal-v03
```

After integration-owner registration, the pending native commands are:

```sh
CRYSTAL_MARAIS_SUPERIOR_TEST_PACK="$PWD/content-packs/minnesota-superior-inland-art-v01.crystalpack" \
  cargo test --locked -p crystal-assets grand_marais_superior_ -- --ignored
cargo run --locked -p crystal-assets --bin build_grand_marais_superior -- \
  content-packs/minnesota-superior-inland-art-v01.crystalpack \
  1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a \
  planning/minnesota/revisions/north-shore-ely-geography-v01/grand-marais/source-v02/native-edit.contract.json \
  content-packs/minnesota-grand-marais-superior-v01.crystalpack
```

Three asset tests include native typed gate indexes, composition, stale inputs
and twelve full-residual corruptions. Registration, Cargo, native publication,
production controller courses and GPU review are integration-owner work and
are pending here. The wider statewide map/design goal remains incomplete.
