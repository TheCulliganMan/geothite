# Route 7 northern grass and stair directions

Eight native metatiles add 32 grass quadrants to the northern entrance and
existing Repel pocket. The entrance requires two grass steps each way. The
existing bridge approach and sign access remain clear. No original grass is
removed. Two obsolete drop messages now describe the existing two-way stairs.
All other maps, actors, events, rewards, encounters, teams, art and audio remain
unchanged. Whole-route and native 2D renders were inspected; both native cameras
have zero pixel mismatch. Directions-only publication preserves these pixels. The current rebase retains both complete Route7 maps, tilesets, encounters, teams and battle/runtime inputs exactly; earlier tests are inherited through exact inputs, not relabeled as fresh runs.

334 production calls verify new-grass encounters/RUN, Repel once, pickup text
input ownership, inventory and actual Continue. A separate 745-call production
course reads the corrected sign, wins the actual T02 battle using FIGHT/SURF,
verifies the 540 reward once and corrected defeated text, then repeats after
Continue. Initial partner, badge and placement are local QA fixtures.

Rust publishers are retained under .local/route-grass-review: portable-r07-
north-pocket-grass-tools-v03/revise_r07_north_pocket_grass and portable-r07-
stair-guidance-grass-tools-v03/revise_r07_stair_guidance. Both take PACK SHA256
BEFORE_JSON AFTER_JSON FRESH_OUTPUT. Exact inputs are in preservation receipts.
Source guard and wrapper path failures were corrected before accepted outputs;
their failed logs are retained. All-target affected-crate Clippy and owned leaf
formatting pass; workspace formatting has unrelated existing differences.

This scoped acceptance does not establish complete current-route traversal or
completion of the statewide grass objective.

`connections-production-v01.json` now verifies 2773 production calls entering
Bemidji and Leech Lake, returning across both boundaries and repeating after
Continue. `current-exterior.contract-v02.json` supersedes the scoped contract and
retires the actual historical SANDBANK_RETURN IDs.
