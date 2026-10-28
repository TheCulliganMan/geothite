# Deploy Geothite multiplayer

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
