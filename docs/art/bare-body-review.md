# Exposed-body geometry and mapping review — 2026-10-11

Scope: Bruno, Chuck, Swimmer Guy, and Swimmer Girl. These models are not approved as finished. The full 75-role character objective remains open.

The later [swimsuit review](swimsuit-review.md) supersedes the female swimmer's trouser/waist findings below; that shape now has a fitted one-piece leg opening and continuous blue fabric. Remaining quality requirements still apply.

The production mesh now rounds the source sleeve surfaces with shoulder/wrist boundaries pinned and maps exposed skin onto the inspected plain skin patch of each supplied atlas. Bruno, Chuck, and Swimmer Guy use that patch for their torso as well. Swimmer Guy now uses the existing fitted shorts operation, including shared cut vertices, exposed lower legs, and a narrow closed fabric hem. No generated painting was changed in this pass.

The first four-angle Idle render revealed cloth-colored shoulder caps and a remaining lower-back shirt island. The selection was corrected to cover complete shoulder triangles and the full torso island. This finding demonstrates why skin-pixel/weight regressions alone are insufficient to approve visual quality.

Initial window screenshot extraction stayed blank after 60 retries. The existing image-target reviewer produced actual lit model renders successfully. This validates the image-target review route only; native-window screenshot extraction remains unverified.

Verification and captures are kept outside tracked source at `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/`. The revised production reviewer build passed. The supplied-rig/painting regression passed for all four identities, and the shorts seam/weight/hem regression passed with Swimmer Guy added. Four-angle Idle and Run 0.125/0.625 captures for all four identities were each individually inspected: twelve images, forty-eight rendered views. Final captures are in `captures/bare-body-final-idle/`, `captures/bare-body-final-run125/`, and `captures/bare-body-final-run625/`.

The revised render removes the cloth-colored shoulder caps and lower-back shirt patches on the three bare-torso roles. Swimmer Girl now has exposed arms rather than source long sleeves, but still reads as a sleeveless shirt and trousers, not a completed swimsuit. Swimmer Guy's shorts retain exposed legs and joined hems in both sampled running poses. All four still show wrist color/shape transitions, a simple shirt-shaped torso rather than anatomical contours, faceted jaw/nape profiles, and generic shoes. None of these renders establishes final art approval or in-game context quality.

Still required: convincing muscular torso shapes for Bruno/Chuck, fitted belts/wrist wraps, swimmer barefoot geometry, a proper female swimsuit cut and exposed legs, goggles, improved jaw/nape seams, and better hands. Skin mapping does not establish those shape or outfit requirements.
