# Swept-crested Pidgeot and long-billed Fearow

`tools/raptor_bird_sculpt.py` replaces the shared generic bird construction
with original individual anatomy. The main builder dispatches its pure recipes
and rejects the obsolete Pidgeot/Fearow route. Pidgeotto now uses its separate [standard skin](pidgeotto-skin.md).
Pidgeot and Fearow now use the standard production skins documented in the
[skin guide](raptor-bird-skins.md). The following records their original static
sculpture baseline.

Pidgeot has a rounded eagle cranium, fitted angular ivory eyes inside hooked
charcoal cheek stripes, a short pink hooked bill and three long swept gold/red
crest ribbons. Cream belly, neck and throat layers follow their actual stored
parent surfaces. Its raised wings have gold leading spars, cream underside
plumage and brown upper plumage. Each wing has eight curved primary vanes and
six smaller coverts on each of its upper and underside surfaces. Five red rear feathers form
the tail. Pink feet have three forward digits, one rear digit and ivory hooks.

Fearow has a long bent narrow neck, narrow head, sharp fitted eyes, a genuinely
open long pink bill and an upright five-point red crown. Its ochre breast bib
and pale rear mane have ragged feather outlines. Brown primary vanes have pale
terminal markings fitted to both faces; separate raised upper and underside
feather layers keep the rear view from becoming an undecorated plate. Its five
rear feathers remain brown. It has the same complete digit anatomy, authored
against its individual body rather than the obsolete shared bird mesh.

Official [Pidgeot](https://sg.portal-pokemon.com/pokedex/0018/) and
[Fearow](https://sg.portal-pokemon.com/pokedex/0022/) artwork was visually
inspected in temporary browser tabs. No reference artwork is retained in the
repository or bundle. The first software review exposed an unsupported wing
root and an obscured crest; these were corrected. Further front/rear review
added continuous cream plumage and curved feather layers on both wing surfaces.

The [Free3D Pidgeot listing](https://free3d.com/3d-model/pidgeot-pokemon-92380.html)
reports no rig or animation and a personal-use license.
[Rashky's Mega Pidgeot metadata](https://api.sketchfab.com/v3/models/3398ad3b162b4f30b92e5ee2df3f9ab2)
reports downloads disabled and no license; that model is also a different form.
Eire's [Pidgeot metadata](https://api.sketchfab.com/v3/models/e0d70c4708404544a2120087bb5e6dd4)
and [Fearow metadata](https://api.sketchfab.com/v3/models/152b20b94b104fea9f59761f2f3707a0)
advertise downloadable CC BY 4.0 models with zero animation clips. Their download
endpoints returned HTTP 401 and their descriptions provide no authoring
provenance. Their geometry and skeletons could not be inspected; zero clips
alone does not prove absence of a rig. No external model was imported. Those
original sculptures formed the static baseline for the later original rig
migration; no external model was imported.

The original static Pidgeot has 105 parts and 70,352 triangles; Fearow has 161 parts
and 57,532 triangles. Heights remain the installed originals' 1.35 and 1.23,
with the lowest point at zero. SHA-256 prefixes are `a53a3cb719856d0f` and
`a7a23686116eb621` respectively.

The final candidates passed all fourteen stored-mesh tests in 29.829 seconds.
They cover closed connected outward solids, finite unit normals and actual
face winding, recipe parity, preserved height and grounding, unique identities,
triangle budgets, species-specific feather counts, nonadjacent body/neck and
feather face crossings, physical neck/head/bill/crest/mane attachment, every
wing and tail layer's actual volume root, and foot/toe/talon support. Fitted
face, breast, throat and pale-tip layers are checked against independent rays
through stored parent triangles, including front triangle centers and edge
midpoints. Fearow's jaw clearance is measured across every stored part;
deliberately detached-wing and filled-bill negative controls are rejected.

All final front, oblique and rear software images were rendered successfully
and inspected. The installed canonical meshes passed all fourteen tests again
in 28.041 seconds. Canonical layout and storage validation passed for the current
shared tree of 571 raw JSON model documents. The fresh production build passed
in 14m 39s, including shared-cache waiting. The Rust all-251 check passed its
one matching test in 1.84 seconds (1,248 filtered out).

Both production GPU battle captures completed with exit zero and were inspected
as full 2000-by-1400 images: Pidgeot has 50,938 colors and Fearow 48,726. Their
individual crests, feather layers, bills and grounded feet are visible with
the arena, opponent and production battle HUD. Pidgeot's long gold/red ribbons
and Fearow's open long bill remain clear in the production camera. The compiled
model fingerprints match the installed meshes. Both captures used one unchanged
executable with verified `CRYSTAL_PREVIEW_MUTE=1` for music and effects; model
hashes and executable identity were checked before and after each capture.

These trials use the explicit owned GPU image target and do not resolve the
separate live native-window capture issue. Temporary verification output is
removed after inspection. System volume and other applications remain
untouched. The full 251-species appearance goal remains active.
