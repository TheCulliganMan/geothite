# Iron Range mine junctions

The five workings form a loop. Haulage and the Sump each have five shafts, including connections that reach opposite sides of the same floor. The old shaft used the same small dark rock-and-ladder silhouette on every landing. All 17 active shaft tiles on these five floors now show an upright ladder for a climb or a recessed hatch for a descent. Each remains on its existing 16-pixel warp tile. The ordinary rock stairs between elevations on the same floor retain their existing art.

Three pieces of mine equipment help distinguish destinations at player scale. Sleeper tracks reach haul landings. Recessed fan housings mark the Ventilation branches. Follow the discharge pipe past the Sump pressure grille and the same red flanged wheel appears beside the Pump Gallery arrival. The side paths retain both pairs of connections back through the mine.

The eastern Haulage descent follows an exposed cut containing fifteen water tiles beside the working, within sight of the hatch down to the Sump. The dry bank at x=30 permits casting into the existing Minnesota lake group. The pool occupies former solid rock. Its north, east and south edges are closed. It is accessible from the east working before the Haulage shutter opens. Once the shutter opens the west working has access by the original path.

[Native artwork](fixtures.json) uses 16×16 authored ink grids. A dot inherits the underlying mine pixels. All terrain coordinates in [the junction layout](junctions.json) are 16-pixel player squares except the six 32-pixel drainage-bank placements. The publisher preserves the collision of every shaft, equipment tile and rail. It checks both shutter states using production movement and occupied actor positions, comparing the old and new dry reachability from the west and east arrivals. Surf gains only the water in the drainage cut. The controls, gates, people, recovery squares, story scripts and rewards keep their original positions.

## Fair Presentation ladder approaches

Haulage's four descents have a destination plate beside each hatch.
The Sump uses a teal water-drop plate, Ventilation a yellow fan, and the Pump
Gallery a red valve. Approach each plate and read it to see the full destination.
The Haulage signs also identify the west and east landings. In the Sump, the
Pump Gallery plate below the shutter is readable while the gate is still
closed. The same plate marks the hatch beyond it.

[Plate positions and text](approach-plates.json) use 16-pixel player squares.
Each placement records its x, y, and the expected compiled metatile ID.
[Plate artwork](approach-art.json) uses the existing native palette banks.
The preview builder accepts either exact Fair Presentation edition and writes
to a fresh ignored path under its own save identity.

```sh
cargo run --locked -p crystal-assets --example pack_northstar_iron_wayfinding -- \
  target/iron-wayfinding-review/fair.crystalpack \
  target/iron-wayfinding-review/preview.crystalpack
```
