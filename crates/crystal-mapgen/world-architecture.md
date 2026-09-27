# Published worlds and later on-demand generation

This document specifies the later server work. No streaming server, background
worker, or shared NPC simulation is introduced by the current map generator.

## Identity and publication

A world revision binds its base-pack content hash, generator version, frozen
source revision, dimensions, selected reciprocal links, and starting cell.
`WorldMapRegistry` assigns exact map constants and numeric map/spawn IDs. The
registry must accompany the published world and only gain allocations; IDs must
never be inferred by truncating hashes. Cell-local script and object names are
namespaced when assembled. A different generated pack requires a new publication
path; existing player saves remain tied to the old pack identity.

Freeze the source snapshot, semantic cell plan, and both sides of every shared
edge before publication. Later exploration must read those decisions, even if
OSM has changed. A future streaming identity must include all recipe/settings
versions and the *whole* immutable source snapshot revision, not just the set of
cells the player has visited. The current finite-region identity deliberately
binds the complete assembled region.

## Jobs and cache

Rust server jobs should accept `(world_revision, cell_id)` and return an immutable
validated cell artifact. Deduplicate requests by that key, limit worker count,
and prepare immediate neighbors at lower priority. Each job proceeds through
source availability, deterministic planning, scene validation, pack assembly,
and atomic publication. Validate reciprocal edge contracts before either side
can be published. Share regional service allocation across jobs so generation
order cannot add, remove, or relocate a Center or Mart.

A failed job leaves the player in their current cell and keeps the last known
valid cached artifacts. Return a retriable status; never move the player into a
partially built map. Use separate temporary paths per job and promote only fully
verified content. Persist job state so process restarts cannot duplicate work or
change registry allocations.

Use cached regional source data or a dedicated geographic-data provider for
scale. Public Overpass is shared infrastructure, not a per-player streaming
backend: https://dev.overpass-api.de/overpass-doc/en/preface/commons.html

## Player progress and acceptance gate

Keep inventory, party, visited cells, respawn location, defeated trainers and
reward flags in mutable player saves. Published scenery, collision, entrances,
and shared transitions are immutable. Existing multiplayer presence/chat,
battle invitations and trades should use the exact same pack identity on both
clients. Shared NPC schedules/economies, extensive quests, and terrain editing
remain separate later work.

Do not enable on-demand publication until the connected seven-cell showcase has
passed its exact-art review, all service/interior/save checks, two-client tests,
unchanged-audio checks, and browser performance budget. The current asset-free
test suite verifies planning invariants, not that acceptance gate.
