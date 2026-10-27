# Installed major city expansions

The user's latest correction is **“Smaller but tighter.”** It supersedes the
earlier nine-times-area choice. Current implementation target is **64×48
metatiles per whole city**, four times the original area and less than half the
96×72 candidates. Preserve all earlier sources/renders as history. Goldenrod
and Saffron set the activity and composition benchmark: dense frontages, short
streets, useful courts and real destinations. Minneapolis, St Paul and Duluth retain their existing native map
identities, original room modules, gym/story scripts and indexed warp returns.
The current64×48 source is published in the660-map tight-final-v02 pack. Local
controller evidence and remaining visual limitations are recorded in the
[implementation runbook](../playable/major-cities.md); publication does not imply
the user's final visual acceptance. Canonical executable files are
`towns/Mn{Minneapolis,StPaul,Duluth}.layout.json` and matching
`.expansion.contract.json`. Earlier candidate sources/renders remain retained.

Root owns native publication. Each author supplies a layout plus an expansion
contract matching `authored_city_expansion_design.rs`. Keep the richer dossier,
activity plan and visual history too; the contract is executable input.

Required contract fields: `schema_version: 1`, `city` (native module name),
`native_module_sha256` (SHA256 of Python `json.dumps(module, sort_keys=True,
separators=(',', ':'), ensure_ascii=False).encode()`),
`old_dimensions_metatiles: [32,24]`, `dimensions_metatiles: [64,48]`,
`translation_core`, `arrival_core`, `clinic_constant`, `default_mart`,
`connections`. All coordinates are **16px gameplay tiles**, except dimensions.

Duluth currently has no native clinic recovery record. Its contract may declare
`clinic_recovery_core` at the existing clinic's safe floor arrival. Publication
appends one fresh recovery identity from that clinic's native metadata; existing
recovery records remain exact except explicitly moved exterior city spawns.

`anchor_overrides` records `kind` (`object`, `background`, `coordinate`, `warp`),
zero-based native vector `index`, exact old `before` and proposed absolute
`after`. Unlisted old records translate by `translation_core`. Old order, IDs,
programs, flags and warp indices remain intact. `spawn_overrides` records the
native `identifier`, `before`, `after`; other exterior spawns translate.

Each connection records `direction`, target native module `target`,
`old_offset`, `offset`, and two-lane arrays `town_edge` / `route_edge`. Offsets
use native metatiles. The reciprocal neighbor's offset changes; its terrain and
events remain exact. All outer walking/Surf edges must close except these lanes.

New `residents`, `signs`, `doors` use the existing town/route schema. New doors
append after every old warp, with separate passive return records. Actors with
`service: "pokemart"` require their own legal nonempty `stock`; each gets a
distinct mart catalog. `heal_party` uses the existing city clinic. Ordinary
actors use `service: null`. Do not invent placeholder quest services.

In the existing Door schema, `arrival_tile` is the **exterior** safe arrival
(equal to `return_tile`). Interior arrival is the room warp selected by
`target_warp`; it is not copied into that exterior field.

New `rooms` contain `id`, `native_constant`, native `template`, `tileset`,
`border_block` (integer), `dimensions_metatiles`, explicit hex `rows`,
`arrival_core`, `warps`, and optional `residents`, `signs`, `trainers`, `items`,
`programs`. Each warp is `index`, `tile`, target native constant `target`,
`target_warp`, `passive`. Index ordering is explicit and contiguous. Stair
floors/rooftops connect to real rooms; they do not need fake street facades.
Passive arrivals stand on ordinary floor and do not immediately trigger exits.
Native department-store `STAIRCASE` pairs are a separate explicit convention:
their indexed arrival can be the opposing stair tile, followed by Crystal's
automatic downward step to Floor. Preserve that native transition and assert
its settled location. Do not mistake it for a passive Floor warp or repeatedly
press into the stair until it bounces back. Directional carpets require their
actual native facing; stepping sideways onto a down carpet does not exit.

New `trainers` use the existing route trainer shape: `id`, `display_name`,
`trainer_class`, `sprite`, `tile`, `facing`, `sight_range`, `defeated_flag`,
`party` (`species`/`level`), `before_lines`, `after_lines`, `loss_lines`.
Native legal level-up moves are derived from the supplied pack. `items` use
`id`, `item`, positive `quantity`, `flag`, `tile`. List all fresh flags under
top-level `event_flags`; no main-story, HM or badge rewards are added.

`programs` maps new labels to native arrays of `{"command":...,"args":[...]}`.
Use it for actual check/give/take/setflag transactions and text bodies. It can
replace a newly added ordinary actor's script; it cannot overwrite an old
native label. Branch labels are globally unique. Native parsers compile all
new script tables. Keep full dialogue and exact reward/failure contracts in
the dossier. Resource failure must not prematurely set success flags.

`text_revisions` is limited to separately approved obsolete wayfinding labels:
each records `label`, exact old printed `before` lines and new `after` lines.
St Paul's public fair opening requires its two northern annex guard texts and
two obsolete fair wayfinding texts to change truthfully. Other old text/programs
remain exact.

Layout source-art additions must declare exact native block recipes and a
city-specific alias. Source pixels, source collision quadrants and palettes are
preserved; a renderer-only recipe is not native support. Root integrates the
art alias before publishing terrain and verifies source/native pixel fidelity.

Review the complete city and its emptiest districts by sight, then test actual
native travel, connections, all door/stair returns, stores/services, optional
battles, quest resource changes, refusal conditions and Continue persistence.
Static counts and large dimensions do not establish Goldenrod-level quality.
