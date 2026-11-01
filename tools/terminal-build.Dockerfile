# Export a prebuilt Linux text client into an ignored, external release directory.
FROM rust:1.99.0-bookworm AS build
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config libasound2-dev libudev-dev libx11-dev libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /source/rust
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .cargo ./.cargo
COPY crates ./crates
COPY assets ./assets
COPY art ./art
COPY modpacks ./modpacks
ARG CARGO_BUILD_JOBS=2
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/source/rust/target,sharing=locked \
    find crates -type f -exec touch {} + \
    && cargo build --release --locked -p geothite \
    && mkdir -p /out \
    && cp target/release/geothite /out/geothite

FROM scratch AS export
COPY --from=build /out/geothite /geothite
