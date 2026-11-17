# Playtest 23: Northstar story cast sprites

The public browser pack inspected on November 1, 2026 had SHA-256
`a129f66ca806cd4c9f5ed0030c622a600da86c0e748f844001898a890b9f6c91`.
It used `SPRITE_ELM` for Tamarack in the Winona field station and
`SPRITE_RIVAL` for Reed, and its six Reed trainer records and Rochester
battle script used `RIVAL1`. Reed's private field and battle art was absent.
The report was a real release-lineage mismatch.

The unselected local review candidate `minnesota-story-review-fixes-v09`
(SHA-256 `175e419ac0abd0261d6417fdb825eabd3de0a7223b51f45baf1390ea8f29259a`)
contains the authored Tamarack and Reed field sprites and Reed's battle sprite.
The native art loader test resolves their palettes and poses, the Reed battle
image, and all ten authored field bindings. All six compiled Reed battle
requests select `MN_RIVAL_REED`: Winona, Rochester, St. Paul, relay, mine,
and boundary. Comparing all 864 trainer records with the selected
`minnesota-reviewed-maps-v11` parent found only those six class changes;
all other trainer fields and unrelated trainer records were equal.

Using an earned New Game checkpoint at the case scene, the production-controller
test continued into Reed's battle and captured the native battle introduction.
The rendered image was inspected and showed the authored Reed portrait with
the `REED wants to battle!` text. The existing executable's art-loader and
battle-capture tests passed against v09. The added assertions for compiled
battle requests and the actual battle class still need coordinator validation.

This is a review candidate, not the public release. Preserve its cast bindings
when integrating subsequent playtest repairs. Production publication and
rendered overworld review remain separate acceptance steps.
