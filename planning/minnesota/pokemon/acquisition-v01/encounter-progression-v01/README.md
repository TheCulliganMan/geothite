# Minnesota encounter progression

The encounter curve follows the actual journey in [PROGRESSION.md](../../../PROGRESSION.md),
with Gym aces at 12, 16, 21, 25, 30, 35, 40 and 45. Grand Marais is reached after
all eight badges and the Harbor signal. Its inherited level 2–3 town table was wrong.
The western loops open after Lake; their route numbers do not imply a late
campaign visit. Surf is available from the Capitol-qualified St Paul handoff,
before the optional Brainerd handoff.

`progression.json` explicitly assigns all 111 installed regional wild maps to
45 journey groups. It covers ordinary grass, cave floors, local zones, Surf,
swarm tables and both Headbutt habitats. Levels and the main-table form mix are
authored separately from the habitat corrections in the preceding biome layer.
Rates, slot weights, time periods, zone boundaries, terrain and story gates
remain intact. The publisher rejects an omitted habitat rather than silently
retaining an inherited table.

| Journey | Ordinary walking levels |
| --- | --- |
| Opening Driftless woods | 2–4; optional lower gallery 5–7 |
| After Care, prairie farms | 8–11 |
| After Prairie, River Valley | 12–17 |
| After Current, Twin Cities | 17–21 |
| After Current and Capitol, northbound woods | 21–25 |
| After Wind, Pine Country | 25–30 |
| After Lake, lakes and western loops | 29–36 |
| After Pine, Iron Range and harbor road | 34–41 |
| After Harbor signal, North Shore and Grand Marais | 39–44; town 39–43 |
| Gunflint and Boundary | 41–46 |
| Restored-survey Ely approach | 43–48 |

Later tables mix recurring common species, middle forms and stronger local
forms. Explicit promotions preserve earlier base-form sources. Species whose
evolution level exceeds a slot's level are walked back within their own family.
This removes underlevel State Fair showcase finals: the first visit offers
levels 21–25, and the final forms are obtained by training those families.
The opening still has Pinit 4 and Dewbit in its seep, alongside the free Grass
starter Pouchid. Pinit also remains in later woods at suitable levels. It is
neither prairie-farm filler nor restricted to northern Minnesota. Crowscare's
510 base-stat total belongs to the later prairie loop; the early farm's night
patch uses Vanechick instead. Grand Marais's grass pockets all use its late
level band. Opening routes retain repeatable low-level encounters on return.

Ordinary trainers on the reviewed maps receive stage floors while retaining
their team level differences, native evolution/learned-move behavior, names,
classes, items, DVs, defeat flags and rewards. The eight Gym aces and League
teams keep their authored curve. Existing runtime scaling can raise a battle's
level; it does not replace these authored baselines.

Methods have distinct contracts. Early-town Surf encounters start at 20–24 on
the later return visit. Original rod tiers remain unchanged, including lower
Old Rod catches, day/night slots and swarm behavior. Free starter partners 5,
the optional hatchling, the clinic companion 8 and revived fossils 10 are
deliberate gifts. Grand Marais's optional young research partner remains a
separate gift; all three starter families are already available near the start.
Voluntary rare discoveries retain their authored levels and independent retry
conditions. No species requires trading, a paid Safari visit or choosing one
fossil permanently over the other.

