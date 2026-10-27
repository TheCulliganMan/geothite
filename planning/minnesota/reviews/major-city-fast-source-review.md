# Fast major-city source review

`planning/minnesota/tools/review_major_city.py` renders explicit city layouts and native contract rooms. It generates no terrain, writes no pack, and proves no gameplay or GPU behavior. Every output is marked **SOURCE REVIEW ONLY**. Root retains native publication and controller acceptance ownership.

## One command after editing

Run from the repository root. Use the original pre-expansion pack required by the contract preimage, rather than an already expanded city pack. This executed Minneapolis command uses the frozen accepted source; canonical layout/contract paths can replace those two arguments after their guarded promotion.

```sh
/Users/ryanculligan/.local/bin/uv run --offline --with cbor2 --with pillow python planning/minnesota/tools/review_major_city.py \
  --base content-packs/minnesota-northstar-itasca-quality.crystalpack \
  --layout planning/minnesota/renders/minneapolis-expansion/tight64-v02/MnMinneapolis.layout.candidate.json \
  --contract planning/minnesota/renders/minneapolis-expansion/tight64-v02/expansion.contract.json \
  --views planning/minnesota/renders/minneapolis-expansion/tight64-v01/native-views.json \
  --output planning/minnesota/renders/cities/fast-source-review/minneapolis-v02-exterior-final \
  --exterior-only --warm-rerenders 1
```

The recorded output directory already exists; choose a fresh output name for each invocation. Existing output directories are rejected. The offline uv invocation uses the locally cached Pillow/cbor2 dependencies; an environment already providing those packages can invoke `python` directly.

Omit `--exterior-only` for the default whole city plus every new contract room. Street-only mode skips all new room rendering and room checks; the receipt explicitly records that restricted scope. Optional views are a JSON array of `{"id":"name","map":"NativeMapId","core":[x,y]}`. Street-only views must target the city. Without views, the tool renders the declared city arrival. Original rooms are retained in the pack, but the fast tool renders only the new rooms listed in the contract.

`--warm-rerenders 1` retains a second review in the same process, sharing loaded pack/block/sprite caches. Both runs use the same captured input bytes. This measures a cached rerender; it does not watch files or reload edits. Separate invocations load the pack again.

## Fidelity and scope

The source review requires the exact whole original-city module hash and explicit matching dimensions/tileset/ID. It uses the existing hand-authored `layout_blocks`, `SourceArt`, native imported-block recipes, relocated original native actors (including indexed overrides), and new authored residents/trainers/pickups. New owned item/trainer flags start unset; scripts, progression, object movement and conditional hiding are not simulated. Moving original sprites are displayed as stationary south-facing source previews, with their native movement recorded. Nondefault original NPC palette overrides are rejected rather than approximated.

New room palettes come from each real native template and `effective_period`; NPC palettes follow the same effective bank. Native original/imported glyph foreground masks are applied with existing `priority_scene`, preserving source glyph-index masks even under palette overrides. Source sprites use the current native `(core_x*16, core_y*16)` origin. Cameras are actual 160×144 source pixels and nearest-neighbor 4× copies. There is no fabricated tile catalog or independent renderer.

Checks cover initial actor standing Floor, exterior Door/return Floor, new room unoccupied arrival Floor, declared warp collision, and unoccupied Floor camera anchors. These checks do **not** replace actor-present navigation, script/service checks, native pack publication, controller tests, or actual GPU comparison. Unsupported recipe row rearrangements fail explicitly.

Every whole terrain/actor/composite stage, imported block original, sprite, source camera/crop/enlargement, room/gallery and archive original is retained through existing render archive/stage helpers. Input JSON bytes are captured before rendering and saved exactly; receipts hash the base pack, inputs and tool, report whether inputs changed during the run, and separate pack-load time from render/check/archive time. No output or failed stage is discarded.

## Executed verification

Measurements below include source rasterization/checks, image/archive writes, room/camera sheets and exact source-block originals. They are local wall times and vary with system contention. Pack loading is measured separately.

| Review | New rooms | Pack load | Initial render | Warm render | Static checks |
| --- | ---: | ---: | ---: | ---: | --- |
| minneapolis-v02-final | 39 | 11.922364 s | 13.447923 s | 9.833556 s | PASS |
| minneapolis-v02-exterior-final (eight cameras) | 0 | 10.218832 s | 11.60202 s | 9.673289 s | PASS |
| minneapolis-v02-one-street-final (one camera) | 0 | 7.227924 s | 2.121734 s | 2.113587 s | PASS |
| duluth-v09-final02 | 41 | 3.527426 s | 5.113446 s | — | PASS |
| st-paul-v10-final02 | 37 | 3.594985 s | 5.028661 s | — | PASS |

Pre-watch tool SHA-256: `6055a6341c5796e7daf07277b926a295dd4ed65b268574399febe1ef0b608546`. Minneapolis all-room timing predates only the optional exterior-only switch and optional facade label correction; all other rows use this final tool.

- planning/minnesota/renders/cities/fast-source-review/minneapolis-v02-final/receipt-019f7b7e4d2dfa087c58319a53b47318d9d0a5bb7d784708c41ceb84444f7fa4.json
- planning/minnesota/renders/cities/fast-source-review/minneapolis-v02-exterior-final/receipt-cf51c0780a77f7f16fa8367016c9ab114ee8ef9d1231cc03181649d810e3f49a.json
- planning/minnesota/renders/cities/fast-source-review/duluth-v09-final02/receipt-c84aa196a022eda20277ed1273ed5a8e0bbc7d201c410c2acc874ed020958d76.json
- planning/minnesota/renders/cities/fast-source-review/st-paul-v10-final02/receipt-a0afcffc585c650a220c31446c570dd6f1bd48aec9fcba942a914c6731b58ec4.json

