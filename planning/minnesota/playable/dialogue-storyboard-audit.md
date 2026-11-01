# Full campaign dialogue storyboard audit

The review reads the required campaign through the ending, its refusal and return
branches, and surrounding NPC directions that can contradict it. Page boards use
the content pack's native font and frame. They are writing and page-layout
artifacts; actual controller captures, sound playback and earned play remain
separate acceptance evidence.

The subsequent [regional travel audit](travel-gate-overhaul.md) found 23 paired
officer checkpoints in both the selected pack and v27. Most are public wardens,
but their repeated appearance and permission checks undermine the story's
pacing. That document revises the access design; revised travel is not yet
implemented or published by the dialogue candidate.

## What the boards exposed

| Scene | Defect | Correction |
| --- | --- | --- |
| Mom / Winona exit | Both directions called Tamarack “his.” | Use “her,” consistent with the accepted older-woman design. |
| Bedroom cousin | A nearby child gave a long, disconnected speech about a high score and stories that are not on the map. | Identify the speaker as the cousin, give the Game Boy a concrete context, and suggest saying goodbye to Mom in two short pages. Preserve the ordinary nearby interaction and zero rewards. |
| Installed cast | Native captures still showed stock Elm as Tamarack and the child Rival as Reed. The accepted cast art had never been bound into this regional lineage. | Integrate Tamarack's accepted older-woman field art and Reed's accepted adult battle art plus repaired field poses through private classes. Preserve all nine Reed appearances, six battle identities, teams and unrelated stock-art users. |
| Field station | The professor listed names and readings before explaining the favor. | Establish river research, Ivy's clinic, water-level notes, the envelope, and the reason for a traveling partner. |
| Reed's arrival | His jealousy about receiving a starter read like a child's rivalry, contradicting his adult design and later employment plot. | He comes looking for paid escort work; his team, egg case, battle and move toward ICE follow that purpose. |
| First Reed result | “For once, you got lucky” presupposed earlier meetings. | Give the first battle its own result lines. |
| Rochester | Narration and Ivy's voice ran together; the next missing person appeared without a clear relationship. | Show the envelope changing hands, resume with Ivy identified, introduce Ellis and Jo, and make the request specific. |
| Route 01 farmhouse | The farmer sent the player to Tamarack at Rochester's clinic. | Ivy works at the clinic. Keep the bluff, grass and berry advice local. |
| Minneapolis | A key, powered latch and guarded controls gave contradictory explanations for one prison. | Describe the powered holding latch and downstairs controls consistently. Keep battle, shutdown and physical release separate. |
| Dispatch evidence | Papers were described as both a locked drawer and a copy hidden under the desk. | Use the physical desk copy throughout; Ellis gives directions and the desk grants the evidence after acknowledgment. |
| St. Paul | Mina introduced her job twice in consecutive blocks; Reed's cargo claims conflicted with her authority. | Introduce Mina once, explain the unsigned rerouting and distinguish the first shipment from the next held load. |
| Brainerd / Itasca | “Independent reference” and sealed readings had no practical explanation. | Identify a public dated water-level log from before the pump tests, show copying it and direct the player to the officer. |
| Bemidji | “Separate basins” was unexplained technical language. | Visibly compare daily levels, repeated numbers and the matching ink blot; explain why the meter is the next useful evidence. |
| Mine | Test orders and crew danger repeated without explaining Reed's involvement. | He knowingly guards a contract while Finn warns about the drill; shutdown and meter recovery remain distinct. |
| Grand Rapids | The technician jumped from missing equipment to a new destination. | Read the test sequence, explain the falling water, keep the meter and explicitly give a log copy for the harbor engineer. |
| Duluth harbor | The engineer announced the whole scheme without showing how the log established it. | Connect drain tests to Lode's plans, then the intake cage and Kettlurk's current; separate evidence, reaction and directions. |
| Boundary intake | Winning the battle could sound like stopping the operation. | Reed guards the switch; motor shutdown, slowing current, opening gates, escape and crew response are separate beats. |
| Grand Marais student | The student directed the player to a duplicate Tamarack in the north. | Give the student a local observation about the broken lamp and a useful station direction. |
| Grand Marais station | An old demo opening gave the male scientist Tamarack's identity and restarted the adventure verbally. | Identify the existing actor as the local researcher, explain the lamp part and describe Birchit as a field-team gift. Preserve existing progression identities. |
| Lighthouse aftermath | Repairing a lamp inexplicably secured “first survey readings.” | Acknowledge the repaired lamp and the actual part delivery. |
| Shore / winter directions | “Harbor signal” referred ambiguously to both a local lamp repair and the investigation's crew alert. | Name the local lamp and Duluth crew alert separately. |
| Returns / League | Advice could repeat an already completed job or declare the damaged habitat repaired. | Track evidence custody, radio contact and shutdown. Preserve the quiet personal returns and acknowledge remaining shore damage. |

