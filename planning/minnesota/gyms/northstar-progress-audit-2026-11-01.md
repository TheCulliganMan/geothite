# Northstar Gym progression review

This review uses `content-packs/minnesota-story-review-fixes-v09.crystalpack`
and the unselected `content-packs/rochester-gym-progress-review.crystalpack`
(content identity `750ee199d652ead7f551752b8b597c08d6999dfca025087331b51ed1d5f257c1`).
The candidate predates the final shortened guidance lines in source; rebuild and
retest it before integration. Neither the selected pack nor a player save was
changed. The candidate needs an
explicit save-identity migration and release integration before existing saves
can use it.

Rochester's Elvik prerequisite previously jumped to the same denial as the
leader's parcel requirement; the installed text sent the player to the clinic.
The repair directs the player to Tavel. Varel checks the two actual trainer
victory events and identifies the first missing win. This also avoids relying on
a stale derived court flag, without granting either victory. The parcel check,
parties, badge and TM remain in place.

The installed scripts for all eight gyms have post-battle map-command metadata
for every trainer and leader, then set their victory events. Leaders retain the
prior badge (or Rochester delivery), issue the next badge on victory, and mark a
TM received only after the item is successfully delivered. Each Gym's exit is
bound to its corresponding city, with a matching city entrance and a valid
return target in the installed warp records. Other Gyms' court flags also control physical
puzzles and were not bypassed. All 26 Gym trainer and leader parties use
species in the active Minnesota roster after retired families and removed
evolutions are excluded.

The pre-existing `crystal_bevy` production-controller test executable ran
`minnesota_gyms_controller_earns_all_eight_native_puzzles_badges_tms_and_continue`
against the candidate: all eight courses, trainer battles, leader victories,
badges, TMs, repeat interactions and save/continue checks passed. These are
separate room fixtures with a strong test party and prior badges, not a continuous
regional journey or balance test. The separate `rochester_earned_starter_route_trainers_first_badge_and_continue`
controller test also passed from the opening with an ordinary starter, earning
both Gym trainer victories, Varel's battle, the first badge and Continue. The
Rochester planter-maze controller test passed both city exit carpets and re-entry. Parcel refusal, leader loss, trainer
loss, skipped-court refusal and full-TM-pocket retry passed on the candidate.
The seven separate prior-badge refusal tests for Gyms 2–8 also passed.
The pack comparison found changes only to Rochester's map and its script index;
all other map data, catalogs, audio and runtime files matched the parent.

The older all-city aggregate test starts at obsolete city coordinates. The
current-room entrance test selects newer room geometry absent from this pack.
Neither is evidence that an actual door is blocked. A new canonical-room test
uses the installed warp records and is pending a compatible test build. The
new Rochester regression also tests the specific stale-court-flag save state and
trainer-specific dialogue; it is pending that build. The user's browser session
was not available for save inspection and no claim is made about which flags it
contains.
