# Deploy Geothite multiplayer

The live browser game serves [The Northstar Survey](https://geothite.ryanculligan.com/?view=2d)
in native 2D. See the [2026-11-17 release record](#minnesota-refinement-release-2026-11-17)
for the exact pack, image, verification scope and rollback command.

## Requirements

Use Docker Engine with BuildKit and the Docker Compose plugin. Run commands
from the repository root. The Dockerfile includes the Rust and WASM toolchains;
no host Rust installation is required for the Docker build. Install Git LFS and
run `git lfs pull` before building so the tracked 3D artwork contains its actual
payloads. Build from that hydrated local checkout; a remote Git build context
does not hydrate LFS artwork. Supply the browser
pack first using [Game content setup](game-content.md). The initial build
compiles the game and takes substantially longer than a cached rebuild.

## Start the server

Create an ignored `.env` file containing a stable random signing secret:

```sh
umask 077
printf 'CRYSTAL_AUTH_SECRET=%s\n' "$(openssl rand -hex 32)" > .env
```

Run this once for a new deployment. Preserve the secret during updates;
replacing it invalidates existing browser identity tokens. If `.env` already
exists, add the variable without overwriting the other settings.

```sh
docker compose -f docker-compose.production.yml up -d --build
docker compose -f docker-compose.production.yml ps
curl --fail http://localhost:3003/healthz
curl --fail http://localhost:3003/v1/clock
```

Open `http://localhost:3003` for local play. The always-visible **2D / 3D / TUI** chooser swaps renderers in the same
running game, preserving position, dialogue, menus, battles, and saves. Direct
links use `?view=2d`, `?view=3d`, or `?view=tui`; `/tui` and `/tui/` select the
same terminal renderer. See [web renderer views](tui.md). Remote browser play requires HTTPS
for the per-player Web Lock. The browser obtains its identity automatically
from `POST /v1/session`; no manually issued token or account is required.

The image contains the server, WASM client, browser assets, and your locally supplied
browser pack. The repository does not distribute the game pack. The embedded modpack JSON is included in the repository.
Generated pack data should be regenerated through the canonical exporter when
content changes, not edited by hand.

## Configuration

The image also serves the Rust/WASM text client at `/tui` and `/tui/`, using
the same hosted game pack. See [Text UI](tui.md) for build, controls, isolated
browser saves, WebMCP and real-game browser verification. Keep the entire
content-addressed TUI bundle together when deploying; do not mix its glue and
WASM from different builds.

| Variable | Default | Purpose |
| --- | --- | --- |
| `CRYSTAL_AUTH_SECRET` | Required | Stable signing secret, at least 32 bytes. |
| `POKECRYSTAL_PORT` | `3003` | Published host port; the container listens on 8080. |
| `CRYSTAL_MAX_CLIENTS` | `20000` | Maximum concurrent client connections. |
| `CRYSTAL_MODPACKS` | Empty | Optional additional accepted pack identities. |
| `RUST_LOG` | Server and HTTP info logs | Runtime log filter. |

The browser server adds its generated multiplayer pack identity automatically.
The Compose file stores server data at `/srv/crystal/data` in a named volume.

## HTTPS and WebSockets

Point a hostname at your reverse proxy and forward traffic to the published
HTTP port. For example, Caddy running on the same host can use:

```caddyfile
game.example.com {
    encode gzip
    reverse_proxy 127.0.0.1:3003
}
```

Replace the example hostname with your own. The proxy must support WebSocket
upgrades on `/v1/ws` and an idle timeout longer than 45 seconds. Caddy handles
WebSocket upgrades automatically. Configure DNS and certificate issuance for
your environment. Container-to-container proxies can instead use service
`pokecrystal-multiplayer` on port 8080 when they share a Docker network.

## Update

```sh
git pull --ff-only
docker compose -f docker-compose.production.yml up -d --build
docker compose -f docker-compose.production.yml ps
```

Rebuilding compiles the checked-out revision and recreates the service when
its image changes. Plain Compose up against a local checkout does not fetch
Git changes. For a stack deploying a feature branch, check out and hydrate that
branch locally, then point its build context at that checkout. Pin the image to the
verified commit and keep the previous image for rollback.
Do not use `--no-build` when you need a fresh build.

BuildKit caches dependencies and compiled targets. The build uses two Cargo
jobs by default; override with `docker compose -f docker-compose.production.yml
build --build-arg CARGO_BUILD_JOBS=4` when resources allow. The browser build
uses the `web-release` profile to limit linker memory use.

## Keep terminal downloads available

The Dockerfile explicitly includes `web-client/install.sh`. Release publishing
also requires a native executable for each supported platform and a manifest:

```sh
docker buildx build --build-arg CARGO_BUILD_JOBS=1 \
  -f tools/terminal-build.Dockerfile \
  --output type=local,dest=.local/terminal-linux .
sh tools/terminal-release.sh RELEASE .local/terminal-distribution \
  content-packs/realtime-clock.browser.crystalpack \
  aarch64-apple-darwin=target/release/geothite \
  x86_64-unknown-linux-gnu=.local/terminal-linux/geothite
```

Build the macOS executable on that host with `cargo build --release --locked
-p geothite`. Hash the actual pack served by `/realtime-clock.browser.crystalpack`
when it differs from the input pack. Publish binaries and launcher before
atomically replacing `downloads/terminal/current.txt`; retain previous releases.
Generated downloads and game packs remain outside tracked source.

The terminal manifest may include `pack-path` with a relative, versioned pack
URL. This pins an existing terminal release to its verified content when the
browser receives a newer story pack. Without that field, the installer uses
`realtime-clock.browser.crystalpack` as before.

The homelab deployment keeps these files in
`/mnt/homelab/config/geothite/terminal-distribution`, with read-only service mounts:

```yaml
- /mnt/homelab/config/geothite/terminal-distribution/install.sh:/srv/crystal/web/install.sh:ro
- /mnt/homelab/config/geothite/terminal-distribution/downloads:/srv/crystal/web/downloads:ro
```

Preserve both mounts when replacing the web image. They keep the curl installer
and native downloads available across browser releases. Verify the actual curl
pipeline from a fresh directory on each published platform:

```sh
node --test tools/terminal-install.test.mjs
python3 tools/terminal-curl-smoke.py https://geothite.ryanculligan.com
```

The process check enters fullscreen play, opens Start, saves, reinstalls without
changing that save, then loads it and drives authoritative movement and Start
through MCP. The controller also has an external-pack regression for bedroom
Continue: completed decoration callbacks must release input before movement.
The macOS binary enables crossterm's portable `/dev/tty` poller because the
installer reopens terminal input after reading its script from curl's pipe.

## Persistence and cleanup

Multiplayer ratings persist in the `crystal_multiplayer_data` volume. Game saves
remain in each player's browser. Live connections, chat history, queues, and
in-progress matches are held in memory and do not survive a server restart.
Returning players retain their browser credentials when the signing secret
stays unchanged. The server loads persisted ratings at startup and writes
ratings atomically to `ratings.json` in its data directory.

Back up server data and preserve the signing
secret before migrating. Routine Compose updates keep the data volume.
Do not use `docker compose down -v` unless you intend to delete server data.

Container logs rotate at 10 MB per file with three files retained. Build cache
can be cleaned manually or by your scheduler:

```sh
docker image prune --force --filter until=168h
docker buildx prune --force --filter until=168h --max-used-space 10GB
```

These commands affect the selected Docker daemon/builder, including cache from
other projects. They prune dangling images and unused cache older than seven
days without deleting volumes. Recent or active cache can exceed the 10 GB
target. Keep useful cache to avoid recompiling the game on every update.

## Diagnose startup problems

```sh
docker compose -f docker-compose.production.yml logs --tail=100 pokecrystal-multiplayer
docker compose -f docker-compose.production.yml exec pokecrystal-multiplayer \
  /usr/local/bin/crystal-web-server --healthcheck 127.0.0.1:8080
```

If a build fails, fix the reported error before treating the update as complete.
Check container health and the public HTTPS endpoint after deployment. A healthy
backend alone does not verify DNS or reverse-proxy configuration.

### Validate browser releases through the public URL

The Docker build gives each generated JavaScript/WASM pair a content-derived
filename and rewrites the page to import that pair. Do not publish generated
files under a reused URL: an intermediary cache can retain the old JavaScript
while fetching new WASM, causing a missing function-import startup failure.
For manually assembled browser distributions, run
`sh tools/version-browser-bundle.sh /path/to/web` after copying the page and
wasm-bindgen output.

Before rollout, run `npm ci`, `npm run test:browser`, and install the test
browsers with `npx playwright install chromium webkit`. Validate the candidate
and then repeat against the public HTTPS URL:

```sh
BROWSER_HEADLESS=1 node tools/browser-deployment-smoke.mjs \
  'https://your-game.example/?multiplayer=off' \
  /path/to/previous-release/crystal-bevy.js /tmp/deployment-proof.json
```

The check reaches the New Game menu in Chromium and mobile WebKit with both
fresh assets and an intentionally stale unversioned JavaScript response. It
fails on startup errors or missing game state and records screenshots. This is
WebKit with an iPhone viewport, not a physical-device Safari test. A container
healthcheck or a localhost-only check does not validate public cache behavior.

## Verified homelab release, 2026-10-25

Deployed `geothite:9f40b434-battle-latency-20261025t170024z-speed` to
`sshpop:/mnt/homelab`. The image ID is
`sha256:0cd6af0aa0905a950704cd06b7b0820e81bd63bf51afe174da43c913eb88dc40`.
It includes working-tree changes over `9f40b434`; its recorded compile-input
tree digest is `2ff587eb185d7dfe2435bee7eaa612c6813c491ef45e577a1bdfffe68bf9d678`.
The release directory retains source, build recipe, previous service configuration
and guarded rollback helper. Only the Geothite service was recreated; its signing
configuration, data volume and Flygon data mount were preserved.

Public HTTPS checks passed health, exact WASM release, fresh/stale startup in
Chromium and mobile WebKit, nine renderer swaps, Start and Mom's dialogue/input
ownership. Real curl installs passed on macOS ARM64 and Linux x86_64, including
Continue, save preservation and authoritative MCP movement/Start. Installer and
downloads are served through the persistent mounts above. The served pack SHA-256
remains `a625e367ee7000e6968cf017264d5178c237887c5a63e1d9fc6883b7f079007c`.
See [battle latency measurements and existing limitations](runtime-input-performance.md#battle-search-frame-latency).

The macOS app was built and installed with:

```sh
python3 tools/desktop.py build --target-dir target/runtime-fps --profile release --jobs 4
python3 tools/desktop.py install --target-dir target/runtime-fps --profile release --skip-build
```

`~/Applications/Geothite.app` passed signature verification and
`--check-installation` from outside the checkout. Its external save location is
unchanged.

### Mobile controls and intro follow-up, 2026-10-25

The follow-up image is
`geothite:9f40b434-mobile-intro-20261025t225948z-final`, ID
`sha256:4744c4246393b1e1db24c4f66c5b8937ced83f8ce7b74c3acad57159dc201dc1`.
It restores the actual touch buttons, reserves space for them in both orientations,
and resolves intro OBJ tiles through the pack's shared BG/OBJ VRAM bindings.
No content pack, battle engine, signing configuration or persistent mount changed.
The updated Mac app passed signature verification and `--check-installation`
from `/tmp`, retaining the same external save path. Public curl installs passed
on both macOS ARM64 and Linux x86_64 after rollout.

The headless mobile regression uses trusted touches for gameplay; MCP only
observes. It watches the complete movie, starts CHRIS, moves the player, selects
and closes the production menu, checks small portrait/landscape layouts in all
three views, and checks multi-touch cancellation in Chromium:

```sh
node tools/browser-mobile-smoke.mjs https://geothite.ryanculligan.com /tmp/mobile-proof.json
```

Both public browser runs passed with no page or console errors, including a 3D
start in mobile WebKit and a 2D start in Chromium. WebKit uses an iPhone viewport;
these are browser-engine tests, not a physical iPhone run.

The full-frame intro regression and ROM pixel checkpoints pass with the external
TUI pack. Comparing the same 92 intro tests with the earlier native test binary
shows 42 failures before and 40 after; the two fixed tests are the full-frame
replay and ROM pixel checkpoints, with no new failures. The remaining failures
use older source fixtures or setup assumptions. Workspace formatting and
all-target Clippy still report existing failures; production-library Clippy passes
with the existing `bad_bit_mask` and `never_loop` lints allowed.

### Region preloading and 3D transitions, 2026-10-26

Deployed `geothite:9f40b434-region-preload-20261026t090056z-final`, image ID
`sha256:e0958be88074ead5e01cf2dec9401bea3799042f1c86f2d241bb22274e666249`.
The frozen working-tree source over `9f40b434` has compile-input digest
`5d2103fa718663ee062c3e2e4727369bf15a2abfcadeb17abb6c44114afa5d5c`.
The release is retained under
`/mnt/homelab/config/geothite/releases/9f40b434-region-preload-20261026t090056z`
on `100.101.29.133`, including source, build recipes, verification summaries,
and the previous service configuration. Only Geothite was recreated; signing
configuration, persistent data, Flygon data and terminal download mounts remain
unchanged. The served content pack retains the SHA-256 recorded above.

Connected-region preloading now follows the complete visible source window,
including chains beyond four small regions, and retains terrain and actors
through camera turns. The release also includes prepared 3D route/battle
transitions and modeled battle trainers. Both WebGL and WebGPU ship with terrain
and actor workers. Their normal scenery detail selection remains enabled, and
the bounded paged worker protocol accepts the larger preloaded scene. Existing
immutable browser assets are retained for tabs using the previous release.

Before rollout, headless production-candidate checks passed 33 camera positions
at the Route 29/New Bark boundary on each renderer, with no terrain rebuild,
pending terrain, additional worker submissions, worker failures or fallback.
A production Falkner battle displayed both modeled trainers and reached the
battle menu with modeled creatures. Fresh/stale startup checks passed in
Chromium and mobile WebKit. Local controller and browser checks also crossed
connected routes in both directions and preserved state when switching to 2D.

After rollout, the public HTTPS endpoint passed health and clock checks, exact
JavaScript/WASM hashes for both renderers, the unchanged pack hash, the same
33-position WebGPU boundary sweep and a modeled Falkner battle. The installer,
terminal release pointer and Flygon routes remain available. The preceding
release's immutable JavaScript URL also remains available.
All four public fresh/stale startup runs reached New Game without page errors
in Chromium and mobile WebKit. These checks ran headlessly with
`BROWSER_HEADLESS=1 node tools/browser-deployment-smoke.mjs`, using the public
game URL, the preceding release's JavaScript and a temporary JSON output path.

The release's guarded deployment commands are:

```sh
release=/mnt/homelab/config/geothite/releases/9f40b434-region-preload-20261026t090056z
python3 "$release/deploy.py" rollout
# Restore the captured previous service only if its configuration still matches:
python3 "$release/deploy.py" rollback
```

Targeted Rust streaming/controller tests, worker paging tests, browser unit tests
and bundle-versioning tests passed. Workspace formatting and unrestricted
all-target Clippy still fail on existing code; scoped Clippy passes with the
existing `bad_bit_mask`, `approx_constant`, `never_loop` and `erasing_op` lints
allowed. This is not a claim that the entire repository test suite passes.

## State Fair normal-speed release (2026-10-27)

The game service now uses `geothite:19688108-state-fair-realtime-20261027-final`,
built from committed source `19688108f2652d05a25f090aa7b7f976fac3dc66`.
Its image ID is
`sha256:fdff078cfd764e8f6625e23e623a61cbffd7ad577a07ea54fd0c2d702cd2a633`.
The build used one Cargo job and the existing external browser content pack.
Save data, signing configuration, downloads, Flygon data and the Pokedex mount
were preserved. Previous immutable browser bundles remain in the image.

The frozen source, hydrated LFS inputs, build recipe and rollback configuration
are retained in
`/mnt/homelab/config/geothite/releases/19688108-state-fair-realtime-20261027`.
The guarded candidate passed fresh/stale startup in headless Chromium and mobile
WebKit; all four sessions reached the production New Game menu without page
errors. The rollout reported healthy, and public health and clock checks passed.
All four fresh/stale startup checks also passed against the public HTTPS URL,
using WebGL in Chromium and WebGPU in mobile WebKit. Both new renderer
JavaScript/WASM pairs and the preceding WebGL JavaScript returned HTTP 200;
the public bundle manifest exactly matched the tested candidate. Temporary
candidate containers and secret files were removed after rollout.

The build snapshot was frozen before later shared authoring edits. Those edits
and required source fixtures were also committed and pushed in `be0ac4da`;
they are not part of the deployed `19688108` image.

The independently published fair showcase is
`20261027-state-fair-realtime-22`, with media under
`/pokedex/media/showcases/20261027-state-fair-v2/`.
The 58-second, 30-fps capture uses the explicitly labeled offline game clock for
normal-speed playback. It is not a host rendering-performance measurement.
Desktop and mobile playback, chapter seeking, media byte equality and HTTP Range
requests passed before publication. The frozen 224-species catalog and its cries
were preserved. See [the Pokedex runbook](pokedex-web.md) for publication details.

Recorder tests (14), browser unit tests (129), continuous patrol movement,
controller interaction/save/Continue and frozen showcase publication passed.
Workspace formatting still reports pre-existing offset-allocator differences;
all-target Clippy remains blocked by the existing `bad_bit_mask` in
`crystal-core/src/multiplayer/mod.rs`. Changed capture Rust files passed scoped
formatting. This is not a claim that unrelated shared work passes every test.

## Northstar native 2D story release (2026-10-29)

Published `geothite:d7634794-northstar-2d-20261029t115843z-final` to
`sshpop:/mnt/homelab`. The final image ID is
`sha256:8cb42a892b3e559e3396f80f086a5da7d78921cf984c858a8a6b640b25d93ab2`.
The source is a frozen working tree over `d7634794`, with the recorded lobby
and installer patches. This is a browser publication; the regional world
selector and existing save identities were preserved.

The input is `content-packs/minnesota-dialogue-overhaul-v05.crystalpack`, built
on the completed Cedar warmth/R20 descendant. It contains 730 maps and retains
the current Northstar scenes, active Minnesota species and native art. Input
SHA-256: `a77997c3e19dd0d7d6f327d9dd5f0fb6b78d9f326260d5ecc51a6850a6ce2446`.
The server applies its existing hosted clock, battle and customization overlays;
the served runtime pack SHA-256 is
`80deedd0db1fcf7b994ee980e260a47f1e8bade0e346e4444b193020c5fd0dd7`.
Independent comparison preserves all 342 Minnesota maps, 78 Minnesota encounter
tables, the complete trainer catalog, native runtime files and audio sections.

The Rust browser client compiles without the voxel feature. Minnesota stays in
2D through old display preferences, `?view=3d`, F3 and renderer requests; the
visible choices are 2D and TUI. The renderer/audio pairs and TUI adapter are
versioned together. Earlier immutable browser bundles remain available.

Public default-online testing caught the story's 14,708-byte manifest lineage
exceeding the lobby's previous 4 KB bound. The first rollout was restored to the
preceding image while the fix was built. The final server accepts compositions
up to 16 KB, retaining the 128-byte component checks and existing WebSocket
frame bounds. A real authenticated welcome with the served story identity
passes, alongside 25 lobby tests. The required all-target server Clippy check
passes with existing warnings. New Rust lines match pinned rustfmt output;
workspace formatting still reports pre-existing differences beginning in
`crates/offset-allocator/src/lib.rs:408`.

The later 732-map Minnesota story/world integration has a 16,467-byte identity.
Current server source therefore permits a separately bounded 32 KiB composition,
retaining every manifest name and the existing 128-byte component checks. All
16 local Hub tests and affected-server all-target Clippy pass, including a real
Hub join with 700 unique chapter IDs above 16 KiB and aggregate oversize rejection.
Actual WebSocket testing of the new final pack and any hosted rollout remain
unverified; the historical authenticated welcome above covers its own earlier
14,708-byte pack only.

The final public HTTPS checks pass:

- Fresh and stale-glue startup in Chromium and mobile WebKit, with exact
  JavaScript/WASM and source/hosted pack hashes matching the tested candidate.
- Default-online Winona New Game in both engines, Mom's actual dialogue pages,
  rejected movement during dialogue, production Start, TUI rendering and return
  to the same game. Native 2D captures were inspected headlessly.
- Native stdio Mom/Tamarack pages, movement ownership, starter decline, Start
  and byte-empty stderr; all 377 original audio PCM hashes, frame counts and
  loop ranges; 133 browser/installer unit tests.
- Existing terminal curl installs on macOS ARM64 and Linux x86_64, fullscreen
  Start/save, save-preserving reinstall and authoritative MCP movement/Start.
- Healthy service and clock, unchanged Pokédex HTML, available Flygon route and
  retained preceding browser assets.

Native downloads remain pinned to their previous verified release and pack via
the manifest's `pack-path`; the new story is available in browser 2D and TUI.
Only Geothite was recreated. Its signing configuration, named data volume,
Pokédex, Flygon data and terminal mounts were preserved. Existing saves were
not relabeled for the new pack. An uninterrupted ordinary-party campaign,
complete difficulty acceptance and native/browser listening review remain open.

The frozen source, build recipes, final source patches, verification records
and private rollback configuration are retained under:

```text
/mnt/homelab/config/geothite/releases/d7634794-northstar-2d-20261029t115843z
```

The guarded rollback restores the captured service and terminal configuration,
provided no subsequent deployment has changed them:

```sh
python3 /mnt/homelab/config/geothite/releases/d7634794-northstar-2d-20261029t115843z/deploy.py rollback
```

## Combined Minnesota release (2026-10-29)

Published `geothite:combined-native-2d-itasca-20261029-final`, image
`sha256:a1e07c74178dd714461acba907ed21303f64682cba3ce26ea292c9867b4fe9dc`.
The combined working-tree source snapshot is retained on the homelab; shared
source checkpoint `3fe69ad0` also includes the concurrent source/art/test work.
The release retains the reviewed Marais/Mille geography and latest dialogue,
adds Verdigrim's completed species/field art, all 225 authored regional cries,
research-count synchronization and both published starter signatures. The
excavation/containment scene and optional capture remain unfinished. The late accepted Lake Itasca grass/ledge revision is included. This publication
does not move the shared authoring selector; Safari admission stays closed.

Input: `content-packs/combined-native-2d-itasca-20261029.crystalpack`, 730 maps,
476 total species, 272 moves and 602 audio programs. Input SHA-256:
`a69580bac73e664e263083fe490cdf057ee73cc6e21bf50a415c0e99c6ba9c9a`.
The hosted clock/customization pack SHA-256 is
`0d6a1206d107f46c0766b440f145f0f58f02643437045a24cd4ebb153449265a`.
Its 342 Minnesota maps, complete trainer catalog, 7,818 runtime files and all
602 audio programs are identical to the checked input. All original 377 audio
programs remain exact. Every PCM hash, frame count and loop range verifies.

The Rust browser build excludes voxel features and offers 2D/TUI. Its public
Chromium/mobile WebKit checks pass fresh/stale startup and an ordinary New Game
through the intro, bedroom stairs, Mom's pages/input ownership and Start menu;
native views were inspected. The subsequent Itasca-only data update preserves
all 729 other maps and the exact browser binary. It passes two default-profile
park controllers, wild/RUN/Continue, 15 clean production MCP calls, a current-pack
headless 2D capture and authenticated browser startup. Trusted sound activation
and visible animation settling precede source-dependent dialogue inputs.
The service is healthy; the final HTTPS pack and connection are verified.

Other completed checks: 133 browser/installer unit tests, six Pokédex catalog/
media/browser suites, 25 lobby tests, all-target server Clippy, production
battle/Mom controllers, stdio-MCP movement/Start and byte-empty stderr. A fresh
combined-pack MCP New Game passes Mom's pages, movement ownership and Start in
43 calls with stderr0. Whole-workspace fmt retains the existing offset-allocator
differences; new Rust files pass scoped rustfmt. Complete earned campaign and
auditory design acceptance remain open.

Only Geothite was recreated. Existing data, signing configuration, Pokédex,
Flygon, terminal mounts and earlier immutable browser assets remain. Native
terminal downloads retain their previous verified executable/pack. No old save
identity was relabeled. Pokédex release `20261029-verdigrim-25` is published and
verified separately in [its release record](pokedex-web.md).

Source/build recipes, pack and private rollback records are retained under
`/mnt/homelab/config/geothite/releases/combined-native-2d-itasca-20261029`.
Guarded rollback, provided the service has not changed since publication:

```sh
python3 /mnt/homelab/config/geothite/releases/combined-native-2d-itasca-20261029/deploy.py rollback
```

## Reviewed Minnesota release (2026-11-01)

Published source commit `2344e5294e3bb2ec47743283ead3959ebc98a73c` as
`geothite:reviewed-v11-20261101t134616z-final`, image
`sha256:5e1044c307182d6d3a8a1d705e22e77bdaa2ed105f0a1cb091f14d9470c1b46a`.
It serves the selected 743-map Minnesota v11 pack, including the inland winter
belt, early fishing access, repaired towns/caves and reviewed route changes.
Input pack: `content-packs/minnesota-reviewed-maps-v11.crystalpack`, SHA-256
`d3c16929aade3df2e4b26406dd1bd1c0ac61f05b05be5f569c41e1657c3899d9`.
The browser was built without voxel features. The service and public HTTPS
health endpoint are healthy. Broader map acceptance and earned campaign review
remain separate from this deployment.

Only Geothite was recreated. Signing settings, persistent game data, Flygon,
Pokédex and terminal download mounts were preserved. Terminal downloads retain
their existing pinned release; saves were not relabeled or migrated.

Frozen source, build recipes and guarded rollback state are retained under
`/mnt/homelab/config/geothite/releases/reviewed-v11-20261101t134616z`.
Inactive build caches were removed after publication. To restore the captured
previous service, provided its configuration has not changed:

```sh
python3 /mnt/homelab/config/geothite/releases/reviewed-v11-20261101t134616z/deploy.py rollback
```

## Optional Flygon

`/flygon` serves the local neural controller. It has no link from the root game.
The page autoloads its brain; Run starts submitting game inputs. The same Rust
server serves the game, neural WASM, viewer, and prepared connectome files.

The Docker build requires the BuildKit named context `game_content`. The supplied
Compose file maps it to the ignored `./content-packs` directory. With a remote Git
build context, set `additional_contexts.game_content` to an external directory on
the build host containing `core-modular.browser.crystalpack`.

Mount the prepared `graph.bin` and `metadata.json` read-only at
`/srv/crystal/web/flygon-data` (the supplied Compose file uses `FLYGON_DATA_DIR`,
default `./flygon-data`). Their combined SHA256 must match
`modpacks/flygon/dataset.json`. Without these optional files, the main game still
works and Flygon reports its missing data. Keep all data outside Git.

Validate `/healthz`, `/`, `/flygon`, `/flygon/`, the hashed Flygon worker imports,
and graph accessibility after rollout. Run the render review against the actual
HTTPS route and check manual takeover followed by Run resumes button submissions.
Keep the previous image, Compose configuration, signing secret and named data
volume for rollback; redeploy only the Geothite service.

## Integrated Minnesota release (2026-11-16)

Published source commit `1ba193536f302602ef4cd020e6d9a733fc2c5430` as
`geothite:integrated-20261116-final`, image
`sha256:87eceee5ea689ba112bfd2638cab84e7b9e662c1a56740e7ebf6e1bcb01f7401`.
The 743-map integrated campaign is selected in `planning/minnesota/world.json`.
The graphical browser remains native 2D for Minnesota. The browser TUI and both
native terminal downloads were also rebuilt and published.

- Input: `content-packs/minnesota-integrated-campaign-v03.crystalpack`, SHA-256
  `6d4cca148a5e277442a714fd1474d84bd315a1c63aea0ff80dd2a1003437e3a7`.
- Hosted content identity, after the server's standard wrappers:
  `b5f8411db557d73a0a9c755243415a33342537f619e200f544233dffc8027c02`.
- Hosted/installer pack SHA-256:
  `8498ef8d685b32d468876dc46da069ab394f97dd9da3da73b4c40ac9afecec0e`.
- Native manifest: `downloads/terminal/current.txt`, release
  `integrated-20261116`, with `aarch64-apple-darwin` and
  `x86_64-unknown-linux-gnu`. Its versioned pack remains in that release directory.

Public checks matched 160 assets byte-for-byte, including the graphical and TUI
WASM, checked both compressed HTML pages, and confirmed health, clock, Flygon
and Pokedex routes. A WebSocket handshake accepted the integrated pack identity.
Both public native downloads, the launcher and versioned pack matched their
manifest checksums. The Linux candidate installer and public macOS installer
completed with `--no-play` and zero output; the downloaded macOS executable
matched the built binary. No gameplay, controller journeys or visual play review
were performed, per the user's instruction.

Formatting and production worker-library Clippy passed with existing warnings.
The worker's all-target WASM Clippy invocation exposed a pre-existing native-only
test call to `bevy::tasks::tick_global_task_pools_on_main_thread` in
`crates/crystal-voxel-view/src/pokemon_motion_quality.rs:304`; that invocation did
not pass. The production library and all release binaries built successfully.
The missing WinterPine worker serialization mapping was fixed in both directions.
Deployment tooling now matches the lockfile's wasm-bindgen CLI 0.2.129 and
includes the authored JSON required by the Rust build.

The first public check received Cloudflare error 1010 for Python's HTTP client
and automatically restored the previous healthy service. Repeating publication
with curl-based asset checks passed. Signing configuration, persistent data,
existing saves, Pokedex/Flygon mounts and older immutable downloads were retained.
Saves were not relabeled or migrated.

Frozen source, build recipes, checksums and guarded rollback state are retained
outside Git at `/mnt/homelab/config/geothite/releases/integrated-20261116`.
The completed build uses `Dockerfile.build`; `package.py` assembles a fresh image
while retaining historical assets. Use fresh staging for another release.
To restore the captured previous service, provided its configuration has not
changed since publication:

```sh
python3 /mnt/homelab/config/geothite/releases/integrated-20261116/deploy.py rollback
```

Inactive local debug/release/WASM compiler output and installer scratch were
removed, retaining the finished clients, packs, saves and review evidence.
Local free space increased from about 342 GiB to 396 GiB. Docker build-cache
cleanup reported 10.69 GB removed; the server has 227 GiB free. Published and
rollback images, volumes and release files were preserved.

## Minnesota adventure release (2026-11-16)

Published implementation commit `c44d317df809d7d41e2980d1550eb6a330f89b86` as
release `adventure-20261116`, image
`sha256:7cad0ccfab7bb68ba360c05a25c8c828fdb49995779b982877c1d23607796d4b`.
The selected 743-map adventure revision adds forty hidden items, eight optional
Champion gym rematches, accepted native leader art, battle-party corrections
and revised campaign dialogue. Minnesota remains native 2D. The graphical
browser, browser TUI and both native terminal downloads were rebuilt and
published.

- Input: `content-packs/minnesota-adventure-v01.crystalpack`, SHA-256
  `36da6928a3aa0b0d75c9ad9a7abe353a9d79a7d3d1327eed2dc29d4fc4adb181`.
- Raw content identity:
  `d4ce7998673f393e69771a36c3575878060f121102f6db840ddf197c583979dd`.
- Hosted content identity, after the standard server wrappers:
  `80dc44ba48e3347b3e760095c295d10748386b9e8ab2a985350f043862236baa`.
- Hosted/installer pack SHA-256:
  `5254da0a14f9bf6acdb6378ee9d7d3f890d4f6eab13e5334ff079033212c4858`.
- Native manifest: `downloads/terminal/current.txt`, release
  `adventure-20261116`, with `aarch64-apple-darwin` and
  `x86_64-unknown-linux-gnu` plus its immutable versioned pack.

Fresh production client/server and pack builds passed. Formatting, all-target
asset Clippy and all-target WASM worker Clippy passed with existing warnings.
Native dialogue compilation/protection, independent static content/preservation
checks and all 602 audio PCM hashes, frame counts and loop ranges passed.
The hosted/raw pack comparison confirms only the standard clock, customization
and manifest wrappers were added; the authored game content is unchanged.

Public checks matched 148 assets byte-for-byte and checked both gzip HTML pages.
Health, clock, TUI, Flygon and Pokedex endpoints passed. The public WebSocket
welcome matched the hosted identity, including its 22,238-byte composed modpack
ID. Both native downloads, the launcher and versioned pack matched the public
manifest checksums. macOS and Linux each completed two silent `--no-play`
installs with zero output; scratch save markers survived reinstall, and no game
client was launched. Two synthetic installer safety cases also passed.

An initial candidate permission issue was corrected before rollout. Publication
passed without a production rollback. Existing saves were preserved without
relabeling or migration. The candidate service and installer scratch were
removed after the checks.

No gameplay, controller journeys, visual review or playthrough verification was
performed, per the user's instruction. Builds and static/content release checks
do not establish complete campaign balance or playability. Earlier releases'
gameplay evidence remains historical and does not verify this release.

Frozen source, build recipes, checksums, verification receipts and guarded
rollback state are retained outside Git at
`/mnt/homelab/config/geothite/releases/adventure-20261116`.
To restore the captured previous service, provided its configuration has not
changed since publication:

```sh
python3 /mnt/homelab/config/geothite/releases/adventure-20261116/deploy.py rollback
```

Inactive local Cargo caches were removed while retaining standalone clients,
packs, saves, review evidence and publication records. Actual local free space
increased by 8.24 GiB, from about 386 to 394 GiB. Remote Docker build-cache
cleanup reported 14.5 GB removed; free space rose from about 204 to 218 GiB.
Current and rollback images, volumes and release files were retained.

## Minnesota refinement release (2026-11-17)

Published implementation commit `e0225a19a8e5bf3814f83e2c2eea4f7a1eec1977` as
release `refinement-20261117`, image
`sha256:720ada1d8afb4b1e7b5ad10746ac194516cbddd899fbe1a17b1b4da12c50dc1a`.
The selected 743-map revision implements the [ten accepted improvements](../planning/minnesota/QUALITY.md#refinement-release-2026-11-17),
including checked save continuity, training and supply services, leader tactics,
discovery rewards, the rematch circuit, Reed's optional battle, distinct museum
activities and persistent restoration aftermath. Minnesota remains native 2D.
The graphical browser, browser TUI and both native terminal downloads were
rebuilt and published.

- Input: `content-packs/minnesota-refinement-v02.crystalpack`, SHA-256
  `5b173130deed7dc9c2af97b4bc2509e4f54f848c207361ff5278e21abce513ad`.
- Raw content identity:
  `a9f23e3bc1dadad74c377d02c23df61dcf9ed70c1b1d5e3918805593f37a9527`.
- Hosted content identity:
  `5eba128f5fd1871d87a58bfca28833476591708c6a6ad6d04075ed29a6f087e4`.
- Hosted/installer pack SHA-256:
  `5d82fb89116f82856f8cc1cef77ceac018d2863f756ad8e7bbb7bbf589cc6c70`.
- Native manifest: `downloads/terminal/current.txt`, release
  `refinement-20261117`, with `aarch64-apple-darwin` and
  `x86_64-unknown-linux-gnu` plus its immutable versioned pack.

Fresh pack and production client/server builds passed. Formatting and all-target
Clippy checks, including native targets and WASM workers, passed with existing
warnings. Pure unit tests covered save compatibility/recovery, move reminder
selection/payment, reward ordering and native dialogue. Independent static
content checks and all 602 audio PCM hash,
frame-count and loop-range checks passed. The hosted/raw comparison confirms
only the standard clock/customization flags, manifests and derived identity
changed; authored game content and compatibility metadata are retained.

Public checks matched 142 assets and both gzip HTML pages. Health, clock, TUI,
Flygon and Pokedex endpoints passed. The public WebSocket welcome matched the
frozen hosted identity and its 22,440-byte modpack ID; the handshake sent zero
gameplay inputs. Both native downloads, the launcher and the versioned pack
matched the public manifest and frozen artifacts. macOS and Linux each passed two silent
`--no-play` installs with zero output and preserved scratch save markers.
Two synthetic installer safety cases and 17 pure publication failure-case tests
also passed. No game client was launched.

Save continuity accepts only the exact checked integrated/adventure releases
at an idle overworld checkpoint, validates the target state and writes an
independent slot. Prior release saves remain intact. Same-pack recovery archives
a damaged primary before restoring a valid backup. Explicit custom paths use
same-pack recovery and do not automatically import another release's save.
These contracts were checked in pure tests; no player's game was opened for
release verification.

Publication needed no candidate corrections or production rollback. The owned
candidate and temporary private files were removed after successful public and
Mac checks. Frozen source, build recipes, artifact hashes, verification receipts
and guarded rollback state remain outside Git at
`/mnt/homelab/config/geothite/releases/refinement-20261117`.

To restore the captured adventure release, provided no later service or terminal
publication has replaced this release:

```sh
python3 /mnt/homelab/config/geothite/releases/refinement-20261117/deploy.py rollback
```

No gameplay, controller journeys, rendering, visual review or playthrough
verification was performed, per the user's instruction. All ten implementation
areas are delivered; campaign balance was assessed through static coverage,
legality, progression and resource checks, not measured through an earned run.
Historical gameplay evidence does not verify this release.

Inactive local Cargo output was removed while preserving standalone binaries,
packs, saves and review/publication evidence. Cleanup recovered 18,744,950,784
bytes (about 17.46 GiB), leaving about 393 GiB free. Remote cleanup recovered
14,926,995,456 bytes (about 13.90 GiB), increasing free space from about 194 to
208 GiB and leaving Docker build cache at 0 B. Only the owned toolchain image
was removed; current and adventure rollback image IDs were verified unchanged.
Volumes, packs, saves, downloads and retained evidence were preserved.

## Minnesota floorplans release (2026-11-17)

The production image for `floorplans-20261117` is
`sha256:89fe18b0c0fece1d00079d714cfeb08259562fb2348d4500be5f33cfe88ac9f9`.
The Linux server, native terminal clients, browser builds and workers were
built from `b81f13a420e0a093568aca2470caa13a7a5996b1`. The final external
pack was authored at `172a0e7004a3d749b534a7e6c0951d03978775a3`.
The release records include the compiler input lists and the exact subsequent
pack-authoring changes.

| Artifact | SHA-256 | Content identity |
| --- | --- | --- |
| Raw Minnesota pack | `2e2554784f82f6b9e53c8b1d7a24e5993f730eefb460239d3edc4bc7d558a3df` | `dc8ae8db7bb28ea5fa3265e739f51d60a8f0cc7c688ee5b532150153b01a8860` |
| Hosted and installer pack | `de7379234cf30325688436bd2c846b7f00e23830f0125d762ea962ad304fddb4` | `9522073158484d254ad32f1b53b4aac326e24e95ba4a5ccad0067f0810d26e1a` |

The release includes all eight redesigned Gyms, the cave and mine revisions,
twenty route landmarks and twenty progress-aware guides. In Brainerd the
player crosses the shallows freely before earning Surf. The guide, both
trainers and Sivren stand on connected sand, and the trainers stay on shore
after approaching a challenger. All 301 ordinary Minnesota trainers have a
native sight battle. All eight Gym leaders retain their conversation, badge
and reward scripts. The two restoration partners at Winona and Leech Lake use
the active Chorip overworld art.

The final compiled pack changes 83 maps from the v04 parent. Comparing both
pack variants verified all 466 existing field pickups and all 2,625 warp
destinations. The regional roster check covered 335 trainer parties and 761
members. Native room renders and static route searches covered the Gym
approaches, including the eight-step walk from Brainerd's entrance to Sivren.
The final Linux checks passed formatting, all-target asset Clippy, Brainerd
and mine regressions, bounded save migration and all 602 hosted audio PCM,
length and loop checks. Verification used static maps, pure tests and
distribution checks. Campaign pacing and combat difficulty were not playtested.

The public server passed all 145 client asset checks, both HTML/gzip checks,
the immutable native download hashes and a WebSocket welcome with zero gameplay
inputs. The network welcome matched the hosted content identity above.
Apple Silicon macOS and Linux x86-64 each completed two silent installs using
`--no-play`, including retention of a scratch save marker. No game client ran.
The proxy briefly returned HTTP 502 during the container switch and subsequently
passed the public health, asset and pack verification. The private candidate
was removed after verification.

The previous refinement image is retained for guarded rollback. From the build
host, use the release transaction if this is still the active deployment.

```sh
python3 /mnt/homelab/config/geothite/releases/floorplans-20261117/deploy.py rollback
```

Paced local Cargo cleanup and removal of duplicate deployment inputs recovered
12.79 GiB on the Mac. Remote cleanup of the isolated validation checkout,
release scratch files and 42 attributable BuildKit records recovered 25.24 GiB.
The final source, published and rollback images, user saves and canonical packs
were retained.

## Minnesota story flow release (2026-11-17)

The external Minnesota pack was authored at `757178bf915e754e7b7b19be7267b521123a5058`.
The server, browser clients and native terminal clients were built at
`c38901eb9f53eaa0f27a134455ee0d6524c29189`, which adds the new save migration.

| Artifact | SHA-256 | Content identity |
| --- | --- | --- |
| Raw Minnesota pack | `02e18ba71203d8aa2e10658c6babc53512daeef592ee53fb2900630f91aa57cb` | `79c1c85ebc25b200c9fe30689299af09bc88182828931576e512736395ebd521` |
| Hosted and installer pack | `f4dd4819f039631c81372078d8640f7929114d46bdef90f5ae90d42612a789a0` | `9fadbe2b9ecf745d0cc2852e745b12d6fd2634dc694e38ac0f0986e4e38624e3` |

Independent checks compared the raw and hosted releases with their respective
parents. They covered 906 revised text entries across 113 maps and 114 maps
with changed programs. All 355 Minnesota rooms retained their floorplans,
warps, placed actors, field pickups and trainer identities. The checks followed
Mina's date offer, the independent Itasca reading, the Duluth distress call,
Hale's evidence check, Finn's return dialogue and the League's eight-badge
requirement. All 602 hosted audio programs matched their PCM, length and loop
references. Verification used compiled data, pure tests and service checks
with zero gameplay inputs.

Production serves `story-flow-20261117` from image
`sha256:e57d284dd3704d73da34dc159490fb8a5d5c431aaff69c0b3af5f691a2eb13e0`.
The public status endpoint and installer manifest match the hosted pack above.
The first promotion rolled back when the public proxy rejected the readiness
probe's default Python HTTP client with status 403. The second promotion used
curl for that probe and passed with the same candidate image.

Both public browser clients returned the expected WASM. Checks covered 144
changed client assets, two HTML/gzip versions and the zero-input network
welcome. The Linux and Apple Silicon installers each completed two silent
`--no-play` installs and retained a scratch save. Both native file hashes
matched their release manifest.

Two story tests, 26 save tests and a native checkpoint migration test against
each pack passed. All-target Clippy passed for the asset, runtime and graphical
client crates, including the 18 changed scene test files. The compiler reported
existing warnings. None pointed to a line changed for this release.

The Mac's two retained release executables and the previous debug executable
remain available. Removing disposable profile output reduced the profiles by
3.04 GiB, leaving 386 GiB free. The build host removed the temporary client
build cache, private candidate and independent validation directory. The
previous production image remains available for rollback.

To roll back the active Story Flow release, run this on the build host.

```sh
python3 /mnt/homelab/config/geothite/releases/story-flow-20261117/deploy.py rollback
```