The original pinned Crystal encounter sources were inspected outside this
repository: early routes reuse basic birds/bugs/rodents, later routes mix base
and evolved forms at rising levels, and fishing follows rod tiers. Emerald's
[native encounter source](https://github.com/pret/pokeemerald/blob/master/src/data/wild_encounters.json)
was also checked: its early-route land tables and later fishing/Surf access
reinforce that walking progression and field-method progression are separate.
No disassembly, exported catalog or content pack is included here.

Build a fresh ignored candidate with the Rust publisher:

```sh
RUST_MIN_STACK=67108864 target/verdigrim-scene/debug/pack_minnesota_encounter_progression \
  content-packs/minnesota-current-fauna-ecology-v02.crystalpack \
  planning/minnesota/pokemon/acquisition-v01/encounter-progression-v01/progression.json \
  content-packs/minnesota-fauna-progression-v01.crystalpack
```

Run `planning/minnesota/tools/audit_encounter_progression.py PARENT PACK PLAN`
for the independent read-only table, stage, evolution, catch-rate, learnset,
trainer and preservation audit. The separate surface audit excludes inert and
paid-only sources. Native controller replays check opening-to-League level
bands with declared stage-appropriate partners, actual encounters, captures,
Surf mounting and Save/Continue. Such local fixtures establish those mechanics;
they do not establish an uninterrupted earned campaign or capture of all 225.

## Verified result

The completed fauna pack is
`content-packs/minnesota-fauna-progression-title-v01.crystalpack`, identity
`8e23fbfad0caa1aa19e0ae0db731cd03621d7178239f48e06fac3a3d4a880145`.
It retains the current Northstar title/Tamarack opening exactly. Reapply the
accepted title publisher after the encounter publisher above:

```sh
target/verdigrim-scene/debug/pack_northstar_title \
  content-packs/minnesota-fauna-progression-v01.crystalpack \
  planning/minnesota/npcs/tamarack-ice/01-tamarack/native-pixels-v4.json \
  content-packs/minnesota-fauna-progression-title-v01.crystalpack
```

The written-pack audit checks 111 encounter maps, 246 local trainer teams,
every time period, field bucket, fishing indirection, scripted gift and static
encounter. It corrects 112 trainer teams. Independent surface-aware closure
finds 164 conditional direct sources and all 225 species after evolution,
excluding paid Safari areas and swarm-only fishing. Every active species has
a positive catch rate and valid learnset; every evolution is a native level
evolution, with the latest at 55. The full 602-program audio verification passes.

Native input replays pass 66 grass locations, 58 Surf banks, 24 rod/clock
combinations, 24 reviewed habitat captures, all 39 rare-discovery cases,
the early starter/fossil cases, four Castorock cases and the clinic, Sweet Scent,
drainage and Headbutt checks. The progression replay uses suitable partners at
13 walking stages, six fair habitats and three Surf stages, checks actual enemy
levels, captures and continues each saved result. All final-pack progression,
specialist, northwoods, early-research and Castorock rechecks pass.

The production stdio client continues the final pack's earned opening return:
39 real calls, three starters, zero badges, authoritative movement and Start
menu, unchanged source save and byte-empty stderr. These checks establish
acquisition conditions and local mechanics. An uninterrupted full campaign and
earned capture of every species remain unclaimed.

The complete title delta independently matches its reviewed publication while
every fauna map, encounter table, team, fishing rule and audio program remains
exact. All-target Clippy passes for the affected crates with existing warnings;
whole-workspace formatting still reports pre-existing differences starting in
`offset-allocator`. The changed leaf modules are formatted.

Web release `20261031-progression-32` preserves all 225 pages and existing media.
Locations use the final pack's levels and access conditions, show the earliest
route first, and give starting locations and training steps for evolution-only
entries. Application, full-catalog, desktop and mobile browser checks pass;
the new field-guide panels were visually inspected. Resolve `world.json` and
the live Pokédex for selection/publication status.

## Consolidated habitat refinements

The main Grand Marais grass remains in its late North Shore level band. An
eight-tile western cedar margin at walking tiles (4–5,10–13) retains repeatable
level 3–7 basic residents, reachable on foot from the Pokémon Center. Existing
Minnesota encounter scaling keeps an authored-level minority, including this
pocket, and preserves encounter species. Repel filters low levels normally.

The northern pine chunk MnR11C00R00 favors Latchquill (30%) and Pitchwick (5%).
The next basalt chunk MnR11C00R01 favors Bandrake (30%) and Agatot (20%).
Other coastal residents, zones, rates, story gates and terrain remain intact.
These refinements extend the complete 225-species fauna/progression source;
final combined-pack validation is recorded by the consolidation runbook.
