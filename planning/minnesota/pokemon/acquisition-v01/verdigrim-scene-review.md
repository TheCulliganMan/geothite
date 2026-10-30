# Verdigrim current-world scene review

The current-world candidate passes its twenty-step verification pipeline and
has a reviewed native 2D intake capture. It remains an **unselected review
candidate**, not a completed campaign or completed Pokédex.

- Pack: `content-packs/minnesota-current-verdigrim-events-v01.crystalpack`.
- SHA-256: `5c804d904ff857d7e1423a1f99f595114ab678c7d60c380d232d7edc99efa17b`.
- Contents: 730 maps, 476 total species including all225 active Minnesota
  species, 272 moves and 602 audio programs.
- The local selector still names `duluth_cliff_stairs_revision`; this work has
  not changed a selected or hosted game.

## Actual current-world lineage

`compose_current_minnesota_release` combines the accepted Duluth world,
its Marais/Mille common ancestor, and the published story/species/art/audio
source. Its input identities are bound by
`planning/minnesota/revisions/verdigrim-shared-events-v01/release-composition.json`.
The accepted branch owns outdoor geometry and spatial mirrors. Other disjoint
story/assets changes merge strictly; conflicting values fail. The composer
keeps binary payloads as byte maps and does not migrate saves or grant progress.

The actual composition and readback pass. The independent audit preserves
180 accepted exterior geometries, their spatial mirrors and 377 original audio
metadata entries, and rejects four terrain/art/species/audio corruptions. All
four self-contained composer cases pass. Its measured publication peak was
1490096 KiB on the shared host.

Three successor publications then pass:

1. `align_northstar_league_entrance` retains canonical0/0 camp/road offsets and
   changes one native R13 floor metatile to connect both clearance lanes. Its
   preservation audit passes with unchanged scripts, species, art and audio.
2. `refresh_northstar_rescue_evidence` preserves728 other complete maps and
   existing event identities. Ellis gives directions at home; the physical
   office desk copy grants the evidence only after acknowledgment. Its
   independent audit rejects four corruptions.
3. `pack_verdigrim_scene` modifies only the existing intake scene. All729 other
   complete maps, all existing runtime files, species and audio stay exact.
   Seven private native art files, eleven flags and two native water-animation
   clips are added. Its independent preservation audit passes.

Strict225 species/Dex membership and the completion threshold pass. All602 PCM
hashes, frame counts and loop ranges pass on this exact final candidate.

## Native gameplay verification

All six current-world Verdigrim cases pass:

- Separate controls, refusal, order and acknowledged crew actions.
- Exposure, cry, green water, dialogue ownership and Continue.
- Full-party capture into the selected PC box and Continue.
- Actual knockout, retreat, hidden Continue and physical retry.
- Actual blackout recovery with uncaught/containment state retained.
- Native water frames move and recolor while the original floor stays static.

The League entrance case passes both lanes, all four requirement denials,
clearance, return and Continue.

The obsolete two-floor rescue replay correctly encountered the expanded
Operations floor instead of jumping straight from cargo to relay. Its actually
reached same-pack cargo-power-off save was retained. A new continuation from
that save passes the expanded headquarters, card access, eight panel pairs,
six doors, optional rooms, rival/enforcer battles, native nurse/shop recovery,
rescue, the witness's home and the acknowledged physical desk copy. It preserves
the party, spent resources and flags across that continuation. The original
party/badges were declared chapter fixtures; this is not an earned campaign.

Both required original-core controller cases pass, including actual battle PP
mutation and the Mom/New Bark script/input contract. Production Start-menu MCP
and clean-stdio regressions pass. Eighteen current-scene production MCP calls
pass movement, dialogue ownership, menu and Continue with empty stderr.

Both affected crates pass all-target Clippy with the recorded baseline
allowances for `never_loop`, `bad_bit_mask`, `approx_constant` and
`useless_conversion`. Existing repository warnings remain; none is reported in
the new shared headquarters continuation or native city-port helper. Scoped
Rust formatting and whitespace checks pass. Whole-workspace formatting has
unrelated baseline failures and must not trigger a broad formatting rewrite.

## Native presentation review

The fresh headless production capture loads the actually reached held-bypass
save. The original gray floor remains readable, the native intake glyphs remain
intact, and blue/green water distinguishes the controlled flows. Actors and the
held two-line dialogue use the native presentation. This accepts one actual
intake view; it does not accept every regional map's visuals.

Ignored local evidence:

- `.local/verdigrim-final-native-ready-v01.json` and its referenced step logs.
- `.local/verdigrim-final-current-native-v01.png`.
- `.local/verdigrim-final-current-saves-v01`.
- `.local/verdigrim-final-expanded-hq-saves-v01`.
- `.local/verdigrim-current-story-cast-v01.log`.

These artifacts are not shipped or embedded as compile-time fixtures. The cast
readback verifies all24 authored grunts, six motivated rival battles, the four
Elite Four members and Champion Casey with active Minnesota parties. All eight
HQ panel pairs, four stair pairs and six card doors match their source graph.
Those static bindings do not prove campaign reachability. Three separate native
headquarters gauntlets now pass with Slipperina, Rimepelt and Portabeau starter
parties at levels22–23, normal DVs, four Potions and two prior badges. Battles,
nurse visits, purchases, rescue and Continue use production input after fixture
setup. Individual saves and the exact pack/binary identities are retained in
`.local/ice-hq-starter-balance-v01/receipt.json`. These are intended-level chapter
balance checks; they do not establish an earned full campaign.

## Remaining comprehensive-game acceptance

The other rare-family current-world acquisition candidates still need their
publication/controller, storage, recovery, production MCP and visual gates.
The full stack then needs integration, a current-selector/parent recheck, an
unboosted same-pack campaign through Hall of Fame, earned postgame access,
key-item/HM/reward closure, and actual acquisition/evolution/Dex acceptance.
Chapter fixtures and permissive acquisition upper bounds cannot replace these.

Earlier prototype and released-parent probes are historical evidence only. The
released donor's1/-1 Boundary offsets broke compact outdoor placement and must
not replace the accepted world. Failed attempts and source/render histories
remain preserved in ignored verification output. Do not disable geography
validation, restore old terrain, or rewrite a save identity to bypass a failure.
