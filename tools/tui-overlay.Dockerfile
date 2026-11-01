# Optional additive release: preserve a verified live image's graphical assets,
# server, packs and Flygon. Tag the exact existing image as geothite:tui-base,
# or supply another explicitly pinned local tag via TUI_BASE_IMAGE.
# The full repository Dockerfile remains the canonical fresh-build path.
ARG TUI_BASE_IMAGE=geothite:tui-base
FROM ${TUI_BASE_IMAGE}
COPY --chown=65532:65532 tui/ /srv/crystal/web/tui/