Tamarack's accepted design is an older woman in
[her character record](../npcs/tamarack-ice/01-tamarack/DESIGN.md). Reed is an adult in
[his character record](../npcs/cast/01-reed/DESIGN.md). Historical script labels
retain their identities; those labels do not establish a second professor.

## Current full storyboard

The v27 candidate contains **481 conversations / 1,170 authored pages across
56 maps**, rendered as 79 native-font sheets. Every page was compared with the
installed pack's actual text program. All sheets were visually reviewed: the
v24 review inherited 59 reviewed v23 sheets and inspected all 19 changed/new
sheets; v25 retains 71 identical v24 sheets and its seven changed sheets were
inspected again. V26 retains that reviewed set; v27 retains its 78 sheets and
adds the inspected cousin sheet. Their programs are checked against the actual
v27 pack. The chapter viewer also includes 16 acted native 2D examples, with
their pack, fixture and renderer provenance stated. The chapter viewer is
`.local/dialogue-overhaul/storyboard-review-v27/index.html`; the complete page
gallery is `.local/dialogue-overhaul/storyboards-v27/index.html`. The local Rust
review server also serves the chapter viewer at
`http://127.0.0.1:3038/storyboard-review-v27/`; its harbor chapter was inspected
in the browser. Page boards remain distinct from live browser gameplay.

The second sweep examined 166 additional headquarters, mine, depot, shelter and
intake text labels. Four obsolete shelter texts are unreachable even in an
intentionally broad script-reference graph; their old remote-shutdown narration
is retained as historical data, not installed as current canonical dialogue.
The other 162 labels now share the canonical authoring and overflow checks.

A separate scan outside that canonical set reviewed 12 text programs naming the
main cast and 34 mentioning investigation terms across Minnesota. These include
direction signs, optional activities, local flavor, postgame rewards and reeds
as plants. The two old mine-crate refusal texts have no reverse reference path
to an installed actor; the actual crate uses the later drill/meter script. They
are retained historical data, alongside the separately proven shelter orphans.
No additional required-handoff contradiction was found in this scan. This is
text coverage evidence, rather than a claim that every optional activity was
played.

| Additional defect | Correction |
| --- | --- |
| Headquarters staff | Mechanical wrapping split thoughts across pages. | Deliberate pages introduce the records room, card, staff rooms and return route. |
| Ollie | Offered unspecified “these.” | Explicitly offers two Super Potions; preserves acknowledgment, full-pocket refusal and once-only receipt. |
| Nora | Treatment dialogue did not clearly ask the existing yes/no question. | Asks whether to treat the player's Pokémon; acknowledgment matches actual healing. |
| Tess | Said she was leaving, then immediately said she would wait; remained after the rescue. | Explains the guarded stair, then leaves offscreen when the existing acted rescue event completes. Actual rescue, physical reentry, walkable former tile and Continue checks pass on v27. |
| Mine guidance | Several pressure-line terms and routes ran together. | Ruth and Vern distinguish both valves, the sump, crosscut and service ladders. |
| Guards | Many unrelated people shared the same generic confession. | Encounter-specific challenge, loss and return replies. |
| Intake guard | Claimed the player came alone while allies were present. | Responds to the intake and shore crews without denying visible actors. |
| Pump guard | Referred to a meter the player had not received. | Refers to the company's ban on outside testing. |
| Drill guard | Claimed the crew was kept outside while crew actors stood inside. | Guards the drill controls and switch. |
| Relay patrol | Talked about a radio call in the power-control room. | Refers to that room's relay panel. |
| Enforcer | Could claim the latch was still inaccessible after its prerequisites were complete. | Uses a defeated reaction without inventing a remaining lock condition. |
| Depot travel | Clerk and timetable promised rides while the actual conductor/service remains closed. | All three describe the installed closed passenger service. No travel feature is enabled. |
| Browser restore explanation | A rejected native checkpoint expanded into thousands of internal profile identifiers. | Display a short matching-game explanation; retain the original strict save validation. |

The canonical main 318-conversation rewrite is preserved while these surrounding
conversations are brought into the same review. The accepted female Tamarack and
adult Reed art remain bound to all ten existing cast actors and six Reed battles.
A native capture from the genuine New Game egg exchange now shows Reed's actual
adult trainer introduction before the battle; that same controller then plays
the battle normally. Resource loading alone was not counted as visual acceptance.

## Executed evidence and open acceptance

The v23 scene matrix passed all three fresh New Games through naming, Reed's
actual battle and Rochester delivery; Mom and depot continuous/diagonal input;
all four lab and ten depot approaches; refusals, loss/retry and Continue; physical
desk-copy and gray-fence prerequisites; actor positions/facing and reward timing;
officer comparison and return states; native text delays, held input, sound
ordering and choice/battle boundaries; original core controller, production MCP
and byte-empty terminal stderr regressions. Its staged 2D captures were inspected
through the harbor, mine, intake consequences and postgame Continue.

The explicitly declared v23 late-story course passed the connected mine, meter
custody, harbor, radio, separate intake battle/shutdown/release, all League
battles, Hall of Fame autosave, credits and actual postgame Continue. Its initial
five-badge, level-65 party fixture is not an earned New Game or balance proof.

