# Capture actual reached winter states

Root executes `../capture_reached_winter_saves.py` only after the native renderer
and the three main-region controller cases are ready. It has not been run yet.
It performs no Cargo work, generates no pack and never decodes/modifies a Save.

Required arguments:

| Argument | Input |
|---|---|
| `--pack` | Actual final native pack; currently `content-packs/minnesota-northern-winter-v01.crystalpack`, SHAaa1aa251… |
| `--binary` | Root's newly built `render_at_location` example, with the Save-clock fix below. |
| `--saves` | Fresh `CRYSTAL_WINTER_REGION_SAVE_DIR` containing exactly the five real controller Saves. |
| `--controller-log` | Fresh passing pretty harness log containing all three exact `winter_region_controller_*` results. |
| `--output` | Fresh ignored native review directory; it must not be inside the input Save directory. |

The log must contain one actual PASS result for each exact main-region test,
a clean zero-failure summary and no failed run. Both ordinary single-line Rust
results and sequential `--nocapture --test-threads=1` results are understood.
The latter can place native battle diagnostics between the harness test name
and its terminal `ok`. Ambiguous interleaved or truncated results are rejected.
A summary count by itself does not prove the three required cases passed.

The exact five accepted filename prefixes are:

- `winter-lower-seventeen-stop-`
- `winter-hut-after-heal-`
- `winter-cache-after-pocket-pages-`
- `winter-stationary-ice-wall-stop-`
- `winter-after-both-native-trainers-`

Each must continue with the actual numeric process ID/nanosecond suffix and
`.crystalsave`. Missing/duplicate/extra Save files are rejected; use a fresh
controller output epoch after a failed attempt. These five Saved scenes retain
real slides, healing, the one-time item/Pocket pages and both trainer victories.
Their initial local Mew80/SURF/Fog/badges/RNG/inventory fixtures remain explicit;
they do not prove earned campaign progression.

The tool copies/hashes one private renderer for the batch, then for each Save
executes exactly:
`--pack ACTUAL_PACK --save ACTUAL_SAVE --view 2d --screenshot NEW_PNG`.
Those names describe its generated argument list, not runnable placeholder
paths. The actual absolute paths, executable/file hashes and complete commands
are recorded in the receipt. No map, tile, hour, party, badge, flag, preview,
walk or live-mode argument is passed. Headless image capture is fixed640×576;
no window or application focus is requested.

**Clock prerequisite:** Root corrected `render_at_location` so
`render_test_hour` is set only when `args.save.is_none()`. Its previous default
would force noon even when the command omitted `--hour`. The refreshed example
now uses the ordinary production Save/Continue/system-local RTC path for Saves.
That clock can advance naturally. This tool claims no synthetic noon override
and no frozen exact Saved second; it must use the rebuilt binary, not an older
copy. Fixed-coordinate source review remains a separate hour12 fixture.

The output retains:

- Original controller log and opaque byte-identical copies of all five Saves.
- Their hashes, chosen pack/renderer/log hashes and all native capture logs.
- Untouched original production PNGs plus every raw GPU descriptor/buffer and
  its size/hash; actual GPU raw filenames must match their content hashes.
- Separate RGB review conversions, stage/history archives and contact sheets.
- Content-addressed partial/final receipts and a snapshot of the capture tool.

The tool verifies all input Save/pack/log hashes and retained Save copies again
at the end, and removes only its own uniquely named private executable. Failed
captures keep their partial outputs/receipts. The final success status says
`NATIVE_REACHED_SAVES_CAPTURED_REVIEW_PENDING`: it establishes actual capture,
not source-pixel agreement or personal visual acceptance. Root still inspects
all five frames and records that review separately.

Static verification: Python syntax parses. The isolated in-memory log parser
accepts both normal and sequential-nocapture passing formats and rejects six
invalid cases (missing/ignored/duplicate/failing/truncated/interleaved results).
No capture CLI, native binary or actual Save was executed for these checks.
