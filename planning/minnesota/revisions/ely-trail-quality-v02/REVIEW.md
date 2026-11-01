# Whole-map review and remaining regional scope

## Current observed change

The v01 source/native whole shore map had a straight 32-core-tile approach
with grass as detached bands. The v02 native-art whole map replaces this with
a narrow northward climb, a turn around a pine/basalt knoll, then a descent to
the working clinic forecourt. Grass affects eastbound walking. The lower shelf
and real west-facing ledge make the return a different trip. The player-scale
native fixture shows the ledge between pine barriers with an open landing and
readable grass above. The whole Ely and connected context retain snowy pine
forest inland and a green North Shore below the snowline.

The actual native images, rather than just the source grid or thumbnail, were
viewed. Static topology alone does not certify the jump or encounter; production
input verification is recorded separately when complete. The source label
"westbound ledge" refers to native HOP_LEFT, not painted rock decoration.

No visual tile edits hide a collision defect. Native stream/waterfall/bridge
cells remain exact. No new water reaches an unbound map edge, and the original
reciprocal Ely/pass/coast seams remain exact. There is no new decorative Cut
tree. The short walk intentionally uses the existing guide and clinic service,
rather than adding several trainers to fill a small connection.

## Broader goal remains active

This is one concrete quality improvement, not completion of Minnesota. The
following still require current-pack, requirement-by-requirement review across
the region; older branch renders or isolated tests cannot establish them:

- Connected useful roads, all exterior doors and room returns; correct complete
  facades without internal house edges or exposed second-floor bottoms.
- Useful grass approaches and well-shaped short ledges with real directional
  behavior, safe landings and ordinary return routes. Outdoor terrace climbs
  should use authentic stairs when a climb is intended.
- Native tile variety that explains the location, readable route widths and
  meaningful features at the chosen size. Avoid repeated sparse compositions.
- All reachable Surf boundaries visibly closed by native buoys or rocks unless
  a reciprocal water neighbor is present; useful docks, fences and bollards.
  The [current waterfront child](../../connections/current-river-v01/README.md)
  now has a complete native Winona–St Cloud round trip, all seven bridge
  passages, badge/Falls checks and reached city-mouth frames. Other lakes and
  optional bank activities still require current evidence.
- Warm North Shore directly west into Grand Marais, both visibly on Superior;
  inland winter toward Ely rather than winter painted along the lake coast.
  North Shore cliffs, narrow tributaries and visible waterfalls must remain
  consistent across actual chunk seams and the state-scale atlas.
- Boundary Waters as connected lakes, grassy portages and pine islands with
  an actual entry gate, campsites and the reachable Root Beer Lady interaction.
- Iron Range rock identity and caves with purposeful chambers, entrances,
  progression and discoveries rather than rectangular hollows.
- Minneapolis, St Paul and Duluth as substantial, tighter inhabited cities with
  Goldenrod/Saffron activities. St Anthony Falls and its north/south river seams
  must function in the current integrated candidate.
- Glacier Line rail service between Detroit Lakes, St Cloud, St Paul and other
  authored stops; fairgrounds in scope; yellow prairie grass and substantial
  icy grass/winter geography rather than tiny scattered ice decorations.
- Minnesota-only current species appearances, legal regional parties and field
  utility, and LUTEBRISK retaining Ice typing. Each future touched map must be
  checked against the current roster/curation, rather than copied Johto tables.
- A full exterior atlas from actual connected native maps at one scale, marking
  unimplemented gaps. Preserve every original render and conversion for reuse.

The region has many independently published candidates. A future integration
must compare their actual content identities and owned module preimages, then
publish through Rust; taking the newest filename or overwriting an older whole
pack would lose accepted compact coast geography or newer regional services.
The current v02 candidate retains the compact coast and this local revision;
it does not claim to contain every other branch's latest change.

The successor [waterfront frontage revision](../minneapolis-waterfront-v01/README.md)
retains this geometry and qualifies the eight raw R02 edge-water cells using
the actual compiled module. Its fences and bridge rails allow no Surf mounting
and no reachable outer water. The initial raw-edge inventory is preserved;
the focused audit resolves this specific concern without decorative barriers
in inaccessible water. Other regional water boundaries remain in scope.
