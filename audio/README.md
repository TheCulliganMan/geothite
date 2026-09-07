# Native Crystal audio

Audio synthesis and export use `crystal-audio` in this repository. The Rust
exporter reads the programs already embedded in the existing bundled content
pack. No separate disassembly checkout, extracted command catalog, TypeScript,
Node.js, or JavaScript synthesizer is required. The wave and drum tables were
moved from the previously bundled website runtime into the Rust renderer.

```sh
cargo run --release -p crystal-assets --bin export_audio
cargo run --release -p crystal-assets --bin export_audio -- --check
cargo test -p crystal-audio --lib
cargo test -p crystal-assets --test native_audio_corpus
```

Export updates only the existing content pack. `--check` renders every asset and
rejects stale PCM hashes, frame counts, or loop ranges without writing artifacts.
Shared calls, jumps, and loop bodies are linked from other programs in the pack;
missing references are errors. Loop discovery follows actual control flow.

The corpus test verifies all 1,130 shipped clips and all 102 music loop ranges.
The initial fix preserved the exact PCM hashes of 1,109 unaffected assets and
repaired 21 source-linkage or stale-metadata cases. Safari/WebKit also verified
all 1,130 clips against native PCM and repeated both repaired music loops twice.
A physical iPhone was not used for that check.

The website's thin worker loads Rust WASM and transfers PCM. Docker builds it
with Rust and wasm-bindgen, with TypeScript output disabled. The module and
worker use content-addressed URLs to keep cached versions consistent.
