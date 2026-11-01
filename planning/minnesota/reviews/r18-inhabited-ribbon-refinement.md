# R18 inhabited dune ribbon — second source proposal

The existing meadow/dune margin now shapes a winding dry approach that descends
to a small lake-facing railing, then climbs the existing public-launch spine.
A short direct grass crossing saves six steps. This addresses the ten-core clear
central lawn without adding another tree island, pier, HM gate or actor. The
approved shorewalk v04 and every existing R18 activity remain exact. This is a
separate unpublished source candidate awaiting Parent visual review and native
testing; active source, packs and Rust were not changed.

[Whole before/after](../renders/mille-lacs-shore/ribbon-20261027/v01/before-after-whole-native32.png)
and [all player-scale views](../renders/mille-lacs-shore/ribbon-20261027/v01/candidate-decision-cameras.png)
show the complete40×24 route. The whole map, existing activities, new bend,
grass choice, lake view and automatically selected weak camera were personally
inspected. The margin connects to the actual necessary habitat, rather than
becoming an isolated prop. The new dry route visits the lookout as part of the
journey; its grass shortcut is legible beside it. The flat shoreline, wider
working piers and thin public shorewalk keep their distinct silhouettes.

## Source boundary and proof

Before sources are exact approved shorewalk v04, preserved under
`renders/mille-lacs-shore/ribbon-20261027/before-shorewalk-v04/`:
dossier SHA256
`21a5bd33acb1760380b1a94494728965f8760d267ee4f969f9bee7e795842c4c`,
layout
`2b5b4a542513691e07dc35845c16af4c6f6a20bad879f3156ed76c9a29290618`.
The native art/reference pack remains exact538 `da27b381…`; the complete approved
shorewalk has not been assumed installed in that reference pack. Future native
composition must use a matching published shorewalk preimage.

Candidate sources are `routes/MN_R18.ribbon.candidate.json` and
`routes/MN_R18.ribbon.layout.json`; the frozen handoff records their full hashes.
Every existing gameplay/event/art field is exact except replacing the main-path
polyline with the actual new dry course. All old loops, camera records and
review records are exact prefixes. Only additional cameras and editorial notes
are added. The layout preserves every prior metadata field, art recipe and its
complete layer prefix. Its24 final block changes stay within meta
`[21,3,27,10)`, using native grass03, ordinary road01 and fitted rail48/49.

All existing quest commands, actors, trainer parties, rewards, flags, badges,
signs and source encounters are exact. Every original grass quadrant and every
old WATER/BUOY metatile is exact. Both western piers, approved shorewalk/dune
bytes, public launch, optional Surf islet and external rock closures are exact.
Both regional directions still require exactly six grass steps; all original
item entry prices remain exact. All four SQ04 actors retain walking approaches.

[The source report](../renders/mille-lacs-shore/ribbon-20261027/v01/review.json)
passes35 checks and rejects ten saved adversarial sources. It invokes the
unchanged full flat-shore checker and ordinary route validator. Negative cases
remove the grass price, erase the dry approach, open the lookout rail to water,
restore the empty lawn, damage the approved shorewalk/old habitat, insert an
incidental Cut, block the launch, change quest completion or move Dory. No shared
guard was weakened. Complete originals, RGBA stages, crops, previews, source
snapshots and every rejected input remain reusable.

## Exact walking choice and observation contract

| Feature | Native core contract |
| --- | --- |
| Decision start / target | `(45,13)` → `(48,13)`. |
| Direct grass route | `(46,13)` → `(47,13)` → `(48,13)`: three walking steps, exactly two TALL_GRASS cells. |
| Dry winding route | `(45,14)` → `(45,15)` → `(45,16)` → `(46,16)` → `(47,16)` → `(48,16)` → `(48,15)` → `(48,14)` → `(48,13)`: nine steps, zero grass. |
| Offset lookout | Core `(45,16)` is ordinary Floor with a visible lake; the native player frame contains22 WATER core cells. Lower row17 is real WALL at x43–47, with a fitted west cap at `(42,16)`. It does not add a Surf opening. |
| Regional main road | New dry source polyline: meta `(1,5)` → `(6,5)` → `(6,6)` → `(22,6)` → `(22,8)` → `(24,8)` → `(24,5)` → `(38,5)`. |
| Public launch/sign | Exact original `(49,19)` Floor → `(49,20)` Water and original sign/reading footing `(52/53,16/17)`. |
| Dry shorewalk approach | The launch spine `(48/49,16/17)` and sign-side cross step `(50/51,16/17)` remain dry, preserving access to the approved lower circuit. |

The biggest all-Floor square inside the former central lawn falls from ten to
two core tiles. Grass remains walkable: this is an encounter/route-choice change,
not a claim that movement is physically confined to that narrow strip. The
unchanged western junction becomes the weakest whole-route Floor view and is
included. No whole-R18 completion or user approval is claimed.

Native source previews use the actual day palette, standing facing and tile
top-left. Ordinary native wall foreground is applied; grass-specific foreground
outside that bounded helper is explicitly marked in the per-camera report and
needs actual native comparison. Source render/collision proof cannot establish
GPU fidelity, random encounters, actual battle/PP effects, report acknowledgment,
item ownership, Save/Continue or earned regional story progression.

After source visual acceptance, use the existing Rust same-extent terrain API
with frozen previous shorewalk layout, this candidate layout and revised path
dossier. Only the existing R18 exterior blocks/owned encoded payloads and derived
identity may change; no events are reinstalled. Verify the written pack against
the exact24-cell delta and preserve all other maps, attributes, scripts/programs,
art/audio, encounters, report, HOME/recovery and catalogs byte-for-byte.

Production acceptance should walk both the dry nine-step route and direct
three-step grass route through the shared controller, with honest legal initial
party/position/RNG fixtures and no later state writes. Observe real encounters
where required, blocked rail movement, the original six-grass neck and new
shorewalk seam. Retain all existing SQ04/rod/Surf-islet/Continue coverage. Capture
both bend approaches, old blank `(47,11)`, grass `(47,13)`, lookout `(45,16)`,
launch fork `(49,15)` and the new computed whole-route weak view before release.