The separate earned v23 New Game course retains its real headquarters loss,
blackout money and already-earned card/guard victories. Its same-pack recovery
uses native care, shops and carried partners. The replay helper now preserves
completed recruitment/cargo/card actions rather than requiring a pristine first
visit. No levels, party, HP, money, progression flags, position or RNG are supplied.
That course earned five badges and reached Lake training, including a native
partner evolution. Its older driver then failed a road-step assertion after
outing 34. The saved native checkpoint is retained. The current driver replans
from observed movement and permits ordinary carried-partner switch training;
same-pack continuation reached the mine, then failed a battle-helper assertion.
Its native saves are retained and are not relabeled for newer candidates. The
earned ending remains unverified.

V26 source publication/readback, the complete scene/frame/core matrix and all
three fresh starter deliveries pass, including the actual Tess rescue/reentry
case. The original-builder preservation check also passes after teaching its
expectation about canonical ownership. V27 Rust publication/readback and every
installed-program storyboard comparison pass. Its production MCP New Game
cousin check verifies both pages, movement ownership, unchanged party/badges and
byte-empty stderr. The fresh production executable also passes the complete
v27 opening MCP regression. Exact-v27 cousin and Tess before/after captures from
the fresh native renderer were visually inspected. The complete current v27
scene/frame/core matrix passes: all three fresh starter deliveries, every
authored arrival, continuous/diagonal interception, defeat/retry, reward timing,
physical evidence and fence release, progress-aware returns, Save/Continue,
paragraph/scroll/pause boundaries, held input, sound dispatch and battle PP.
The source records are `storyboard-integration-v27-v02.log` and its scoped
compile-fix continuation `storyboard-integration-v27-v03.log`; the complete
matrix is `storyboard-controller-matrix-v27-v02.log` under the ignored evidence
directory. The exact-v27 earned New Game course supplied no progression or party;
it earned two badges, then stopped on a Minneapolis Mart routing assertion. Its
native checkpoints are retained. It has not completed the ending.

Independent checks preserve all 7,836 parent runtime files, original catalogs,
encounters, audio manifest, warps and current geography, apart from the previously
reviewed gray fence and one League floor cell. All 377 original PCM/hash/frame/loop
checks pass on v27, whose fresh source publisher reproduces its candidate byte
for byte. An independent comparison confirms that v27 changes only the cousin's
two-page conversation and its native final-page acknowledgment relative to v26.
Its selected-parent, frozen-campaign and original-composer preservation checks
also pass, along with current assets Clippy and fresh controller compilation.
The latest driver Bevy Clippy stopped on a shared-checkout unresolved
`international_falls_journey` test module; that follow-up pass remains open.
A concurrent fishing-module compile error interrupted the first attempt;
that module was corrected in the shared checkout, and the failed tail was resumed
without rerunning or discarding passing publication/preservation checks.

The Rust-only 2D browser build passed. Its local v27 review reuses that completed
frontend for the subsequent data-only changes, with two byte-exact v27 pack
inputs. The hidden browser cold-starts in 2D, exposes only 2D/TUI navigation,
unlocks sound and renders the native title with no observed warning/error logs.
The earlier v25 browser also reached the native clock introduction. Importing
the native harbor checkpoint was correctly rejected: the browser adds
`realtime-clock` and `player-customization` runtime profiles. The checkpoint was
not relabeled. Its error had exposed the full internal identity chain; the thin
browser integration now displays a concise explanation, verified through the
actual restore UI. The later-chapter live browser review remains open. These
checks are not a browser campaign or subjective listening review. Normal-speed recordings and actual
native/browser audible listening remain open. Nonzero audio samples and an
unlocked sound control do not establish that listening pass.

The unselected current candidate is
`content-packs/minnesota-dialogue-overhaul-v27.crystalpack`, identity
`ccf84221d047e958736a4ca5cbecc917713a4496e03ffe89527e72e2d2593914`,
SHA256 `35a585671a8ac174961c1d3aa2b10b329fb67a1bb3d89534d58fae9f290b69c2`.
It extends the reviewed Minneapolis citywide-cycling descendant, preserving the
intervening Duluth, St. Cloud and Bemidji repairs. This work has not changed the
selected world. Final promotion still requires the complete acceptance course
and a fresh check of the then-selected lineage. Packs, saves, images, transcripts
and verification logs remain ignored. Workspace formatting still has unrelated
pre-existing failures beginning in `crates/offset-allocator`; owned leaf modules
are formatted. Runtime and Bevy all-target Clippy pass with existing workspace
warnings; the latest test-driver correction has a follow-up Clippy queued.
Remaining checks are not assumed.

The original-builder preservation check initially expected the old volunteer
flavor text after that label moved into canonical ownership. It now checks owned
labels against compiled canonical pages and requires exact parent text for every
unowned label; that v26 result passes. V27 extends the same expectation to the
cousin's existing conversation program, retaining every other command and actor.
That v27 original-composer preservation check also passes.
