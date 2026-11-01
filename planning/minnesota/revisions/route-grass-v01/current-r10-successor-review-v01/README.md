# Route 10 central grass neck and stair directions

Four native metatiles add sixteen grass quadrants around the existing central
patch's entry corners. The checked west/east approach now requires nine grass
steps each way instead of seven. Existing grass, sign access, cave mouths,
stairs, trainers, encounters, rewards and shoreline remain unchanged. Whole-route
and before/after source renders were reviewed, followed by three native 2D
cameras with zero pixel mismatch.

392 production calls verify new-grass encounters/RUN, walking in both directions,
menus and actual Continue. Three obsolete drop/ramp messages now describe the
existing stairs. A separate 507-call course reads the corrected sign and road
keeper dialogue, wins actual T01 using FIGHT/SURF, verifies the 1088 reward once
and corrected defeated text, then repeats after Continue. Initial partner, badges
and position are explicit local QA fixtures, not earned regional progression.

Rust publishers: .local/route-grass-review/portable-r10-neck-grass-tools-v01/
revise_r10_neck_grass and portable-r10-stair-guidance-grass-tools-v03/
revise_r10_stair_guidance. Each takes PACK SHA256 BEFORE_JSON AFTER_JSON
FRESH_OUTPUT; preservation receipts record the exact inputs and outputs.
Directions-only publication preserves all tested grass and gameplay inputs.
All-target crystal-assets Clippy and owned formatting pass; workspace formatting
has unrelated existing differences.

This scoped grass and directions acceptance does not prove complete current
Route10 traversal, cave gameplay, neighboring-town journeys or statewide completion.

`full-route-production-v02.json` expands the evidence to 2403 production calls
through both chunks, both terrace stairs in both directions, and round trips
into Grand Rapids and Duluth, repeated after Continue. These actual maps are
present in the retained production responses. Cave interior and Surf play are
not freshly re-tested by this course.
