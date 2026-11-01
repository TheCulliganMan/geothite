# Route 8 shoreline grass and stair directions

Three native grass metatiles extend the existing patch across the shoreline
path in MnR08C00R01. Twelve grass quadrants are added, none removed. The
previously dry approach now requires six grass steps each way in the ordinary
walking model. Cut gates, bridges, water, stairs, actors and rewards are preserved.

Whole-route and before/after source renders were inspected, as were both native
2D cameras with zero pixel mismatch. A 284-call production course verifies
walking both ways, encounters in the new grass, RUN, menus and actual Continue.
Three obsolete drop/ramp messages now describe the existing two-way stairs.
A separate 464-call course reads the sign and walker dialogue, wins actual T07
using FIGHT/SURF, verifies the 640 reward once and corrected defeated text,
then repeats after Continue. Initial partner, badge and placement are local QA
fixtures; no earned statewide progression is claimed.

Rust publishers are retained under .local/route-grass-review: portable-r08-shore-
grass-tools-v01/revise_r08_shore_grass and portable-r08-stair-guidance-grass-tools-v03/
revise_r08_stair_guidance. Both take PACK SHA256 BEFORE_JSON AFTER_JSON
FRESH_OUTPUT. Exact inputs and hashes are in the preservation receipts. The
grass pack filename retains an old south-woods label; its native changes are
the shoreline path. Directions-only publication preserves all tested terrain
and gameplay inputs. All-target crystal-assets Clippy and owned formatting pass;
workspace formatting has unrelated existing differences.

Scoped grass and stair-directions acceptance does not establish full current
Route8 traversal, neighboring-town journeys or statewide completion.

Expanded current course: `full-route-production-v02.json` records 4364
production calls through all four actual chunks, both gate approaches and
stair ascent/descent, repeated after Continue. The first planner omitted the
second column; its failed harness is retained. `current-cut-native-v01.json`
verifies Cut permissions, Great Ball once, Continue and return. Neighboring town
transitions remain separate.

`connections-production-v01.json` now verifies 2261 production calls entering
Bemidji and Leech Lake, returning across both boundaries and repeating after
Continue. `current-exterior.contract-v02.json` supersedes the scoped contract and
retires the actual historical BIRCH_RETURN_DROP and CEDAR_PIER_BANK_DROP IDs.
