# Minnesota regional trainer expansion

Goal: add ten more regular trainer types, distribute the expanded cast around
Minnesota, replace current generic trainer appearances with regional roles,
and repair older custom overworlds that do not match normal Crystal proportions.

The initial authoritative parent is the selected Iron Cliff Stairs revision,
`content-packs/minnesota-current-iron-cliffs-v02.crystalpack` (SHA256
`9181eba4c44d85e840e99d29c6284ecaebe6aa099ccb373d6c703364ff205e98`).
Initial inspection found 316 Minnesota trainer records, 307 trainer IDs directly
connected to field actors, 342 Minnesota maps and no installed private regional
trainer classes. Earlier art packs remain review branches. They are sources for
this integration, not proof that the current game already uses their art.

Use normal Crystal six-frame bases for field proportions. Keep tiny equipment
out of 16×16 sprites. Preserve the accepted woman Tamarack and mean paramilitary
ICE direction; leadership retains the olive greatcoat/haircut reference.
Existing named story roles, teams, progression, dialogue and rewards stay intact.
Only active Minnesota species and their active evolution families may be used.

The older 36-set field gallery was inspected at native size and 3×. Many custom
silhouettes use small heads, long bodies, pinched profiles or noisy accessories;
these will receive stock Crystal field bases while retaining their battle art
and palettes. The latest eight regular types and Tamarack/ICE already use those
normal bases; retain them unless a specific visual defect is found.

| New type | Battle identity | Status |
| --- | --- | --- |
| Snowplow Driver | ROWENA, sturdy middle-aged woman, orange winter jacket, knit cap | Native v4 and intermediate encounter accepted |
| Rail Worker | HOBB, blue workwear, cap and lantern | Native v2 and intermediate encounter accepted |
| Lake Diver | NAIA, blue wetsuit, broad mask and fins | Native v5 and intermediate encounter accepted |
| Disc Golfer | FINN, sporty teen, green shirt and one disc | Native v4 and intermediate encounter accepted |
| Prairie Painter | ALDEN, older artist, burgundy smock and broad brush | Native v2 and intermediate encounter accepted |
| Conservation Officer | WREN, green outdoor uniform and field notebook | Native v2 and intermediate encounter accepted |
| Transit Driver | OTIS, blue jacket, cap and ticket pouch | Native v2 and intermediate encounter accepted |
| Sauna Goer | HILDA, lavender robe, towel and sandals | Native v2 and intermediate encounter accepted |
| Rock Hound | KIT, young agate collector, cap and one broad stone | Native v2 and intermediate encounter accepted |
| Snowmobile Rider | INGA, red helmet, winter suit and mittens | Native v2 and intermediate encounter accepted |

Each type must pass native/reference comparison, its actual encounter and
production-controller battle/save/repeat checks before advancing to the next.
The final integration needs region-wide ID/actor/art/party audits, current-pack
gameplay, actual rendered review, native-only presentation and audio preservation.
No implementation or final publication pass is claimed by this status record.

All ten new types now have their own private class, matched native field/battle
art, a real additional encounter and an intermediate production-controller pass.
This does not complete the regional objective. The next stage binds the older
31 regular type assets onto existing ordinary trainer identities, repairs their
older custom field banks as needed, and distributes the full 41-type pool.
Existing trainer names, parties, AI, positions, scripts, progression and rewards
are preserved. Historical unselected review NPC placements remain historical
sources; this pass uses the current world's existing encounters. Named story
art gets exact variant/actor bindings separately. Final parent, preservation,
rendered review and combined encounter checks still remain.

Older field repair progress: Northwoods Ranger native v5 accepted after a real
MN_R02_T01 encounter (existing TESS) on the intermediate ranger pack. Canoe Guide
native v2 retains the original Fisher head/poses with simple life-vest panels;
its existing Mississippi trainer encounter is running. Thirty-six ordinary
opponents use scripted trainer battles (18 Minneapolis cycle NPCs and 18 Gym
aides), so the regional swap audit must include those as well as 227 direct
trainer actors: 263 current ordinary opponents in total, plus the ten new ones.

First four older type repairs accepted on their current encounters: Ranger/TESS,
Canoe Guide/WADE, Miner/MINA and Botanist/HUGO. Each completed actual PP-spending
battle, legal victory, repeat/dialogue ownership and save/load. The fresh Rust
batch publisher built and published these stages; the distribution story-role
unit guard passed. Full workspace fmt check reports pre-existing offset-allocator
formatting differences; changed Rust files are individually formatted.

