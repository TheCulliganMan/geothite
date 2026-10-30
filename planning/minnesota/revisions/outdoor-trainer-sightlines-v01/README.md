# Mina's lower-bench sight line

The real boulder MCP replay exposed a previously covered trainer sight line.
After crossing the cleared rim to(41,34), Mina at(41,37) faced north and
challenged the player across the tall cliff at(41,35). The native sight test is
geometric; changing the global engine would alter Crystal behavior elsewhere.

`plan.json` turns MN_R09_T03 east along the existing lower bench. Her complete
three-tile lane(42..44,37) is ordinary walkable ground with no NPC body. Her
position, range, team, dialogue, flags and combat remain intact. The source
preview and exact blocked/clear lane check are retained in
`renders/outdoor-stair-landings-v01/iron-sightline-v62/`. The nine other boulder
courses passed; the original failed Iron Range receipt stays in
`boulder-stdio-v60/MnR09C00R00-setback-20-17/`.

The Rust builder uses a separate trainer-only revision and full map preimage
hash, validates the native actor identity and old facing, and requires every
step of the complete new sight lane to be walkable and unoccupied. It changes
only the one standing-direction field. The strict pack audit must preserve all
terrain, programs, battles, parties, art, audio and every other actor field.

Mina's current native team is active Minnesota CARTICK, BANDRAKE and PELLETANK.
The crate-internal ignored test `mina_every_scaled_team_uses_active_minnesota_families`
checks every production rematch/evolution from the authored ace through100
against the runtime-read active roster and retired-family exclusions. It needs
`CRYSTAL_OUTDOOR_SIGHTLINE_PACK`. It is queued, not yet a passing scaling claim.

Native publication and input verification remain pending. Required gameplay:
cross the upper rim without trainer dialogue and retain its solid north/south
walls; approach Mina from the east on the lower bench and trigger her ordinary
challenge/battle. Do not mark her defeated or disable her sight to make the
geometry case pass. No existing failed receipt or screenshot is overwritten.

Native v02 now exists on the ten-boulder parent. The strict v69 comparison
passed: exactly one native standing-direction field changes, with terrain,
every other actor field, battles/teams, programs, art and audio exact.
`renders/outdoor-stair-landings-v01/sightline-preservation-v69/receipt.json`
records the parent/child/plan hashes. The seven-corner child is
`minnesota-current-outdoor-terrain-v02.crystalpack`, SHA256
1890d9e0e261a0a1387c6fb9d64846a9d3547cc3311a5009d6c151090e07131f.
All377 audio checks pass for that composed child. Native input and the
crate-internal scaled-team check remain pending; the candidate is unselected.

The composed terrain v02 production MCP run now passes all17 boulder/corner
cases with210 actual calls and empty stderr, including Mina's formerly failing
upper-rim crossing. Evidence: `current-terrain-stdio-v72/receipt.json` and
`current-terrain-mcp-proof-v72.json` under the outdoor-stair render folder.
Initial local position/partners/Repel are explicit setup. No trainer is marked
defeated and no state is repaired after input. Mina's retained lower-bench
battle, scaled roster and fresh renderer checks remain separately queued.

Mina's lower-bench challenge now passes through production MCP on composed
terrain v02: normal approach, FIGHT/PKMN/PACK/RUN, move selection and actual
FLAMETHROWER turn. The visible move menu and native save validator both show
PP15→14; native saves preserve186HP, bag and badges. Evidence:
`mina-battle-stdio-v80/receipt.json`, `mina-native-turn-check-v81.log`, and
`mina-battle-proof-v81.json` under the render folder. Healthy owned level60
Borealynx/Voyagear and initial position are explicit setup, not earned play.
The first v79 harness mishandled the ordinary post-KO party prompt; retain its
failed receipt. The v80 run declines that prompt through normal B input and
returns to the battle menu. No live state or cursor is rewritten.
