# Iron drill chamber

The operating drill now sits in a hand-authored rock-cut bay, with a northern
control alcove, western crew space and a lower shaft forecourt. The full room
is 12×10 metatiles. The original machine pixels, crew, rival, guard anchors,
pressure/feed prerequisites and recovery choices remain. Two eastern witness
approaches now detour below the guard at `(18,12)` instead of walking through it.

`before.json` preserves the previous complete room. `after.json` is the complete
authoring source; `contract.json` binds the exact original module and shaft
records; `plan.json` records all scene paths and gates. Numeric machine callbacks
18 and 22 retain their native extended/retracted images in the private night
tileset. The shutdown control is approached at `(16,3)`. Its original player
movement ends at `(9,9)`; FINN walks from `(15,9)` to `(8,7)` and is restored there
on Continue. The actual live object struct, rather than the object's original
anchor, proves the played crew position.

The full native controller chapter starts in the public mine with the local
foreman, pressures, feed, witness, rival, shutdown and recovery flags false. It
plays those events and battles and returns through the real shafts to the
public mine. Separate eastern-entry fixtures prove both rare detours, the two
machine frames and dialogue input ownership. Initial setup supplies prior
regional access, eight badges, four original Minnesota level55 partners and
ordinary supplies. This is local chapter verification, not earned campaign
access or balance acceptance.

The independent stdio-MCP course uses two actual reached chapter saves: the
isolated feed checkpoint and the defeated-rival checkpoint. It verifies witness
frames/input ownership, denial before the rival win, shutdown refusal/retry,
crew movement, fresh-process Continue, handoff refusal/retry, the unacknowledged
quest flag, subsequent acknowledgement and the production Start menu. These
are separately loaded earned chapter checkpoints, not one continuous MCP run.

Executed publication on the current R17 parent:

```sh
planning/minnesota/renders/iron-drill-core-v601/publisher-v604 \
  content-packs/minnesota-current-r17-trail-guidance-v02.crystalpack \
  186c262b18d5174cb7ba9727358d4022bea9fce6d362529405728028ef92c60a \
  planning/minnesota/caves/iron-drill-core-v01/before.json \
  planning/minnesota/caves/iron-drill-core-v01/after.json \
  planning/minnesota/caves/iron-drill-core-v01/contract.json \
  content-packs/minnesota-iron-drill-core-v04.crystalpack
```

The frozen publisher was built with `cargo build --locked -p crystal-assets
--bin rebuild_iron_drill_core`, using `CARGO_TARGET_DIR=target/verdigrim-scene`
and the shared Cargo slot. It requires a fresh ignored output; retain historical
packs instead of overwriting them. Pack writing and verification use Rust.

Controller replay uses `.local/core-controller-v605 iron_core_controller_native
--ignored --test-threads=1 --nocapture`, with `CRYSTAL_WINONA_TEST_PACK` set to
the candidate's absolute path and fresh absolute output directories in
`CRYSTAL_IRON_CORE_REVIEW_DIR` and `CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR`.
The frozen test binary's source hashes are recorded in
`renders/iron-drill-core-v601/controller-binary-v605.json`.

Source review remains independent of Cargo:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-current-r17-trail-guidance-v02.crystalpack \
  --source planning/minnesota/caves/iron-drill-core-v01/after.json \
  --output planning/minnesota/renders/iron-drill-core-next-source --watch
```

Use fresh output folders. The measured warm source revision took 0.093 seconds.
Every render, rejected pass and native save capture is retained. The full native
camera comparison and acted save captures verify the night palette and actual
machine images; source thumbnails alone do not establish visual acceptance.
The five-floor review uses actual terrain at one scale and labels real shaft
pairs. It does not invent walking seams between separate floors.

All-target Clippy passed for `crystal-assets` and `crystal-bevy` with existing
warnings. The owned Rust leaves pass formatting. Workspace formatting still
reports pre-existing differences beginning at `crates/offset-allocator/src/lib.rs:408`.
The current acceptance record and selected pack/source index are bound in
`planning/minnesota/world.json`. Broader regional map improvement remains open.

[Whole chamber before/after](../../renders/iron-drill-core-v601/audit-v622/whole-before-after.png),
[eight native cameras](../../renders/iron-drill-core-v601/native-v622/MnIronDrillCore-production-cameras.png),
[all five mine floors and shaft pairs](../../renders/iron-drill-core-v601/five-floors-v623/connected-native-floors-original.png),
[MCP verification](../../renders/iron-drill-core-v601/mcp-v623/receipt.json).
