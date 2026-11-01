# Iron Range West cliffs and native ledge stairs

Selected as `world.iron_cliff_stairs_revision`.
Pack: `content-packs/minnesota-current-iron-cliffs-v02.crystalpack`. SHA256: `9181eba4c44d85e840e99d29c6284ecaebe6aa099ccb373d6c703364ff205e98`.
Identity: `f98e360d072ad075c637dd40277f1231aa984be6e559fe5b13b38240e719da89`.
Parent: `content-packs/minnesota-current-grass-lower-woods-v01.crystalpack`, SHA256 `6cb0c84e8b41436185a98c1afdaf7c43fdc0c5ccb147b950459f2a55f96d77f7`.

The user identified native cave stair blocks12,30,36. The R09 private tileset
appends their exact source pixels and Floor/Wall collision at85,86,87. No ladder
or warp is used for these climbs. All133 previous blocks keep their exact pixels,
metatile bytes, used palette entries and collision.

The quarry stair at meta24,20 uses cave12. Cave30/36 form the mine-apron stair
flight at18,17 and18,18, with an inside cliff corner joining the foot. Both
flights work in both directions. A separate native south ledge at19,17 gives a
dry return after the grassy climb. Both lanes block reverse movement and side
bypasses. The ordinary climb from core39,36 to39,33 is13steps versus a3tile hop.
The lanes require2and3grass steps uphill and zero on the return. The established
quarry circuit retains its4/0grass contract.

The new stair/drop pair is west of Mina's existing sight course. Her placement,
approach and actual party remain intact. The v01 pack intersected that course
and is rejected. All failed sources, logs, screenshots and candidate packs remain
retained; see CANDIDATE-HISTORY.md. No rejected candidate was selected.

Ten thin bedrock ribs become broader coherent formations, including the joined
northern ore crown. Repetitive decorative crown boulders are removed.173cells
change across the four R09 exterior chunks. All726other maps, every actor,
current Minnesota party, mine/story room, wild/fishing table, connection, gate,
recovery field, title, audio and unrelated art remain exact. All377audio programs
pass PCM hashes, frame counts and loop ranges. The full regional species proof
covers all four exteriors and the current mine, excluding retired families and
removed evolutions. North Shore and Cedar remain ice-free; inland winter is
preserved byte-for-byte.

Four native controller cases pass: both stair directions/both uninterrupted
ledge lanes/Continue; full route/seams/rails/rooms/Cut access; actual trainer
battle/Surf PP/medic/Continue; and actual wild encounters with RUN. Camera and
geometry fixtures use explicit local empty parties. The established battle and
wild cases use separate setup parties/badges/divider. These prove behavior and
persistence, not earned regional access or battle balance.

The actual stair-course save passes 13 fresh-process production
stdio calls: Continue, walking up onto cave36, Start/Pack, Items/Balls and return
to walking. Stderr is empty. Earlier full-route client traces remain historical evidence. JSONL request/response traces and saves remain retained.

Complete source views, authentic Crystal Route45 and the cave tile atlas were
visually inspected. All six final native2Dcaptures in
`planning/minnesota/renders/iron-cliffs-review-v01/native-v03/` match every pixel
and were visually reviewed. The southern clipped-chunk corner uses the native
compositor's two-hop source context. Earlier one-hop mismatches are retained.
No3Dpreview was produced.

`preservation-v03.json` proves residual equality and rejects five mutations.
`MN_R09.after-v08.layout.json` and `MN_R09.dossier-v08.json` are the accepted
sources; canonical R09 files match them. The bounded Rust publisher is
`target/route-grass/wasm-dev/revise_iron_cliff_stairs` with arguments PACK BEFORE
AFTER DOSSIER FRESH_OUTPUT. All pack writes are Rust-only.

Affected-crate Clippy passes with pre-existing warnings. Scoped Rust formatting
passes. Whole-workspace formatting still reports unrelated existing differences,
retained in `.local/route-grass-review/iron-cliffs-workspace-format-v03.log`.
Native log: `.local/route-grass-review/iron-cliffs-controller-v05.log`.

Iron Range Descent/R10 is the next separate area; this accepts no terrain there.
The broader route/park/town and final current-story review remains open.
