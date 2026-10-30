# Duluth garden stair verification

The five-metatile candidate uses an installed cave36 flight, retaining walls and a lower turning landing. Source coordinates are metatiles; gameplay coordinates in the journey are 16px walking tiles. The selected parent remains Duluth cliff-stairs v03 until fresh publication and source checks finish.

| Check | Result |
| --- | --- |
| Exact preview preservation | Five Duluth cells and their block mirror change; all 729 other maps and normalized pack data remain exact |
| Native audio | All 377 PCM hashes, frame counts and loop ranges pass |
| Player-scale native2D | Three inspected complete 640×576 views; zero pixel mismatch |
| Production stdio route | 78 actual RPCs, forced side-wall and foot-cap denials, main street, Save/Continue, real clinic return and uphill journey; both processes exit0 with empty stderr; input save unchanged |
| Ten existing one-way ledge lanes | Preview native regression passes: hops, reverse denials and ordinary stair returns |
| Eleven existing stair flights | Initial preview regression traversed its courses but failed an obsolete final assertion that the new intentional street connection must still be blocked; failure retained. The source now checks the retaining wall and foot cap. Fresh replay pending. |
| New garden controller regression | Pending fresh default-profile build |
| Fresh publisher and scoped Clippy | Fresh publisher build running; three-case native replay and scoped Clippy queued serially |
| Scoped Rust formatting | Pass |
| Whole workspace formatting | Existing unrelated differences beginning offset-allocator/src/lib.rs:408 |

All renders, rejected proposals, original captures, saves, RPC streams and logs remain in `renders/duluth-garden-connector-v01/`. The cached-library preview is unselected and separately identified. Explicit initial fixtures establish local geometry and input behavior, not earned regional progression.
