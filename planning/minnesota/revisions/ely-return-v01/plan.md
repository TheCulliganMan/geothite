# Regional plan and city doors

The source check follows 20 routes, 19 hubs, 53 route chunks, 82 adjacency and gate records, 13 quests and 24 milestones. Its symbolic progression reaches every gym and all 24 milestones, including the Champion. The eight gym layouts have registered entrances and exits. Route 11 has a four-metatile terminal. Route 12 reaches Grand Marais through the Cedar Approach and meets Boundary Waters from its northern chunk. The five registered mine floors connect the drill core back to Route 9. The Boundary Waters check also rejects 17 deliberate changes to gate assignments, offsets, guards, fees and prerequisites.

The original city expansion archives retain their exact 117 source-room reservations, indexed bindings and 64×48 exterior plans. Minneapolis, St. Paul and Duluth each have a later compact exterior and subsequent building revisions. Applying a hash of the original city's complete room scripts to a later edition caused dialogue and shop changes to fail a geometry check. The compact source checks now compare the indexed door anchors, floor layouts, city seams and registered interior connections separately.

Both selected Ely Return editions passed the archive checks for 118 street entrances in those three cities, including the correct safe outside landing for every return carpet. Across the 117 expansion rooms, all 133 registered street or stair entries have the expected destination index and arrival collision. Internal stairs and ladders have indexed reciprocal returns. The three realigned clinic and shop doorways in St. Cloud and Bemidji also land on their authored tiles. A deliberate fourth-exit substitution at the St. Paul depot was rejected even though the substituted clinic landing was walkable and the other three depot carpets stayed correct.

Run these checks from the repository root.

```sh
python3 planning/minnesota/tools/check_plan.py
python3 planning/minnesota/tools/check_boundary_walking_links.py --self-test
for chosen in content-packs/minnesota-ely-return-v01.crystalpack content-packs/minnesota-intro-ely-return-v01.crystalpack
do
  uv run --offline --with cbor2 --with pillow python planning/minnesota/tools/check_plan.py --pack "$chosen"
  uv run --offline --with cbor2 --with pillow python planning/minnesota/tools/check_city_binding_plan.py "$chosen"
done
```

The graph checks reachability from recorded requirements. The archive checks inspect the actual three city exteriors, room events, collision and Safari tables. The Route 1 flood fill uses the source layout and the supplied tileset. Species checks confirm the identifiers in the plan exist in the archive. Current encounter availability, trainer behavior, and walking through each of the 296 listed maps require separate evidence.
