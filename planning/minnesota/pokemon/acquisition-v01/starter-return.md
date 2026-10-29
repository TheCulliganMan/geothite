# Kindlit research partner

The Winona field-station aide offers a level-5 KINDLIT after the regional
Champion victory. The existing research dialogue and rewards run first.
This closes the Fire starter family's choice lock for players who chose
POUCHID or BIRCHIT. It does not increase the structural species upper bound.

Declining leaves the offer available. A full party leaves the claim available;
successful native delivery sets the once-only claim. Repeated interaction and
Save/Continue preserve the partner and its caught Pokédex entry.

Selected review pack (`starter_return_revision` in `world.json`):
`content-packs/minnesota-current-starter-return-ely-v01.crystalpack`.
Its parent is `route_grass_ely_shore_pass_revision` in `world.json`.
Candidate SHA-256:
`2576f58812e8b475677b083fe6230beaac77d2854004adf6c5eec1c34a75cb09`.
Content identity:
`93ba090b97aa89e41119c993a38f4b8500f58f3151eb3c1959b3425b39a59444`.

The default-profile production-controller regression passes pre-League denial,
decline, full party, successful delivery, caught entry, Continue and duplicate
prevention on this candidate. Fixtures declare their Champion state and party;
this is not an earned League journey. The independent preservation audit finds
all 729 other maps exact, with terrain, warps, actor positions, original research
programs, encounters, teams, items, art and audio unchanged. Only the existing
aide's script binding is wrapped. Assets and Bevy all-target Clippy pass with
the four documented pre-existing lint allowances.

The current Ely-parent candidate also passes 63 production stdio calls through
real full-party refusal dialogue with empty stderr. The current source roster has since advanced to 225 species with VERDIGRIM.
Nine families now lack acquisition sources in this selected pack; earned full
collection and the new species integration remain open.

Builder: `pack_minnesota_starter_return PARENT OUTPUT` in `crystal-assets`.
Regression: `starter_return_controller_league_guard_full_party_and_once_after_continue`,
with `CRYSTAL_WINONA_TEST_PACK` naming the candidate and `--ignored`.
Independent comparison: `planning/minnesota/tools/audit_starter_return_preservation.py PARENT CHILD`.
