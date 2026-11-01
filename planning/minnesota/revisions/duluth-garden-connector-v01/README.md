# Rejected Duluth garden street connector

The selected cliff repair left a short civic street ending against the garden wall. This follow-up adds a real north-south cave36 flight and a turning paved foot landing. Walls on both sides of the flight and below its foot force the player to use the steps.

The complete town stays 64×46 metatiles. All 38 doors, 76 exterior warps, programs, actors, items, story gates, art, audio, outdoor recovery and regional connections retain their existing data. Five existing metatiles change; no art is imported or recolored.

The garden-to-main-street trip from (55,33) to (64,37) improves from 29 to 13 ordinary walking steps, with actor bodies included. All ten existing ledge returns retain their 11–27-step paths. The new stairs are at metatile (28,17), upper landing (56,33), lower turning landing (56,36)/(56,37), and street connection through (57,36)/(57,37) to (58,36)/(58,37). The two retaining-wall blocks at (27,17)/(27,18) and the cap at (28,19) prevent walking sideways across the height change.

## User rejection

The user rejected the pictured geometry: “No good.” This candidate must not be selected. Passing collision, preservation and pixel checks did not establish believable terrace composition. Its split retaining wall, isolated rock cap and wrapped landing remain visual defects. All evidence is retained as rejected history.

## Authoring and status

`before.layout.json` reproduces selected Duluth v03. `after.layout.json` adds five explicit points using installed native blocks. `publication.contract.json` binds the full native module and every indexed anchor without relocation. Publication uses the existing Rust `recompose_authored_city` builder, with a fresh ignored output.

The unselected Rust-published preview passes exact full-pack preservation, all 377 audio programs, three native views with zero pixel differences, and a production stdio journey through the stair, Save/Continue and the real clinic. Fresh Cargo publication and the new default-profile native regression remain pending; the selected pack is not changed yet. Every render, including the rejected cave17 and quarter-stair experiments, is retained in `planning/minnesota/renders/duluth-garden-connector-v01/`. The rejected cave17 image is a cave-floor corner rather than a visible stair; the quarter-only proposal did not provide a convincing north-south climb. The accepted source candidate instead reuses the full cave36 flight already installed in Duluth.

The exact preservation/graph audit is `tools/audit_duluth_garden_connector.py`. The new production controller case is `duluth_garden_stair_turn_reaches_main_street_continue_and_real_clinic`: side-wall denials, descent, turning foot, real street, Save/Continue, actual clinic entry/return and the uphill trip, with unchanged party and bag. Explicit initial fixtures do not prove earned statewide progression.

The south ledge now has two deliberate uphill options. Its nearest ordinary return stays 23 steps. The authoring return band becomes [54,33,57,43], covering the waterfront flight and new garden flight; sealing that whole band must remove every ordinary uphill path. This replaces the obsolete single-flight assumption without weakening reverse-hop denial.

Executable publication command (a fresh output is required):

```sh
target/runtime-fps/debug/recompose_authored_city \
  content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack \
  planning/minnesota/revisions/duluth-garden-connector-v01/after.layout.json \
  planning/minnesota/revisions/duluth-garden-connector-v01/publication.contract.json \
  content-packs/minnesota-current-duluth-garden-connector-v01.crystalpack
```
