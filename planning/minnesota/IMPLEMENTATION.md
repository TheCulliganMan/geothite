# Minnesota implementation sequence

## Repository boundaries

Read the root `AGENTS.md`, `docs/game-content.md` and `docs/cold-start-3d.md` first. Work against the completed ignored pack in `content-packs/`. The reference rendering tool never needs a pret checkout. Keep all packs, extracted assets, ROMs, source exports and verification output ignored; preserve base audio and unrelated game content.

The user rejected the geographic generator as the foundation for this project. Author the maps from this folder's fixed geometry and edit them deliberately. A renderer or serializer may lower authored data; it must not invent the routes, landmarks, story, terrain, or building layouts from OSM, a seed, or scenery quotas.

## Integration points already in the repository

| Concern | Existing boundary | Required use |
| --- | --- | --- |
| Pack and map definitions | `crates/crystal-assets/src/map_modules.rs` | Materialize complete `MapModule` values |
| Verified map addition | `CompiledMapExtension`, `with_map_extensions` in `mutation_protocol.rs` | Add mutually referencing maps atomically |
| Source-compatible derived art | `CompiledTilesetExtension` | Add bounded art and collision together when needed |
| Map dimensions and seams | `crates/crystal-core/src/map.rs` | Use typed attributes, cardinal connections and offsets |
| Coordinates and metadata | `crates/crystal-core/src/world/map.rs` | Keep source, core and metatile units separate |
| Story control and flags | `systems/script_control.rs`, `systems/script_flags.rs` | Use compiled commands and persisted state |
| Encounters | `crates/crystal-core/src/world/encounters.rs` | Use full timed tables and explicit per-zone bounds |
| Gameplay frontend | `crystal_bevy::VisibleShellController` in `bevy_shell.rs` | Own scripts, movement, menus, battles, travel and dialogue |
| Native capture | `crates/crystal-bevy/examples/render_at_location.rs` | Headless production captures for verification |

`with_map_extensions` adds map modules, metadata, spawns, block/split payloads and optional encounters. It does **not** by itself author the new global trainer parties, story constant allocation, Minnesota Pokegear landmarks, badge labels or region-specific recovery policy. Inspect the existing builders and implement one typed Minnesota overlay inside `crystal-assets` to perform those additional mutations and derive the final verified identity. Do not assume a map-only pack addition completes the story.

A new-game spawn override changes `SPAWN_HOME`. It does not automatically retarget every canonical home, blackout, Fly, phone or story callback. Audit each one that can run during the Minnesota slice. Do not let a new Minnesota save silently return to Johto.

## Planned source layout

These names are implementation targets, not commands or files already delivered:

- `crates/crystal-assets/src/minnesota_modpack.rs`: typed pack overlay and validation orchestration.
- Focused leaf modules for authored map lowering, story/flag allocation and trainer/landmark overlays if the builder becomes large.
- `crates/crystal-assets/src/bin/pack_minnesota.rs`: consume a verified local base and authored design, publish to a distinct ignored path.
- `modpacks/minnesota/`: runtime-facing original authoring inputs and a verified runbook after the first slice exists.

Keep the current planning schemas readable and explicitly versioned. Decide whether to lower their brush records or export hand-edited block arrays from a local authoring tool. Either choice must preserve the fixed coordinates, original designs and source art. Do not duplicate the base game's source catalogs into tracked files.

## Milestones

### M0 Source style and layout fitting

Compare the blockouts and source references at 1:1 scale. Complete Route 1's source-quadrant fitting: water corners, cave approach, tree edges, terrace ramps and the farmhouse door. Update its explicit layout and events together. Split the fitted route at the declared chunk origins and inspect both sides of each active seam. Record the native map and route-wide mosaic as local evidence.

For routes 2 through 20, the current six-district reservations are starting geometry. Each needs individual hand authoring; a bulk render is not acceptance. Preserve each route's Minnesota identity, main/optional path graph and authored event budget.

### M1 Winona onboarding

Build the home floors, field station and town; wire stairs and exterior returns. Implement guardian interception and starter selection with idempotent rewards and a real requested trainer spawn. Keep Route 1 locked until starter receipt. Prove dialogue ownership and Start/Pack/Party behavior through the controller and MCP.

### M2 Route 1 and Rochester

Complete all nine Route 1 chunks, town gates, cave and farmhouse returns, route trainers and persistent pickups. Build the Rochester clinic, shop, gym and story NPC. Resolve every event reservation to a collision-tested core tile and freeze a door/warp table. Add new-game-to-first-badge controller coverage before expanding the region.

### M3 Prairie and Twin Cities

Author R02 through R05 and the Mankato, Minneapolis, St Paul and St Cloud hubs. Implement gyms 2 through 5, notebook/dispatch scenes, courier battle and the two-badge north gate. Verify that R04 remains a returning excursion and cannot softlock the player in St Paul.

### M4 Lakes and headwaters

Author R06 through R08, R14 through R19, Brainerd/Bemidji gyms and the lake/prairie hubs. Add fishing and optional Surf through inherited mechanics, the Itasca report and optional observation rewards. Test walking completion without Surf. Validate that the optional Cut shortcut has a valid non-Cut route around it.

### M5 Iron Range and North Shore

Author R09 through R12, the mine cache, Grand Rapids depot, Duluth/Grand Marais and harbor workshop. Implement foreman and survey-chief battle outcomes, badge 8 and receiver restoration. Local block and object changes must survive reload, blackout and revisiting the map.

### M6 Ely and border loop

Author R13, R17, R20, Ely and International Falls; finish all League parties and rooms. Prove that the Voyageurs loop cannot bypass the restored-survey requirement. Add Hall of Fame persistence and a safe return to the region after completion.

### M7 Pack publication and client proof

Build distinct native, browser and text packs from their matching bases with the Rust overlay, preserving each base's audio storage. Verify the actual files, run the repository gameplay regressions and Minnesota-specific tests, inspect production 2D captures, and launch the production client only as authorized by the user's live-review request. Document the commands actually executed in the runtime mod-pack runbook. No unexecuted builder command should be described as working.

## Next agent task

Read this folder, select the exact source facade and terrain recipes for Route 1 and Winona, then complete M0 and M1 before proposing the region finished. Produce a reviewable source diff, a freshly verified ignored slice pack, collision-backed travel evidence, and native-scale production captures. Keep a concrete list of failed checks and unresolved fittings. Do not create another independent runtime dispatcher or a renderer-owned simulation to make the demonstration appear interactive.
