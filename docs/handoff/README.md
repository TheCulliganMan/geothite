# Restart the 3D goals

This is the host-transfer checkpoint for `feat/connected-johto-3d`.
The visual goals remain unfinished. Their notes preserve what each chat was
doing, verified results, known defects, dependencies, and exact restart steps.
Read the relevant note before resuming changes; source checkpoints do not imply
visual approval or completion of a goal.

| Work | Restart note |
| --- | --- |
| Human characters, clothing, hands, shoes and bicycles | [Character quality](character-quality.md) |
| Map scenery and canonical Pokémon model work | [Map scenery and Pokémon](map-scenery-and-canonical-pokemon.md) |
| Exterior terrain and source-driven scenery | [Outdoor terrain](outdoor-terrain.md) |
| Lighting, effects, performance and Goldenrod crash investigation | [Rendering and Goldenrod](render-performance-and-goldenrod.md) |
| Kanto interiors and shared furniture geometry | [Kanto interiors](kanto-interiors.md) |
| Cave geometry, stairs, rock footing and visual fidelity | [Cave fidelity](cave-3d-fidelity.md) |

Start with [the cold-start and transfer runbook](../cold-start-3d.md). Git
commits, Git LFS artwork, ignored content packs and saves are separate transfer
inputs. Build caches and old captures are evidence, not required runtime inputs.
Use `scripts/cold-start-3d.sh` to validate/build/play; the goal notes contain
additional commands for their regression and capture workflows.

Several chats edited shared renderer modules. Their individual commits must be
transferred together with the final shared integration checkpoint. Do not
cherry-pick a scope's standalone commit and assume it includes every module
registration, source binding, mesh deletion or rendering change used by its tests.

Resume one goal at a time initially. Confirm the destination's pack, artwork,
GPU/window environment and production controller behavior, then reproduce the
specific unfinished check in that goal's note. Keep new capture/test output in
ignored directories and update the note with new verified results. Commit owned
paths explicitly; coordinate shared files before concurrent edits.
