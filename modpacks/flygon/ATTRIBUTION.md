# Flygon attribution

Connectome: MaleCNS v1.0, FlyEM / HHMI Janelia, University of Cambridge,
MRC Laboratory of Molecular Biology, Google Research and the release contributors.
Source: https://male-cns.janelia.org/download/
License: Creative Commons Attribution 4.0 International:
https://creativecommons.org/licenses/by/4.0/

Flygon transforms the source tables into a compact directed graph, retains cells
with an assigned superclass except Glia, and retains every pair edge among those
cells. Source soma positions are normalized for viewing. No somas are invented
for cells without coordinates. Rendered connections are schematic soma links.
Source hashes, graph identity and selection counts are in `flygon-dataset.json`.

The neural equations, sensory/current interface, reward teacher and action decoder
are engineered models. Simulated spikes are not recordings from a living animal.
No upstream project executable or source code is redistributed as Flygon.
Research references and pinned repository investigations are in docs/FLYGON_PLAN.md
in the source repository.

Flygon is an optional controller for an existing Geothite browser game bundle.
The packager preserves that game's assets; it does not supply a ROM or add game
content. Pokémon and associated names belong to their respective owners.
The Rust source follows the repository's license.
