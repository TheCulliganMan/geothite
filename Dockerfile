FROM rust:1.94-bookworm AS build
WORKDIR /source

RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.126 --locked

WORKDIR /source/rust
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
COPY vendor ./vendor
COPY crates ./crates
COPY modpacks/modern-move-split/data.json ./modpacks/modern-move-split/data.json
ARG CARGO_BUILD_JOBS=2
# Keep dependency and target caches across source edits. Run from the workspace
# so Cargo reads its WASM target configuration.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/source/rust/target,sharing=locked \
    cargo build --locked --release --package crystal-web-server --bin crystal-web-server \
    && cargo build --locked --profile web-release --package crystal-bevy \
        --bin crystal-bevy --features fullscreen-scaling,voxel-view --target wasm32-unknown-unknown \
    && cargo build --locked --profile web-release --package crystal-audio --lib \
        --features browser-synth --target wasm32-unknown-unknown \
    && mkdir -p /out/web \
    && cp target/release/crystal-web-server /out/crystal-web-server \
    && wasm-bindgen --target web --no-typescript --out-dir /out/web --out-name crystal-bevy \
        target/wasm32-unknown-unknown/web-release/crystal-bevy.wasm \
    && wasm-bindgen --target web --no-typescript --out-dir /out/web --out-name crystal-audio \
        target/wasm32-unknown-unknown/web-release/crystal_audio.wasm \
    && gzip -9 -k /out/web/crystal-audio_bg.wasm \
    && gzip -9 -k /out/web/crystal-bevy_bg.wasm

# Page, audio, and pack updates do not invalidate the Rust compilation layer.
COPY web-client /source/web-client
COPY tools/version-browser-bundle.sh /source/version-browser-bundle.sh
COPY content-packs/core-modular.browser.crystalpack /out/web/core-modular.browser.crystalpack
RUN cp /source/web-client/server-clock.js /source/web-client/audio-worker.js /source/web-client/audio-worker-client.js /out/web/ \
    && cp /source/web-client/player-customization.js /source/web-client/player-customization.css /source/web-client/index.html /source/web-client/audio-unlock.js /source/web-client/view-toggle.js /source/web-client/touch-controls.js /source/web-client/gamepad-controls.js /source/web-client/mobile-player.css /source/web-client/browser-session.js /source/web-client/webmcp.js /source/web-client/social-chat.js /source/web-client/social-chat.css /out/web/ \
    && gzip -9 -k /out/web/core-modular.browser.crystalpack \
    && sh /source/version-browser-bundle.sh /out/web

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=build /out/crystal-web-server /usr/local/bin/crystal-web-server
COPY --from=build --chown=65532:65532 /out/web /srv/crystal/web
COPY --chown=65532:65532 docker/runtime-data /srv/crystal/data

ENV CRYSTAL_HOST=0.0.0.0 \
    CRYSTAL_PORT=8080 \
    CRYSTAL_WEB_ROOT=/srv/crystal/web \
    CRYSTAL_PACK_DIR=/srv/crystal/packs

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/crystal-web-server"]
