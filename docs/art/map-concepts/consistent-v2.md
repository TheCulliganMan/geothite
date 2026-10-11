# Consistent map revisions — 2026-10-10

Generated with the built-in imagegen tool. These are revised art references, not live renderer screenshots, runtime terrain, or content packs. Prior references remain available.

The edit targets were local viewer captures of Route29 at (38, 9) and NewBarkTown at (8, 8), zoom 0, orbit 0, requested viewport 1280×960, from the ignored realtime-clock.browser.crystalpack. The existing local viewer executable was used; this request did not rebuild it. User-attached forest and village images supplied the style. The revised forest also supplied the town's shared style.

Selected outputs:
- [Route 29](route29-consistent-v2.png)
- [New Bark Town](new-bark-consistent-v2.png)

Visual review: both use layered green crowns, warm sunlight and soft canopy shadows, richer ground detail and fuller vegetation around trees. Route 29 shows contiguous tall-grass beds and a joined visible lower ledge. New Bark retains the four-building arrangement, four characters, signs and open courtyard. Imagegen interprets small foliage, character, framing and material details; authored map coordinates, collision, elevation and actor state must remain authoritative for any runtime implementation. These PNGs are not currently consumed by the renderer and are not gameplay evidence.

## Route 29 prompt

Use case: style-transfer
Asset type: revised Route 29 map art for the Geothite project.
Input 1 is the edit target: the actual Route 29 game capture. Input 2 is the user's preferred forest style reference.
Redo input 1 into a polished, cohesive stylized 3D forest map matching input 2's lush foliage and exceptionally warm dappled lighting. Preserve input 1's exact elevated camera, framing, tree clusters, walkable corridors, tall-grass footprints, terrain elevations, bottom ledge and small red-capped player location and scale. Keep the partial building at upper left in its location but harmonize its materials.
Replace repeated ball-like crowns with varied natural layered leafy canopies, rich greens, rounded leaf clusters, and sturdy softly textured brown trunks. Add abundant understory: ferns, low bushes, moss, leaf litter and small plants clustered beneath trees, never blocking paths. Forest floor should have subtle organically blended grass and earth, softly mottled leaf shadows, warm sunlight pools and cool canopy shade. Tall grass must form uninterrupted connected beds across neighboring tiles with flowing overlapping blades and natural edges; no gaps or tile outlines. Make visible ledge faces and grassy top rims a single contiguous coherent terrain edge, with joined corners and consistent height, never detached platforms. Keep all level topology from input 1.
Art direction: warm green and cream palette, restrained teal foliage shadows, soft ambient occlusion, clear readable player, handcrafted high-quality 3D adventure game environment. No UI, labels, watermark, grid, disconnected ledges or new landmarks. This is one map image, not a collage.

## New Bark Town prompt

Use case: style-transfer
Asset type: revised New Bark Town map art for Geothite, consistent with the revised Route 29.
Input 1: actual town game capture, the EDIT TARGET. Input 2: user's preferred village visual reference. Input 3: revised Route 29, use its exact foliage style, color treatment and lighting quality so these two maps feel like the same game.
Redo input 1 as a polished stylized 3D village. Preserve its elevated camera, 4:3 composition, building footprints, teal-roof laboratory and terracotta houses, door/window placements, signs, open courtyard and all four character locations and small scale. Match the source layout closely rather than replacing it with input 2.
Improve the repeated tree spheres into varied rounded layered leaf canopies with expressive silhouettes, sturdy trunks and rich dark green shade under fresh sunlit leaves. Fill understory along forest borders with ferns, low shrubs, moss, small flowers and leaf litter, keeping courtyard, doors and paths open. Blend green grass into warm sandy earth organically, continuous across tiles, no square seams or staircase grass border. Add subtle earth texture and forest floor detail.
Lighting: warm afternoon sunshine with soft irregular canopy shadows on the ground, gentle bounced light on cream walls, dimensional roof tiles and soft ambient occlusion. Use the same lush greens, warm earth, restrained teal shadows and natural lighting as input 3. Materials must be tactile and softly rendered, not flat slabs. Keep character silhouettes readable and proportionate.
No additional landmarks, buildings, people, UI, labels, watermark or grid. One finished town image, not a collage.