All four completed reviews have zero static review errors and unchanged input hashes. All-mode Minneapolis renders39 new rooms, Duluth41 and StPaul37; eight explicit exterior cameras each. The exterior-only run renders zero rooms/checks and the same eight Minneapolis camera anchors. Personally inspected the Minneapolis source camera sheet, Duluth camera sheet and StPaul first new-room gallery. Prior interrupted optional-door-ID adapter runs remain retained at `duluth-v09-final` and `st-paul-v10-final`, with their `.local/review-major-city-*-final.log` failure logs. The generic tool now uses the target/warp index when an optional door ID is absent.

The measured focused street loop uses the same command with `--views planning/minnesota/renders/cities/fast-source-review/inputs/minneapolis-one-street-view.json` and fresh output `planning/minnesota/renders/cities/fast-source-review/minneapolis-v02-one-street-final`. It retains the full exterior whole and one actual market camera at core87,50, while skipping all interiors. Receipt: `planning/minnesota/renders/cities/fast-source-review/minneapolis-v02-one-street-final/receipt-bced58be8046d0597c93ce9243eb129cbe3344cad4ecc6f6ec1887e236c1bfbe.json`. Static checks PASS, input hashes unchanged, warm2.114seconds; pack loading7.228seconds separately. The same-process cached repeat avoids a second pack load. Python AST parsing also passes.

## Actual edit-and-rerender watch loop

Add `--watch` to reload the current layout, contract and optional views whenever their content hashes change. The native pack, SourceArt/block caches and sprite caches remain loaded. This mode is independent of `--warm-rerenders`; combining them is rejected. `--max-renders 2` bounds a smoke run to two attempted input revisions. Omit that bound for an editing session and stop with Ctrl-C.

Each input set must be stable across250ms before capture. Every attempt gets a fresh `revision-NNN-HASH` directory, exact captured JSON and tool snapshots, original image/stage archives and its own content-hashed receipt. A parse/render failure retains its snapshots/error and the watcher can process the next edit. A change during rendering marks that revision `inputs_unchanged=false`; the next revision reloads the new bytes. The rendered images always correspond to that revision's captured bytes, never to a retrospectively copied current file. Existing revisions are never overwritten. A changed base pack requires restarting the watcher; it does not silently reuse different native art.

Example with the retained temporary smoke copies (choose a new output directory before executing):

```sh
/Users/ryanculligan/.local/bin/uv run --offline --with cbor2 --with pillow python planning/minnesota/tools/review_major_city.py \
  --base content-packs/minnesota-northstar-itasca-quality.crystalpack \
  --layout planning/minnesota/renders/cities/fast-source-review/watch-edit-smoke-02/temporary-input-copies/layout.json \
  --contract planning/minnesota/renders/cities/fast-source-review/watch-edit-smoke-02/temporary-input-copies/contract.json \
  --views planning/minnesota/renders/cities/fast-source-review/watch-edit-smoke-02/temporary-input-copies/views.json \
  --output planning/minnesota/renders/cities/fast-source-review/new-editing-session \
  --exterior-only --watch
```

Edit those input files while the command waits. Substitute your own current authoring inputs to review another city. Default all-room behavior and the fixed-input warm benchmark remain available without `--watch`.

Bounded smoke proof: `renders/cities/fast-source-review/watch-edit-smoke-02/proof-8fe4d0665ad85090c18ee9037c890d23aa03081687fa4b9fa38b71f1e9b8a3a1.json`. The driver ran two revisions and exited0. It moved only temporary-copy resident `MN_MP_MARKET_BERRY` from89,49 to90,49; both native Floor checks pass. It deliberately saved during revision0: that revision retained old actor89,49 and reported input instability; cached revision1 retained new actor90,49 and reported stable input. Exactly356 source160 pixels changed, all inside the two sprite footprints (bounds113,56–143,72). Both actual source4x frames were personally viewed. Base pack and all frozen original inputs remained hash-exact.

Revision0 rendering took0.588s; after-edit cached rendering took0.530s. The new receipt arrived1.076s after the edit, including completion of the old revision, polling/stability delay, second render and verification. Local contention differs from earlier timings; this is a measured actual file edit, not extrapolation from a fixed-input repeat. All buffers/snapshots/temporary input copies and processes' exit receipts are retained; the private watcher process exited and no background watch job remains.

The first smoke's render/watch behavior also worked, but its independent verifier had an8px camera-origin assertion error. Its driver, logs and both revision archives remain at `watch-edit-smoke-01`; the corrected second smoke derives footprint bounds from the actual source camera crop. No source or renderer was altered to fit that verifier.

Default all-room CLI rechecked after the watch addition: `renders/cities/fast-source-review/minneapolis-v02-default-after-watch/receipt-8df856be77287f7e8d08ea2132063e9f31546f87c62302cec7b53d26241ca86e.json`;39rooms/one camera, staticPASS, input hashes unchanged,0.831s render and1.450s pack load.

Current watch-capable tool SHA-256: `c0cfcd8227ad025609a703fce6e7d44d1eb03aaed506859e4828e2e1754de0d2`.
