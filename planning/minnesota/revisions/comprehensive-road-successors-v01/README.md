# Accepted road and R17 cave integration

The accepted R06 south-woods world descends from the R17 loop, R05 bank/cache,
R09 southwest grass and R11 upper grass successors. Its complete delta over
the previously integrated ice-turning world is twelve entries: six cached maps,
five raw block mirrors and one R05 sign program. No assets/audio change.

`integrate_comprehensive_road_successors` pins comprehensive v08 (baacf2ca),
the common ice world (e964dc76) and the accepted R06 world (41aad022).
Every adopted entry requires the exact common preimage. The independent
`audit_comprehensive_road_successors.py` computes the full three-way result,
requires the exact twelve paths and rejects changes to optional content,
regional fishing, Reed, audio and the existing mine-cache claim flag.

Compilation/publication and scoped native successor checks remain pending.
The current selector is unchanged. A complete earned campaign on the final
combined candidate remains required; old saves are not migrated.
