# Full Pokémon roster angle review

User feedback reopened this earlier visual acceptance on October 20. A fresh
quarter-view overview still found weak proportions and postures in the roster;
the historical three-angle claim below is not current blanket approval. See
the [Totodile family/Geodude revision](crocodile-geodude-fidelity.md) and
[Graveler/Golem revision and remaining triage](stone-pair-fidelity.md), plus the
[Ampharos/Granbull/Mewtwo posture and leg-angle revision](upright-trio-fidelity.md)
and subsequent [Ampharos/Granbull head-volume refinement](head-volume-fidelity.md).
The [Machop family volume and posture revision](fighting-sculptures.md) now
replaces its earlier generic heads, shared arm posture and round foot pads.
The [Elekid/Electabuzz/Magby/Magmar revision](elemental-fidelity.md) addresses
round-body depth, upright supports, distinct arm carriage and flame volume.
The [Hitmonchan/Hitmonlee/Hitmontop revision](hitmon-fidelity.md) adds rounded
head depth, upright coils, complete boots and a compact, source-backed spin pose.
The [Tyrogue/Smoochum/Jynx revision](ice-fighter-fidelity.md) corrects skull and bob
depth, short bent supports, temple seams, actual mouth recesses and distinct guards.
The [Rhyhorn/Rhydon/Larvitar revision](rhino-larva-fidelity.md) corrects flank
armor volume, flat crest roofs, shin direction, raised palms, skull depth,
invented fingers and the radial tail fan.
The [Tyranitar revision](tyranitar-fidelity.md) corrects its bottle torso,
diagonal ankles, long muzzle, single dorsal row, raised tail and missing vents.
Its [native battle follow-up](pokemon-source-frame-fidelity.md) repairs a
terrain-origin correspondence failure and verifies actual admitted, matched
battles without changing model size or admission thresholds.
The [Cyndaquil/Quilava/Typhlosion revision](fire-starter-fidelity.md) uses the
actual Crystal front animations and genuine game full orbits to correct
forepaw support, Quilava's stance, muzzle depth, coat length and flame width.
The [Chikorita/Bayleef/Meganium revision](grass-starter-fidelity.md) corrects
barrel and skull depth, collar volume, crown direction, antennae, mouth space,
tail roots and both flower surfaces using genuine complete game orbits.

The [Sneasel/Stantler revision](claw-deer-fidelity.md) corrects sagittal
feather planes, upright hocks, angular antlers, padded cheeks, flank ribs and
sloping cloven hoof volumes.
The [Scyther/Snorlax revision](mantis-sleeper-fidelity.md) corrects actual blade
and wing profile area, short crown points, upright greaves, rear shell seams,
seated barrel/hood depth, bent arms and larger soles.
The [Charizard/Dragonite revision](dragon-pair-fidelity.md) corrects forward
neck carriage, upright ankles, raised/bent palms, compact rear-swept wings,
wing front/back pigments, backed mouth volume and long low tails. It also
corrects Charizard's former sail loop failure.
The [Aerodactyl/Kabutops revision](fossil-pair-fidelity.md) corrects broad wing
planes, jaw and eye depth, helmet and chest volume, scythe thickness, hocks,
back ridges and tail direction. It also resolves Aerodactyl's loop seam.
That batch's wider near-loop check reached an unchanged Pidgeotto wing and
failed at the existing tolerance. The subsequent
[Pidgey family revision](pidgey-family-fidelity.md) corrects it; both its focused
check and the complete-roster check now pass without relaxing the tolerance.
The [Spearow/Fearow revision](spearow-fearow-fidelity.md) corrects floating wing
cases, diagonal shanks, feather pigment boundaries, forward neck carriage,
wing profile area, the sagittal comb and airborne tucked claws.
The [Hoothoot/Noctowl revision](owl-fidelity.md) corrects belly/eye pigments,
clock-hand and crown shapes, shoulder and tail roots, folded-wing depth and
upright shanks. It preserves the tucked foot during one-support world travel.
The [Sentret/Furret revision](sentret-furret-fidelity.md) corrects the narrow
hooked support, coat pigments, extra belts, thin body, muzzle depth and paw
colours. Its motion review retains Sentret's suspended feet and Furret's
separate four-paw travel posture.

## Earlier roster pass

The current catalog contains 251 species and 276 canonical GLB variants,
including all 26 Unown letters. A fresh windowless GPU review captured every
variant from the front, side and rear. All 23 labeled contact sheets were
individually inspected for silhouette, proportions, facial readability,
material boundaries, exposed undersides and detached parts.

The reviewer reads the actual canonical GLBs, including the five actor-space
sources. It applies their scene transforms, vertex normals and material colors
under shared PBR lighting. Three instances use identical normalized scale;
this deliberately makes small and large species equally inspectable. The
renderer excludes Winit and audio and captures directly to an image target.
No windows open and no game or save state changes.

The angle pass found broken-looking body creases on Grimer and Muk and a gap
between Unown Y's lower stem and foot. Independent occupied-volume checks
confirmed these defects. Their corrections preserve unrelated anatomy, bounds,
materials and animation keys:

- [Grimer and Muk folds](slime-sculptures.md#continuous-fold-roots) now meet
  their actual body triangles throughout the tested clips.
- [Unown Y's foot](unown-sculpture.md#y-foot-attachment) meets its stem, matching
  the supplied pack's front animation frames and back picture.

The initial complete capture retained immutable source and reviewer hashes.
After those corrections, 273 variants retained their exact captured GLB bytes;
Grimer, Muk and Y were recaptured with the same reviewer. The three affected
sheets were rebuilt and individually inspected again. No blocking silhouette
or attachment defect was observed in these sampled neutral angles.

This review supplements the earlier [full-roster pose and numerical
checks](pokemon-authoring-portability.md). It uses neutral geometry and
normalized studio scale; the species guides separately record matched-state
2D/3D production reviews at pack size. These three angles do not establish
every animation phase, map, camera, move effect or browser presentation.
Shiny variants were not part of this pass.

A current packed-buffer audit also passed for every GLB: 3,436,678 vertices
and 5,244,489 triangles across the 276 variants. It calls the existing geometry
and storage validators on the actual stored attributes, requiring finite
values, unit normals, outward winding, positive volume, floor roots, unique
geometry and unchanged loader budgets. This is separate from regenerating
all authoring recipes. The five corrected models pass their focused source
parity and anatomy/skin checks; the earlier full authoring survey is recorded
in the portability guide.

The current runtime schema and complete roster motion tests also pass. Before
execution, all 276 exact compressed canonical GLBs were verified in the test
executable; source and executable hashes remained unchanged afterward. The
motion test samples sixteen travel phases per entry, including the base Unown
alias, and checks hierarchy order, support chains, floor penetration, excessive
floor correction and connected split surfaces. Focused seal and slime support/
articulation tests pass as well. These numerical checks do not replace the
scoped production reviews above.
The fresh Unown regression also passes for all 26 forms after distinguishing
the strict authored letter selector from the world roster's explicit I alias.
Unresolved battle form identities remain rejected.

The final Y production review also passed in a witnessed Route 29 encounter;
its [form guide](unown-sculpture.md#y-foot-attachment) records the state-matched
pair and the executed review command. The classic move menu covers the
player's back picture, so the pack pictures and studio review supply the
attachment comparison.

Temporary captures, source hashes and review receipts were kept outside
tracked source and removed after verification. Pack content and audio are
unchanged.
