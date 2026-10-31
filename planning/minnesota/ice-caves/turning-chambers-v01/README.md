# Voyageurs turning chambers

The upper floor teaches how ice carries the player into rock and snow stops. An attached southern shoulder ends the first eastward slide; a northern tooth turns the route toward the lower ladder. The Ice Heal remains a reachable supply pocket before the main crossing. The lower floor now has two broad ice basins divided by offset glacier tongues. Reaching the NeverMeltIce shelf requires a sequence of turns around those tongues. The Max Revive is a separate, easier western branch.

This keeps substantial connected ice: 144 walking quadrants upstairs and 224 downstairs. The room sizes remain 14×12 and 16×12 metatiles. The original guide, Voyageurs canoe surveyor, all three rewards, event flags, native blue palette and every warp/program remain exact. There is no new HM, admission charge, trainer quota or chapter authority.

## Connections, stops and rewards

The real exterior entrance remains `MnR20C01R00 (6,33)`; the passive outside return is `(6,34)`. Entering lands on upper-floor passive warp 3 at `(5,19)`. Upper active ladder `(23,5)` enters the lower floor at passive warp 2 `(5,19)`. Lower active ladder `(5,21)` returns to upper passive warp 4 `(23,6)`. Upper active exit `(5,21)` returns outside. FLOOR arrival records are separate from the active LADDER records and must not be normalized or removed.

| Feature | Walking coordinate | Native behavior |
|---|---|---|
| Upper guide | `(3,16)`, approach `(4,16)` | Original multi-page advice; dialogue owns input |
| Ice Heal | `(9,10)`, approach `(8,10)` | Original one-time item and flag |
| Upper first stop | `(15,16)` | Eastward slide meets the southern shoulder |
| Upper northern tooth | `(15,10)` | Northward travel stops before the rock tongue |
| Lower surveyor | `(3,20)`, approach `(4,20)` | Original winter/canoe conversation |
| Max Revive | `(5,5)`, dry approach `(6,5)` | Optional western supply branch |
| NeverMeltIce | `(19,3)`, approach `(19,4)` | Reward for the deeper turning course |
| Lower ice checkpoint | `(13,15)` | Resting ice stop remains stationary through Continue |

From the lower arrival `(5,19)`, a complete demonstrated source course toward the northern shelf is:

`RIGHT→(11,19), UP→(11,6), RIGHT→(13,6), DOWN→(13,15), RIGHT→(21,15), UP→(21,6), LEFT→(19,6), UP→(19,5), UP→(19,4)`.

The first seven commands are forced slides; the final two step onto the dry shelf. A directional input during an active slide cannot steer sideways. The native controller tests prove this independently of the source planner. The full source courses, pickup approaches and modeled stop states are retained with each preview under `renders/ice-turning-v458/source-loop/`.

## Authoring and fast review

- `before.json` is the frozen former source; `after.json` is the reviewed publication input.
- `contract.json` binds both complete native modules, including all installed actor overlays and programs.
- `current.json` and the canonical `ice-caves/voyageurs-ice.json` identify the selected source after integration; their geometry is identical to the tested input.
- `plan.json` records the role of each floor, route and reward, the exact paired connections, verified evidence and fixture limits.
- `sources.json` inherits the complete previous source index and adds the current ice-dungeon binding.

From this checkout's existing Pillow/CBOR environment:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_ice_turns.py \
  --pack content-packs/minnesota-ice-turning-chambers-v01.crystalpack \
  --source planning/minnesota/ice-caves/turning-chambers-v01/current.json \
  --output planning/minnesota/renders/ice-turns-live --watch
```

Measured warm review of both rooms took 0.107 seconds. The watcher loads native art once and preserves each source, both full-resolution rooms, their 2× versions and the whole-dungeon board. All initial and rejected passes stay retained. The first lower layout allowed a straight eastern shortcut; offset northern/southern tongues now require the intended turns. The final native views were inspected against original Ice Path rooms, including the broad ice areas as well as the stops and reward shelves.

`revise_voyageurs_ice_turns` is the Rust publisher, taking `PACK SHA256 BEFORE AFTER CONTRACT FRESH_OUTPUT`. `build-results-v460.json` and `publication-ready-v462.json` under the render folder retain the executed build/publication commands and exact hashes. It changes only the two terrain block arrays and their uniquely owned encoded payloads, then derives a new content identity and performs strict readback. It rejects stale modules, changed retained source fields, open borders, unsafe slides and pickup-dependent traps. Existing actors and story programs are cloned from the installed pack rather than regenerated from an older source catalog. A later edit must bind its actual current parent and frozen current source; never select an older pack to bypass a guard.

## Verified scope

Both controller cases pass with original Minnesota partners: a complete exterior/upper/lower journey, forced slide direction, both retained conversations, all three rewards, pickup persistence, an ice-stop Continue, paired ladders and the same outside return; full-pocket denial preserves the ball/flag, native Toss frees a slot, retry collects once, and a partial-pickup state returns safely. The stdio-MCP course records 482 RPC calls with empty stderr and unchanged inputs, including a fresh-process ice-stop Continue. Its source-frame advancement uses ordinary bounded Game Boy B inputs, without changing game state directly.

All nine complete headless native frames match the source at the same position, facing and palette, with one coherent native ice-animation clock per frame. The independent whole-pack audit preserves all 730 other maps, every program/event/actor/reward, attributes, identities, source art, audio and unrelated catalogs. Only `data.maps`, `data.map_blocks` and the derived identity change. The source and native collision model also prove returns for all six local pickup-body combinations and no dry bypass to the main ice destinations. These graph checks are separate from the actual controller/MCP cases and visual review.

Affected-crate all-target Clippy and owned Rust formatting pass. Full workspace formatting retains unrelated differences beginning at `offset-allocator/src/lib.rs:408`. Audio verification checks all 377 programs' PCM hashes, frame counts and loop ranges. The touched rooms' encounters/fishing, regional trainer references and active evolutions pass the original Minnesota roster policy audit; unrelated inherited old LAKE/SHORE references on other maps remain outside this delta.

The initial partners, eight badges/lighthouse progress, positions, Repel and bag-capacity setup are declared local QA fixtures. They do not prove earned endgame access or campaign completion. The broader map and cave improvement goal remains active.
