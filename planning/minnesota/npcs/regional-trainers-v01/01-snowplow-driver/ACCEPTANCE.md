# Snowplow Driver ROWENA

Accepted native source: `native-pixels-v4.json`; placement: `placement-v1.json`.
Uses Crystal Pokéfan F proportions and real walking poses, with hand-edited orange
coat and dark trousers. Native v2 battle eyes, cap and smile were simplified.
Field v2 looked bare in the actual game; v3 was too black. Both are rejected.
Native v4 and the actual field/battle captures were inspected at native scale
and enlarged. The orange coat reads clearly while the original head stays intact.

Intermediate pack: `content-packs/minnesota-regional-trainers-01-snowplow-v03.crystalpack`.
Production controller: `target/regional-trainers-v01/01-snowplow-driver/controller-v3.log`
passed (1 test, 21.32 s), with captures in `gameplay-v3/`. This checks dialogue
ownership, a selected move and PP mutation, legal victory, defeat/repeat and
save/load, using the declared three level-60 legal Minnesota starter fixture.
This is neither intended-level balance nor final combined-pack acceptance.
