# Preserve room stories while accepting newer terrain

All review is native 2D. Red Wing's station remains excluded. These local
compositions do not replace `world.json`'s shared terrain selector.

`rebase_accepted_terrain` applies the difference between two completed accepted
packs to the interior branch. The contract pins all three content identities,
the complete changed-map inventory and every changed cell's preimage. The Rust
builder rejects changes to programs, actors, warps, native art, audio, dimensions
or catalogs in the accepted pair. It preserves newer room stories and services,
publishes native mirrors, runs the full geography guard and strictly reads back
the output. Choose a fresh output name; existing publications are preserved.

The completed local chain is:

| Stage | Accepted terrain | Interior output |
| --- | --- | --- |
| `contract.json` | 34 R03 grass cells, 53 R13 cave cells and border | `minnesota-interiors-terrain-sync-v01.crystalpack` |
| `r17-contract.json` | 45 R17 cave cells and border | `minnesota-interiors-terrain-sync-v02.crystalpack` |
| Falls candidate | One real southern R17 road, clinic and shop; checkpoint at the actual Rainy River outlet | `minnesota-interiors-falls-one-road-v02.crystalpack` |
| `r04-r20-contract.json` | 11 R04 grass cells, 48 R20 cave cells and border | `minnesota-interiors-terrain-sync-v03.crystalpack` |
| `r04-r20-scout-contract.json` | Same terrain, with the newer scout moved out of the original R20 return | `minnesota-interiors-terrain-sync-v04.crystalpack` |
| `../rail-stop-correction.json` | Remove the compact parent's stale St Paul Red Wing timetable entry | `minnesota-interiors-current-no-red-wing-v01.crystalpack` |

The R17 revision initially and correctly rejected the newer lichen reader's
old wall position. Its explicit relocation moves only that ordinary resident
from `(8,10)` to the alcove at `(6,10)`, approached from `(5,10)` and reachable
from the real entrance. The original 187-story source remains unchanged. Native
speech, identity and all other actors/programs remain intact. Service, gift,
trainer and progression actors cannot use this ordinary-speech exception.
The R20 controller then caught a second real composition defect: the newer
scout at `(6,9)` lengthened the original ledge return from 15 to 17 inputs.
The v04 contract moves that ordinary resident to the eastern alcove `(12,7)`,
approached from `(11,7)`. Both original 15/13-input returns, both ledges, stairs,
mouths and original potion now pass unchanged. Its fresh native 2D capture has
zero differing pixels and was personally inspected with the whole cave.
Controller review uses `CRYSTAL_INTERIOR_ACTOR_RELOCATIONS` with the cumulative
`resident-relocations.json`, rather than editing the frozen story contract.

Verified publication command for the latest stage, from the repository root:

```sh
target/interior-life/wasm-dev/rebase_accepted_terrain \
  content-packs/minnesota-interiors-falls-one-road-v02.crystalpack \
  content-packs/minnesota-r17-talus-den-v01.crystalpack \
  content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  planning/minnesota/interior-life/terrain-sync-v01/r04-r20-scout-contract.json \
  content-packs/minnesota-interiors-terrain-sync-v04.crystalpack
```

The written v04 has 712 maps and 214 non-gym interiors, file SHA-256
`b82d10d98798c4df746d8ff544d946f7cc32caf540e07f74ee403452cbc4f826`,
identity `12bdd1403deae8a77f7b2e03f7ed8900409f5f32a5c2216bd487f1016fbb8f46`.
Its final terrain step leaves 710 whole maps exact, including repaired St Paul,
and changes only the named 59 cells, R20 border and explicit scout position.
Compared with provisional v03, only that scout and its event mirror change.
All original runtime files,
audio and unaffected catalogs/programs are exact. The preservation receipt is
`renders/interior-life/terrain-sync-v03/preservation.json` and
`renders/interior-life/terrain-sync-v04/preservation.json`.
The final [rail correction](../RAIL-STOP-CORRECTION.md) then removes one stale
visible St Paul timetable listing inherited from the compact parent. Its only
changes are that existing text's typed body/raw payload and publication identity;
the complete repaired city exterior and every other map remain exact. Use that
corrected child for the next composition, rather than restoring the v04 listing.

Both terrain rejection regressions pass, including stale identities, overlapping
cells, changed catalogs, duplicate installation, wall/warp/unsafe approaches and
attempts to move an original pickup as conversation. The 117 newer residents
can all be approached and spoken to on the exact v04 child (137.00s). All three
R20 controller cases pass there (37.21s), and all 377 audio programs pass again.
Three fresh St Paul controller cases pass on v03: 39 ordinary entrances/returns,
both clinic services and PCs, both bridges, Surf underpasses, river seams and
Save/Continue. All 377 audio programs pass PCM/hash/frame/loop verification.

Changed Rust leaves pass `rustfmt --check`. Workspace `cargo fmt --all --
--check` still fails first at the existing `offset-allocator/src/lib.rs:408`
formatting. Normal all-target assets Clippy stops at the existing core
`bad_bit_mask` denial in `multiplayer/mod.rs:1332`; this work does not change it.
Normal all-target Bevy Clippy stops at the same core denial. The supplemental
all-target check of both affected crates completes with `--no-deps` and only
the existing `bad_bit_mask`/`never_loop` denials allowed; baseline warnings remain
and no diagnostics occur in the new terrain, relocation or journey leaves.

Falls remains a candidate until its full regional circuit passes. Its room,
care, PC, shop and actual southern-road checks pass. The first corrected run
also exposed an obsolete R12 arrival fixture and the journey planner's missing
ice/gate handling; those checks are being repaired, without weakening the game
guards. Ten fresh native 2D comparisons pass (five town views, both rooms,
the lichen alcove, southern seam and actual winter outlet), and the whole town
and retained player views were visually inspected. A review-tool correction
derives camera bounds from immediate neighbors while composing two-hop scenery,
matching the production renderer; rejected comparisons remain retained.
The deterministic Falls preservation/rejection regression also passes (31.46s):
original winter terrain and every existing actor/program/event remain exact;
only the new checkpoint, warden, actual-arrival callback and three new maps are
added. Its refreshed split-payload assertions cover the coordinate triggers and
warden rather than expecting the withdrawn direct town-to-icefields shortcut.
The regional journey case still fails to plan the complete course; the three
other Falls controller cases pass and the circuit remains explicitly pending.

The three fair rooms, passenger service and broader regional quality/progression
acceptance remain open. Neither room counts nor local setup fixtures establish
completion of the all-interiors objective or earned campaign progression.

The independent shared terrain branch has subsequently accepted R01 limestone
hollows on the R20 parent (29 cells/border). That next terrain delta has not yet
been composed into this candidate; do not rewind the shared selector to v04.
