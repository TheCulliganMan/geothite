# Repository rules

- Keep implementations in this Geothite repository. Audio synthesis and export
  must use Rust. Only minimal browser integration JavaScript belongs in the
  website; do not introduce TypeScript or an external exporter dependency.
- Do not add pret disassembly, ROMs, raw or exported command dumps, duplicate
  audio source catalogs, generated PCM files, or new compiled artifacts to this
  repository or its shipped bundle. Preserve the existing bundled artifact
  inventory. Regenerate the already tracked content pack when fixing its data.
- Read audio programs from the existing bundled content pack. Do not extract
  and commit another copy of those programs or add generated audit manifests.
- Build products and temporary verification output are not deliverables to
  commit. Keep them ignored and remove task-generated scratch artifacts when
  finished. Never stage them as part of an “all code” commit.
- Verify audio changes against the pack's PCM hashes, frame counts, and loop
  ranges. Preserve unaffected audio and unrelated game content.
