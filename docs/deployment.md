# Deploy Geothite multiplayer

## Requirements

Use Docker Engine with BuildKit and the Docker Compose plugin. Run commands
from the repository root. The Dockerfile includes the Rust and WASM toolchains;
no host Rust installation or sibling repository is required. The initial build
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

Open `http://localhost:3003` for local play. Remote browser play requires HTTPS
for the per-player Web Lock. The browser obtains its identity automatically
from `POST /v1/session`; no manually issued token or account is required.

The image contains the server, WASM client, browser assets, and canonical
exported browser pack. The embedded modpack JSON is included in the repository.
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
Git changes. For a stack using a remote Git build context, specify its main
branch and `pull_policy: build` to resolve and build that branch on each up.
Do not use `--no-build` when you need a fresh build.

BuildKit caches dependencies and compiled targets. The build uses two Cargo
jobs by default; override with `docker compose -f docker-compose.production.yml
build --build-arg CARGO_BUILD_JOBS=4` when resources allow. The browser build
uses the `web-release` profile to limit linker memory use.

## Persistence and cleanup

Multiplayer ratings persist in the `crystal_multiplayer_data` volume. Game saves
remain in each player's browser. Back up server data and preserve the signing
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
