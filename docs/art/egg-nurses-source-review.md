# Chansey and Blissey source review

Before replacement, the canonical pair was rendered at 1600×824 and
inspected in full.
It reuses a faceted pink egg body, protruding bead eyes, a stick smile,
cylinder arms, simple feet and triangular side curls. Both have a cream
badge-like pouch with a projecting white egg. Blissey adds three crown
spheres and a ring of white balls but keeps a wholly pink body. These
baselines remain unapproved for the requested art upgrade.

The [official Chansey page](https://sg.portal-pokemon.com/pokedex/0113/) and
[official Blissey page](https://sg.portal-pokemon.com/pokedex/0242/) were read.
The actual main images were retrieved with HTTP 200 and inspected in full:
[Chansey artwork](https://sg.portal-pokemon.com/images/pokedex/67786c71bc7911f9faa28b01ab1b2cf661d4c28f.png)
and [Blissey artwork](https://sg.portal-pokemon.com/images/pokedex/2ea9c95fc5631b6f3c053994398ecff332b40e56.png).
Neighboring-species images were excluded.

Chansey has a broad soft pink egg body, six long rounded side ribbons,
tiny dark eyes with white glints, a curved closed smile, short arms gathered
around a large pale egg, a pink cup-shaped pouch, small broad pink feet and
a short pointed rear tail. Blissey has a pink upper body and white lower
body, two large side crown lobes with curled front locks, tiny dark eyes, a
small open happy mouth, white wing-like shoulder frills, pink arm tips,
white skirt petals, slim pink legs and a pale egg in a pink pouch. The pink
pouch and Blissey's white lower body are material anatomy, not optional
accessories. The images are references and are not shipped assets.

Primary public model APIs and actual download endpoints were checked for
five relevant CC Attribution / CC BY 4.0 listings:

- [Chansey by nguyenlouis32](https://sketchfab.com/3d-models/chansey-f58955041e7647bf8c585f1432b6a771):
  3,660 faces, zero listed clips and a Pokédex-text description.
- [Chansey by PrufDesigner](https://sketchfab.com/3d-models/chansey-e3e9cda1787542c2b6261d0c8f5e18f5):
  520 faces, zero listed clips; the creator identifies it as Pixelmon Chansey.
  A listing does not establish independent model provenance.
- [Chansey by 7najohhthe97](https://sketchfab.com/3d-models/0113-chansey-3c57213939e84d60a067b2aa1f192df7):
  4,750 faces, zero listed clips and an empty description.
- [Blissey by nguyenlouis32](https://sketchfab.com/3d-models/blissey-0da7c07340f24705af94d37e7e485618):
  942 faces, zero listed clips and a Pokédex-text description.
- [Checkered Blissey by galexdl](https://sketchfab.com/3d-models/checkered-blissey-15f0b356fbae4bd3a3323f126b2c5aca):
  3,460 faces, zero listed clips and an empty description.

Every actual download endpoint returned HTTP 401, requiring authentication.
No archive, mesh, texture, rig or animation was retrieved or imported.
Zero listed clips does not establish absence of skeletal skinning. Paid
print files, coins, manhole covers and an unrelated human Blender mascot
are not replacements for this pair.

Original separately authored anatomy is the available next step, including
a real supported egg/pouch arrangement, rounded side ribbons for Chansey,
and white lower body, curling locks and feather-like frills for Blissey.
Chansey's original replacement is now installed after 12 candidate and
12 canonical checks and three updated views. Its first egg protruded below
the pink pouch; that was corrected and a stored-surface containment check
now rejects the same failure. Its explicitly muted 2000×1400 native GPU
capture exited 0 and was inspected in full. An unrelated concurrent
interior API change initially prevented Rust test compilation; those
interior fixes are now present.
Blissey's separate replacement is installed after ten candidate and ten
canonical checks and three final views. Its first curls failed the
nonadjacent-face crossing check and visually looked like horns; its skirt
petals were too narrow. Both were corrected before installation, and an
oval surprised mouth was changed to a happy open smile. All 12 expanded
canonical checks passed, including actual mouth space and filled invalid
solids. The final native build contains both full canonical hashes. Both
explicitly muted GPU captures exited 0 at 2000×1400 and were individually
inspected in full; executable identity and both source hashes stayed stable
after each. The direct exact all-251-species Rust loading/bounds check
passed in 9.75 seconds against an executable containing both full final
hashes, with executable identity and both source hashes stable through
the run. The union-feature cargo job subsequently exited 0 with the same
actual one-test check passing in 5.14 seconds; all-species art approval is
not claimed. These proofs cover geometry, attachments, three-view
inspection, muted final-hash native GPU captures and Rust model integration.
No ordinary live-window, browser, rig, animation or gameplay approval is
claimed. All egg-nurse verification jobs are terminal. Task-owned ignored
reference,
candidate and capture scratch was removed after the final proofs.
