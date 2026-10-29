# Grand Marais partner delivery

The selected starter-return parent still has an unchecked native `givepoke`
inside `MnRewardMachinePartOriginal`. If both the party and selected PC box
are full, native GivePoke returns 2 without delivering Birchit. The old script
continues granting supplies and sets `EVENT_MN_GRAND_MARAIS_STARTER`, consuming
the offer despite the failed partner delivery.

`build_grand_marais_partner_retry` inserts a result-2 branch immediately after
that existing gift command. It preserves successful party delivery and native
selected-box delivery. Failure explains the capacity problem and leaves the
welcome flag and supplies untouched. The existing Machine Part recovery wrapper
and lighthouse repair remain separate.

Selected as `grand_marais_partner_retry_revision`, a direct child of the accepted
Mille Lacs grass revision. Pack:
`content-packs/minnesota-current-marais-partner-mille-v02.crystalpack`.
SHA-256: `b3112da0db3ce6d254a2a3be726f1fc2dcb30796cb49d7693f8f0fe7e5278fc2`.
Identity: `6bc7fd50d36cea546df1bf25e83cf59834be734d46bce1ebc97bb3d1c758f4a5`.
Publisher: `pack_grand_marais_partner_retry PARENT OUTPUT`.
Controller regression:
`marais_partner_controller_full_party_retains_offer_and_success_survives_continue`
with `CRYSTAL_WINONA_TEST_PACK`. It declares legal Minnesota party/PC fixtures;
subsequent gifts, dialogue, walking, hand-in and Continue use production input.
Independent comparison:
`planning/minnesota/tools/audit_marais_partner_preservation.py PARENT CHILD`.

Acceptance requires party and PC delivery, failure without lost offer/supplies,
repeat/Continue preservation, actual station-to-lighthouse hand-in and unchanged
unrelated map/content/audio data. No complete earned regional access is implied.

The default-profile controller regression passes all three party/storage branches
and the actual station-to-lighthouse trip, Machine Part consumption, repair,
reward and Continue. Strict parent comparison preserves all729 other maps,
geometry, actors, encounters, teams, items, constants, art and audio. Twenty
production stdio calls pass refusal, dialogue input ownership, returned movement
and Start menu with empty stderr. Assets/Bevy all-target Clippy passes with the
four baseline allowances. Scoped rustfmt passes; whole-workspace fmt fails at
the pre-existing offset-allocator formatting difference.

The full-storage branch proves the claim and supplies remain untouched before
and after Continue. It does not free storage through PC menus and return to
retry. Fixtures do not prove earned regional access or full campaign completion.
