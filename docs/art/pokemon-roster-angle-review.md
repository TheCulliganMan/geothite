# Full Pokémon roster angle review

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

Local captures, source hashes and review receipts remain ignored under
`target/pokemon-native-review/seal-strokes-20261020/roster-neutral-*`, with
the three replacement triptychs in the adjacent slime and Unown review
directories. Pack content and audio are unchanged.
