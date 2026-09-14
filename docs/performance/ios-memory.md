# Mobile 2.5D memory regression

Switching the deployed browser game to 2.5D expanded identical scenery into
separate vertex arrays. In mobile WebKit, a real New Bark Town save grew from
274,792,448 bytes of WASM memory to 3,125,936,128 bytes on the first switch.
The largest WebGL buffer was 440,779,780 bytes. WASM retained that high-water
allocation after returning to 2D. This resource spike is consistent with the
reported iOS page-process termination; the physical iPhone Air was not available
for testing.

The runtime now shares exact painted scenery prototypes. The key includes the
silhouette, all source RGBA pixels, dimensions, scale, depth, and rounded shape.
Each placement retains its translation and ground surface. The distant repeating
ground uses four strips and an 8×8 nearest-sampled repeating source texture.
Scenery coverage, source resolution, collision, and simulation are unchanged.
Replaced instance hierarchies are removed instead of retaining their mesh handles.

## Reproducible assertions

`tools/browser-voxel-memory.mjs` loads an ordinary in-game save, uses the actual
view controls for six switches, rotates eight times, zooms, holds the mobile
D-pad, and changes to landscape. The route fixture must cross into Route 29.
It fails on a page error, context loss, failed movement, incorrect view state,
WASM high-water memory above 512 MiB, or any WebGL buffer above 32 MiB. These
are regression-test budgets, not allocations or verification in the game loop.

```sh
npm ci
npx playwright install chromium webkit
node tools/browser-voxel-memory.mjs 'http://localhost:8080/?multiplayer=off' memory.json webkit
node tools/browser-voxel-memory.mjs 'http://localhost:8080/?multiplayer=off' route.json webkit route
node tools/browser-voxel-memory.mjs 'http://localhost:8080/?multiplayer=off' chromium.json chromium
```

The voxel unit suite runs in GitHub Actions on every main push and PR. It
compares expanded and shared geometry positions, normals, shading, and sampled
colors; refuses sharing different paint; checks hierarchy replacement; verifies
repeated-ground area and pixel coordinates; and enforces a 2 MiB geometry budget
for all 512 trees in a large synthetic scene. All 440 tests passed locally.
The combined release also passed 62 browser tests, six nickname tests, and the
locked production Docker build (native server plus release WASM).

## Measurements

The fixed combined production image was tested through a private SSH tunnel.
Mobile WebKit used a 430×932 viewport at DPR 3 and then landscape. These are Mac
WebKit measurements, not an iPhone hardware certification. WASM bytes do not
include the entire browser process or GPU residency; uploaded bytes are cumulative
traffic, not retained memory. Allocator peaks vary slightly between runs.

| Resource | Before | Fixed |
| --- | ---: | ---: |
| New Bark WASM high-water | 2,981 MiB | 386 MiB |
| Largest New Bark GPU buffer | 420 MiB | 15.1 MiB |
| Route traversal WASM high-water | — | 383 MiB |

Repeated switches did not increase the fixed heap high-water mark. Both WebKit
and Chromium passed the mobile switch/walking budget checks.

The real-input walking benchmark used headful Chromium at 1280×900 with the same
saved game and pack. The baseline was the live pre-memory-fix release; the
candidate also includes concurrent mobile UI and nickname fixes. These short
single-run comparisons are evidence of improvement, not a claim of fixed FPS
on every device. Browser work was run sequentially for the candidate and route
comparisons; the walking baseline briefly overlapped the end of a separate
WebKit diagnostic, so its timing should be treated as indicative.

| 2.5D scenario | Median ms before → after | p95 ms before → after | Worst ms before → after |
| --- | ---: | ---: | ---: |
| Walking | 33.2 → 16.7 | 50.4 → 33.9 | 83.4 → 67.3 |
| Route transition | 33.3 → 16.7 | 83.5 → 50.0 | 1,299.9 → 466.6 |

Walking retained all composed GPU textures. Map changes still allocate map
assets and can hitch; this change does not claim stall-free transitions.
Raw measurements are in [ios-memory-results.json](ios-memory-results.json).
