# Flarkindle: flower mythical redesign

Replaced the flat pointed bud and raised glove-like cup with a lavender three-petal hood, exposed pale face, calm smile and asymmetric leaf gesture. One peat rib opens low and outward, a second curves behind the hood, and the shorter front rib stays below the face. Two open fen-pool gaps and the short root tassel preserve its memory-keeper identity. The rear swaps the arm heights and enlarges the nape; lower roots crop offscreen.

The source sheet was edited with the **built-in imagegen tool**. [First prompt](prompt-v1.txt) establishes the blossom hood and pale face; [second prompt](prompt-v2.txt) corrects the raised-arm silhouette. Both generated sheets remain saved here as `front-back.imagegen-v1.png` and `front-back.imagegen-v2.png`. Earlier source versions and `manifest-before.json` are preserved.

Selected native recipe: [manifest-accepted.json](manifest-accepted.json), second source, area-vote sampling, solid exterior, front fit52×53 / rear46×47, original RGB555 palette `[32767,30459,11687,0]`. Fifteen guarded native pixels separate the upright pupils and catchlights from the face and remove a stray hood pixel. The source-center candidates were rejected because they broke facial rims and introduced stray green/black pixels. The symmetric first source remains a saved alternative.

[Accepted front and rear](../../../../renders/flarkindle-redesign-20261027-v4/accepted/line.review.png) · [Previous sprite](../../../../renders/flarkindle-redesign-20261027-v4/accepted/versions/4f7ea572f33ce8d29eb0d0afa0251f264068793f94f3cf3b775195981d7d0dca.line.review.png).

Both native views pass independent PNG/index/2bpp readback, exact four-ink palette, byte-length and border checks. A one-species fixture successfully builds through the production catalog and guarded source replay into a 252-species pack against `content-packs/core-modular.browser.crystalpack`. `verify_crystalpack --audio --desktop` passed: 388 maps, 252 species, 251 moves and all 377 audio PCM hashes, frame counts and loop ranges match. Temporary validation files are removed afterward. Shared published packs are not overwritten by this art task.
