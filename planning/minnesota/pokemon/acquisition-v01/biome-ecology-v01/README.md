# Minnesota encounter distribution

Use Crystal's encounter design: recurring low-level birds, bugs, small mammals
and plants, a few local specialties, changes by time of day, and stronger forms
as the journey advances. A species can occur on several routes. Habitat is a
useful local distinction, not a strict latitude rule.

The reference is the original pinned `pret/pokecrystal` checkout's
`data/wild/johto_grass.asm` and `data/wild/treemons.asm`, read from the external
export workspace. Routes29–31 repeat ordinary early creatures at levels2–5;
Routes38–39 reuse the same common species with different farm specialties.
Pineco occurs across multiple tree sets. The installed modular pack contains
additional completion encounters and was not treated as the unmodified oracle.
No disassembly or duplicate catalog is included here.

The opening Driftless woodland retains level4 Pinit. Prairie-farm grass favors
field species, while ordinary woodland species can appear in southern or
northern woods. Keep cheese curds, cookie-serving containers and carnival rides
at the State Fair. The lighthouse and smokehouse have their own specialties.
Generalist species remain shared across routes rather than being forced into
one exclusive location.

This biome layer is followed by the complete
[journey progression layer](../encounter-progression-v01/README.md), which fixes
inherited levels and form mix across every regional encounter map. The biome
layer alone is not a balanced release.

`ecology.json` edits the actual installed tables, including inherited local
zones and a logging-trail tree slot. Existing terrain, rates, weights and levels
remain intact except the declared relocated shipwreck level. Native rods retain
their thresholds, Old Rods and swarm behavior. The sealed Route2 drainage has no
accessible bank and therefore does not advertise Surf or fishing encounters.
The rhythm companion is a free Rochester resting-house offer after the first
clinic parcel; refusal/full party keeps that offer available.

The integration parent adds the nine optional rare-discovery sources to the
selected early-research lineage. Starter partners remain free at level5 after
the opening delivery. Both restored fossil lines are available after the Care
Badge through the same Fossil Lab; neither requires the Champion. This is an
acquisition revision, not a claim that an entire statewide campaign was replayed.

Build with the repository Rust publisher:

```sh
RUST_MIN_STACK=67108864 target/verdigrim-scene/debug/pack_minnesota_biome_ecology \
  content-packs/minnesota-current-fauna-current-strandrak-v01.crystalpack \
  planning/minnesota/pokemon/acquisition-v01/biome-ecology-v01/ecology.json \
  content-packs/minnesota-current-fauna-ecology-v02.crystalpack
```

Outputs must be fresh and ignored. Native local replays declare their initial
position, party, time and prior local trainer victories. Every subsequent
encounter, catch, resource change and Continue uses production input. Those
fixtures establish the local mechanics, not earned arrival from New Game.
