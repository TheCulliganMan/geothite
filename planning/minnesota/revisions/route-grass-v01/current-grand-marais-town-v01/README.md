# Grand Marais town exterior review

Accepted locally as `route_grass_grand_marais_town_revision`. Pack: `content-packs/minnesota-current-grand-marais-mart-v01.crystalpack`, SHA256 `7f60e243cbda816f497303907c8db124451e00d750d9e9afb8b6c48c2d0f08b3`. See `acceptance.json` and `selection-receipt-v01.json`.

The mart's mixed wooden wall is replaced by its original native pink-brick component at metatile(10,4). The complete facade matches Cherrygrove's original assembly. Every collision quadrant, all six existing door/return pairs, actors and signs remain unchanged. The complete canonical source preimage matches installed terrain; its one-cell successor is now the canonical layout. The dossier is unchanged.

Five meaningful child controller cases pass: all six original room returns; missing-badge and signal guard refusals; four qualified outward/return lanes and free inbound care; the Surf checkpoint, seams and visible water boundaries; and all seven residents/three signs followed by Continue. The first over-specific test filter matched zero cases and is explicitly rejected in `native-filter-correction-v01.json`; only the corrected four-case log plus the separate public-path case count as acceptance. Existing Cedar grass/ledge acceptance remains inherited rather than running an unrelated duplicate case.

The production client loads the actual reached mart-return save at(21,10), walks to(20,10), opens Items/Balls and returns to walking:15 calls, clean stderr. Eight fresh actual 2D captures were visually inspected and match source with zero pixel mismatches. Whole-town source and connected mart before/after views were inspected too. Shared all-target crystal-assets Clippy completed after registration of the bounded publisher; its source/log binding is retained. Three invalid publisher controls fail without outputs or parent mutation.

Independent normalized whole-pack comparison proves all729 other maps and all State Fair facades, scripts, encounters, trainers, art and audio unchanged. Warm North Shore geography and the existing28 town grass quadrants are retained. No ledges are added to this flat harbor town. Explicit local party, badge and signal fixtures are QA setup; this is not earned statewide progression.

Twelve town reviews, seven remaining frozen facade candidates and final current-story integration/coverage remain open. Safari paid capture remains unselected. All render originals, initial diagnostics, source snapshots and reached saves are retained; `review-history-before-selection-v01.md` preserves the prior pending state.