Angler/LENA and Courier/IVAR repairs passed; Courier field skin tone now matches
his dark skin and fur-trimmed outfit. Hockey Captain native v3 is in its current
JUNE encounter check. All 326 Minnesota trainer records in the ten-new-type pack
use active roster species. Scaled evolution closure is still a final audit item.
`cargo clippy --locked -p crystal-assets --all-targets` completed successfully
with existing warnings; no warning points at the changed distribution or batch
publisher files. Fresh client/controller test builds have started in the isolated
regional trainer build cache.

Hockey Captain/JUNE and Orchard Keeper/VERA repairs passed their production
encounters; Wild Rice Harvester/HOLLY is running. The current production client
built successfully. Controller and client test binaries compiled successfully;
the controller is being rebuilt once more to include the newly added opt-in
clear-field capture after closing repeat dialogue. The regional authoring pack
still uses the frozen Iron Cliff parent; current selected map work has advanced
to Rainy River frostgrass and must be preserved in the final rebase.

The final distribution preservation test now also calls the actual production
team-scaling function at every supported ace level through 100 for every
Minnesota trainer. It must validate all resulting species against the current
active roster minus retired families, including evolved teams and learnsets.
This extended ignored-fixture check is pending the installed full style pool.

Repairs through State Fair Baker/CORA are accepted (12 older types); Storm
Watcher/EDISON is in its encounter test. The current-source controller now
produces complete clear-field captures after closing repeat dialogue, and those
views have been inspected for Mechanic/ESKER, Photographer/LARK and Baker/CORA.
Bevy Clippy passed on retry after concurrent seasonal palette source was fixed.

Storm Watcher/EDISON accepted; Fossil Researcher/ADA is in the actual encounter
check. The expanded scaling test compiles. Both affected-crate Clippy commands
have passed on the current source, with existing warnings and no warning at
changed files. The extended preservation/scaling fixture still awaits all
installed styles and the final distribution source.

Night Sky Observer/BERT and Cycle Tourist/DANA repairs accepted, bringing the
older regular field repairs to 16. Beekeeper/NELL is in the actual encounter
check. All four required original-controller/client/clean-stdio regressions
passed using the current built binaries and completed external TUI pack.
The original female Tamarack and Bovino-referenced olive-coat ICE leadership
assets remain the accepted source for the upcoming current-world binding.

Regular repairs through Volunteer/IVY accepted (22 types); Ski Patroller/ELSA
is in its encounter check, with her existing dark ponytail retained. All ten
new encounters were rebased onto the selected Rainy River frostgrass parent.
Eight already-normal earlier types retain exact accepted pixels and now have
current-world bindings; their final combined runtime checks remain pending.
All four Tamarack/ICE bindings match the current 18 actors and 17 ICE battle
variants. Reed binding now targets all six real rival variants and all nine
current field appearances, rather than the historical street review fixture.

All 23 older regular repairs are accepted. The current-parent base now contains
all 41 regular styles, female Tamarack and all 17 mean paramilitary ICE variants.
The regional distribution has published 273 ordinary encounters (263 existing
plus ten new), with every one of 41 styles appearing at least four times.
The full preservation/actual-production scaling test is running. Named repairs
Reed and Varel passed their actual encounters; Torven is in his native Gym test.
Reed binds all six existing rival variants and all nine current appearances;
final actual scene-variant checks remain pending.

Named repairs Reed, Varel, Torven, Avrin, Brannic, Saven, Sivren, Orlis and
Norek passed current encounters. Fern is running. All prior battle pixels and
palettes remain exact; field bases use real complete Crystal walking banks.
Captain, Karen and Will references have empty walking slots, so they cannot
be used as six-pose walking banks. The final selected parent advanced to State
Fair Buildings; final publication must preserve that revision (its outdoor
actors and walking collision are exact). The full distribution test initially
failed an incorrect test assumption that automatic scaling was enabled; the
selected world has authored levels. That assertion was corrected without
changing the setting, and fresh testing is queued behind other Cargo builds.

All 36 older overworld repairs passed intermediate encounters. The first
combined candidate was rejected before final acceptance because its regional
field swap would remove the 18 Minneapolis bicycles. Cycle Tourist binding v2
now restyles their battles while retaining exact original bike fields/patrols.
Distribution v2 updates 255 field/battle pairs; those plus the 18 battle-only
cycling bindings cover all 273 ordinary opponents. All 41 types remain used at
least four times. The revised candidate is publishing; 82 final controller
cases include the original cycling turn/Continue, victory-return, patrol and
decline courses plus a scripted Gym aide. The fresh Cargo verification has
started after the shared build queue cleared.
