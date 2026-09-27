# Security review — September 16, 2026

The requested release is a new Vercel project with Flygon as its entry page and
Pokémon embedded inside Flygon. It is not deployed yet.

## Verified changes

- Updated rustls 0.23.44 to 0.23.45 for RUSTSEC-2026-0285. `cargo audit`
  reports no known vulnerabilities. `cargo check --locked -p crystal-net
  -p crystal-mapgen` passes for its direct consumers.
- Bounded browser asset downloads to 256 MiB by default, checked declared sizes
  before allocating, and cancelled streams on overflow. Regression tests cover
  oversized headers, unbounded streams, mismatched lengths, and valid downloads.
- Rejected neural clock overflow before mutating simulation state. Queue indexing
  now takes the modulo before narrowing to a WASM-sized index.
- Added server MIME-sniffing, referrer, framing, object, base URI, form and browser
  permission restrictions. Same-origin framing preserves Flygon's embedded panel.
  All 16 server tests pass, including live HTTP success and 404 header checks.

## Remaining verification

- Vercel CLI device authorization is pending. Safari recognizes the account but
  its Allow Access button has remained disabled. No new project is verified.
- A production Flygon build and browser smoke test remain required. The local
  release assets were located on the existing host and copied outside the repo.
  The connectome matches its pinned sizes and digest; the full-data test passes.
- RustSec reports maintenance warnings for bincode, paste, and ttf-parser.
  These are not patched-vulnerability findings; replacement requires compatibility
  review, especially for serialized content.
- All 34 default Flygon tests and the separately run full MaleCNS integration test pass.
- HTTP hardening is a partial CSP, not a strict script policy. The deployment
  needs a policy verified against its actual generated scripts and WASM workers.

Content packs, audio sources, and generated PCM have not been modified or added.
Generated build output must remain ignored. These checks do not establish that
all possible security issues have been eliminated.

## Flygon Vercel candidate verification

`tools/prepare-flygon-vercel.py` converts a verified candidate into a Flygon-root
static site, removes stale compressed assets, hashes inline scripts for CSP, and
keeps the game pack external. Only the existing clock and pack endpoints are
proxied. This deployment remains dependent on geothite.ryanculligan.com for them.

Local browser verification with the generated policy loaded all 166,700 neurons
and 25,582,938 connections, loaded the embedded game, and submitted a neural
START decision. The generated site is about 274 MB; the Vercel account's upload
limit must be checked after authentication. Actual Vercel proxy behavior and
public HTTPS browser behavior remain unverified. No project was created.

The authorization blocker persisted through multiple checks and two CLI device
codes. Safari is signed in but Allow Access stays disabled. User completion of
Vercel login is required before project creation and deployment can proceed.
