# Route 1 fossil terrace access

The eastern passage mouth replaced the native outdoor stair at metatile
`(20, 9)`. Move that same stair to the adjacent cliff block `(19, 9)` and
retain the mouth at core `(40, 19)` and its return at `(40, 20)`. The stair's
walkable core tile `(39, 19)` joins the lower approach `(39, 20)` to the
terrace `(39, 18)`; the existing ledge and fossil hunter `(26, 14)` remain.

The Rust passage builder rejects an unexpected preimage and checks the new
collision. The whole Route 1 native-source-art composite was inspected, with
the stair and cave adjacent and visually distinct. This was not a production
client screenshot.

The ignored integrated review pack `target/northstar-fossil-stair-review-v01.crystalpack`
was assembled from `content-packs/minnesota-story-review-fixes-v09.crystalpack`
and `caves/geographic-audit-v01/r01-passage-draft.json`. Its SHA-256 is
`b63dd2d4e07782ff31306cae262b7401c02f022d4bd174019c771cabd9a651d5`.
On that pack, `fossil_hunter_cliff_conversation_and_route_remain_accessible`
passed using an explicit position and level-10 escort fixture, followed by
controller movement from below the hill, a real trainer victory, the scientist's
pages, refusal and Lens Fossil choice, both route exits, and Start. The passage
controller test also passed both cave directions and save/resume. The pack
verifier checked all 602 audio PCM hashes, frame counts and loop ranges. A
separate declared badge, party and specimen fixture passed both species'
restoration, badge denial, full-party retention and Continue.
The older `fossils_controller_both_choices_decline_persist_and_post_league_exchange`
test did not pass on this pack: it expected a choice but saw
`MnFossilDirectionsText`. The current research-access builder gates the second
specimen on restoration, whereas that test used post-League fixtures. Updated
restoration fixtures now pass both fossil choices, refusals, persistence and
the second-specimen exchange. The restoration flags are explicit setup, not
evidence of earning the first restoration on that walk.

`cargo fmt --all -- --check` and all-targets Clippy for `crystal-assets` and
`crystal-bevy` passed in the shared checkout. Clippy reported existing warnings
elsewhere in the workspace.

The fixture does not establish an earned opening or badge, and this review pack
is not selected or deployed. Reassemble and verify the final integrated pack
after concurrent changes; an existing installed pack does not change when the
builder source changes.
